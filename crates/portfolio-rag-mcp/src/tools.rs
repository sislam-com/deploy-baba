use anyhow::Result;
use serde_json::Value;
use tracing::{error, info};

use crate::rag::PortfolioRAG;

const DEFAULT_TOP_K: usize = 5;
const DEFAULT_MAX_CONTENT_LEN: usize = 500;

#[derive(Clone, serde::Serialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "context_brief".to_string(),
            description: Some(
                "Get a compact context summary (~500 tokens) for a task. Returns relevant corpus names, top document paths, and one-line summaries. Use this FIRST to decide what to drill into, then query_rag for full content.".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "Description of the task or question you need context for"
                    }
                },
                "required": ["task"]
            }),
        },
        Tool {
            name: "query_rag".to_string(),
            description: Some(
                "Query the portfolio RAG system. Supports corpus filtering, result count, and content truncation for token efficiency. Use context_brief first for orientation, then this tool for detail.".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "corpus": {
                        "type": "string",
                        "description": "Optional source_kind filter (openapi, portfolio, rust, hcl, plan, cache, challenge)"
                    },
                    "top_k": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 5). Use 3 for quick lookups.",
                        "default": 5
                    },
                    "max_content_length": {
                        "type": "integer",
                        "description": "Max chars per result content (default: 500). Set 0 for full content. Truncated results have truncated=true.",
                        "default": 500
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "list_corpora".to_string(),
            description: Some("List all available RAG corpora with names".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_corpus_stats".to_string(),
            description: Some(
                "Get real statistics for a corpus: document count, chunk count, avg token size, date range".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "corpus": {
                        "type": "string",
                        "description": "The corpus name (openapi, portfolio, rust, hcl, plan, cache, challenge)"
                    }
                },
                "required": ["corpus"]
            }),
        },
        Tool {
            name: "project_health".to_string(),
            description: Some(
                "Get project health metrics: plan coverage %, open drift items, cache age, RAG index stats. Use for AI-DLC health dashboards.".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

pub async fn context_brief(rag: &PortfolioRAG, args: Value) -> Result<Value> {
    let task = args
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'task' parameter"))?;

    info!("Context brief request: '{}'", task);

    match rag.query(task, None, 3, Some(80)).await {
        Ok(results) => {
            let mut relevant_corpora: Vec<&str> = Vec::new();
            let briefs: Vec<Value> = results
                .iter()
                .map(|r| {
                    let corpus = r.get("corpus").and_then(|v| v.as_str()).unwrap_or("unknown");
                    if !relevant_corpora.contains(&corpus) {
                        relevant_corpora.push(corpus);
                    }
                    serde_json::json!({
                        "path": r.get("source_path").and_then(|v| v.as_str()).unwrap_or(""),
                        "corpus": corpus,
                        "preview": r.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                        "score": r.get("score"),
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "success": true,
                "task": task,
                "relevant_corpora": relevant_corpora,
                "top_documents": briefs,
                "hint": "Use query_rag with corpus filter and top_k for detailed content"
            }))
        }
        Err(e) => {
            error!("Context brief failed: {}", e);
            Err(anyhow::anyhow!("Context brief failed: {}", e))
        }
    }
}

pub async fn query_rag(rag: &PortfolioRAG, args: Value) -> Result<Value> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

    let corpus = args.get("corpus").and_then(|v| v.as_str());
    let top_k = args
        .get("top_k")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(DEFAULT_TOP_K);
    let max_content_len = args
        .get("max_content_length")
        .and_then(|v| v.as_u64())
        .map(|v| if v == 0 { None } else { Some(v as usize) })
        .unwrap_or(Some(DEFAULT_MAX_CONTENT_LEN));

    info!(
        "RAG query: '{}' (corpus: {:?}, top_k: {}, max_content: {:?})",
        query, corpus, top_k, max_content_len
    );

    match rag.query(query, corpus, top_k, max_content_len).await {
        Ok(results) => {
            let response = serde_json::json!({
                "success": true,
                "query": query,
                "corpus_filter": corpus,
                "results": results,
                "result_count": results.len()
            });
            Ok(response)
        }
        Err(e) => {
            error!("RAG query failed: {}", e);
            Err(anyhow::anyhow!("RAG query failed: {}", e))
        }
    }
}

pub fn list_corpora(rag: &PortfolioRAG) -> Result<Value> {
    let corpora = rag.get_corpora();

    Ok(serde_json::json!({
        "success": true,
        "corpora": corpora,
        "corpus_count": corpora.len()
    }))
}

pub fn get_corpus_stats(rag: &PortfolioRAG, args: Value) -> Result<Value> {
    let corpus = args
        .get("corpus")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'corpus' parameter"))?;

    info!("Corpus stats request for: {}", corpus);

    match rag.get_corpus_stats(corpus) {
        Ok(stats) => Ok(serde_json::json!({
            "success": true,
            "stats": stats
        })),
        Err(e) => {
            error!("Corpus stats failed: {}", e);
            Err(anyhow::anyhow!("Corpus stats failed: {}", e))
        }
    }
}

pub fn project_health(rag: &PortfolioRAG) -> Result<Value> {
    info!("Project health request");

    match rag.project_health() {
        Ok(health) => Ok(serde_json::json!({
            "success": true,
            "health": health
        })),
        Err(e) => {
            error!("Project health failed: {}", e);
            Err(anyhow::anyhow!("Project health failed: {}", e))
        }
    }
}
