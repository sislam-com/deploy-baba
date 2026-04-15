//! `.claude/` agent cache chunker (local CLI only — never bundled into Lambda).
//!
//! The `.claude/` directory contains JSON index files, memory files (Markdown),
//! and skill definitions. This chunker dispatches by file extension:
//! - `.json` → JSON-leaf splitter (one chunk per top-level key-value pair)
//! - `.md` → heading-split (delegates to the markdown chunker)
//! - Everything else → treated as plain text and chunked by sliding window

use crate::types::Chunk;

use super::markdown;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

/// Chunk a `.claude/` cache file.
///
/// `path` is the file path (used to determine strategy by extension).
pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    if path.ends_with(".md") {
        return markdown::chunk(path, content);
    }
    if path.ends_with(".json") {
        return chunk_json(path, content);
    }
    // Plain text fallback
    chunk_plain(path, content)
}

/// Split a JSON object into one chunk per top-level key.
///
/// Falls back to `chunk_plain` if the content is not a JSON object or an array.
fn chunk_json(path: &str, content: &str) -> Vec<Chunk> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return chunk_plain(path, content);
    };

    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .enumerate()
            .map(|(ord, (key, val))| {
                let text = format!(
                    "{key}: {}",
                    serde_json::to_string_pretty(val).unwrap_or_default()
                );
                let token_count = text.split_whitespace().count();
                Chunk {
                    ord,
                    content: text,
                    token_count,
                    meta: serde_json::json!({ "path": path, "key": key }),
                }
            })
            .collect(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .enumerate()
            .map(|(ord, val)| {
                let text = serde_json::to_string_pretty(val).unwrap_or_default();
                let token_count = text.split_whitespace().count();
                Chunk {
                    ord,
                    content: text,
                    token_count,
                    meta: serde_json::json!({ "path": path, "index": ord }),
                }
            })
            .collect(),
        _ => chunk_plain(path, content),
    }
}

/// Sliding-window plain-text chunker.
fn chunk_plain(path: &str, content: &str) -> Vec<Chunk> {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut ord = 0usize;
    let mut start = 0;

    while start < words.len() {
        let end = (start + MAX_TOKENS).min(words.len());
        let text = words[start..end].join(" ");
        let token_count = end - start;
        chunks.push(Chunk {
            ord,
            content: text,
            token_count,
            meta: serde_json::json!({ "path": path, "window_start": start }),
        });
        ord += 1;
        if end == words.len() {
            break;
        }
        start += MAX_TOKENS - OVERLAP_WORDS;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_object_one_chunk_per_key() {
        let json = r#"{"foo": "bar", "baz": 42}"#;
        let chunks = chunk(".claude/index.json", json);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].content.contains("foo") || chunks[1].content.contains("foo"));
    }

    #[test]
    fn json_array_one_chunk_per_element() {
        let json = r#"["alpha", "beta", "gamma"]"#;
        let chunks = chunk(".claude/list.json", json);
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn md_file_delegates_to_markdown_chunker() {
        let md = "## Heading\n\nSome text here.\n\n## Another\n\nMore text.";
        let chunks_cache = chunk(".claude/memory.md", md);
        let chunks_md = markdown::chunk(".claude/memory.md", md);
        assert_eq!(chunks_cache.len(), chunks_md.len());
    }

    #[test]
    fn plain_text_sliding_window() {
        let text = "word ".repeat(1600);
        let chunks = chunk(".claude/misc.txt", text.trim());
        for c in &chunks {
            assert!(c.token_count <= 800);
        }
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn chunks_have_ascending_ord() {
        let json = r#"{"a": 1, "b": 2, "c": 3}"#;
        let chunks = chunk(".claude/x.json", json);
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.ord, i);
        }
    }
}
