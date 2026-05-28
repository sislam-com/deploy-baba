use crate::types::Chunk;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

static ITEM_KEYWORDS: &[&str] = &["def ", "async def ", "class ", "@tool", "@app.", "@router."];

fn is_item_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    ITEM_KEYWORDS.iter().any(|kw| trimmed.starts_with(kw))
}

fn is_decorator(line: &str) -> bool {
    line.trim_start().starts_with('@')
}

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut ord = 0usize;
    let mut current: Vec<&str> = Vec::new();
    let mut item_indent: Option<usize> = None;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if item_indent.is_none() {
            if is_decorator(line) || is_item_start(line) {
                if is_decorator(line) {
                    current.push(line);
                    i += 1;
                    while i < lines.len() && is_decorator(lines[i]) {
                        current.push(lines[i]);
                        i += 1;
                    }
                    if i < lines.len() && is_item_start(lines[i]) {
                        item_indent = Some(indent_level(lines[i]));
                        current.push(lines[i]);
                        i += 1;
                        continue;
                    }
                } else {
                    item_indent = Some(indent_level(line));
                    current.push(line);
                    i += 1;
                    continue;
                }
            }
            i += 1;
        } else {
            let base = item_indent.unwrap();
            if !trimmed.is_empty()
                && indent_level(line) <= base
                && !line.trim_start().starts_with('#')
            {
                if is_item_start(line) || is_decorator(line) {
                    flush_item(path, &current, &mut ord, &mut chunks);
                    current.clear();
                    item_indent = None;
                    continue;
                } else {
                    flush_item(path, &current, &mut ord, &mut chunks);
                    current.clear();
                    item_indent = None;
                    i += 1;
                    continue;
                }
            }
            current.push(line);
            i += 1;
        }
    }

    if !current.is_empty() {
        flush_item(path, &current, &mut ord, &mut chunks);
    }

    chunks
}

fn flush_item(path: &str, lines: &[&str], ord: &mut usize, chunks: &mut Vec<Chunk>) {
    let text = lines.join("\n");
    if text.trim().is_empty() {
        return;
    }
    emit_chunks(path, &text, ord, chunks);
}

fn emit_chunks(path: &str, text: &str, ord: &mut usize, chunks: &mut Vec<Chunk>) {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return;
    }
    if words.len() <= MAX_TOKENS {
        chunks.push(Chunk {
            ord: *ord,
            content: text.to_owned(),
            token_count: words.len(),
            meta: serde_json::json!({ "path": path }),
        });
        *ord += 1;
    } else {
        let mut start = 0;
        while start < words.len() {
            let end = (start + MAX_TOKENS).min(words.len());
            let chunk_text = words[start..end].join(" ");
            let token_count = end - start;
            chunks.push(Chunk {
                ord: *ord,
                content: chunk_text,
                token_count,
                meta: serde_json::json!({ "path": path, "window_start": start }),
            });
            *ord += 1;
            if end == words.len() {
                break;
            }
            start += MAX_TOKENS - OVERLAP_WORDS;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
"""LangGraph ReAct agent for cover letter generation."""

from __future__ import annotations

import os
from typing import Any

from langchain_anthropic import ChatAnthropic
from langgraph.graph import StateGraph


SYSTEM_PROMPT = "You are a cover letter assistant."


def _get_llm() -> ChatAnthropic:
    """Return the configured Anthropic LLM."""
    model = os.environ.get("ANTHROPIC_MODEL", "claude-sonnet-4-5-20250929")
    return ChatAnthropic(model=model, max_tokens=4096)


async def agent_node(state: dict) -> dict:
    """Call the LLM with tools bound."""
    llm = _get_llm()
    response = await llm.ainvoke(state["messages"])
    return {"messages": [response]}


class RAGSyncState:
    """State for the RAG sync graph."""
    health: dict | None = None
    failures: list | None = None
"#;

    #[test]
    fn produces_non_empty_chunks() {
        let chunks = chunk("services/agent/src/agent/graph.py", FIXTURE);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn captures_functions() {
        let chunks = chunk("services/agent/src/agent/graph.py", FIXTURE);
        let joined: String = chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(joined.contains("_get_llm"), "should contain function name");
        assert!(
            joined.contains("agent_node"),
            "should contain async function name"
        );
    }

    #[test]
    fn captures_class() {
        let chunks = chunk("services/agent/src/agent/graph.py", FIXTURE);
        let joined: String = chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(joined.contains("RAGSyncState"), "should contain class name");
    }

    #[test]
    fn captures_decorated_functions() {
        let decorated = r#"
from langchain_core.tools import tool

@tool
def check_rag_health() -> str:
    """Get RAG system health."""
    return "healthy"

@tool
def get_eval_report() -> str:
    """Get eval report."""
    return "report"
"#;
        let chunks = chunk("services/agent/src/agent/tools/rag_eval.py", decorated);
        assert!(!chunks.is_empty());
        let joined: String = chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            joined.contains("check_rag_health"),
            "should contain decorated function"
        );
        assert!(joined.contains("@tool"), "should include decorator");
    }

    #[test]
    fn empty_file_produces_no_chunks() {
        let chunks = chunk("empty.py", "");
        assert!(chunks.is_empty());
    }

    #[test]
    fn chunks_have_ascending_ord() {
        let chunks = chunk("services/agent/src/agent/graph.py", FIXTURE);
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.ord, i);
        }
    }
}
