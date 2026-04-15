//! Rust source code chunker (P1 — regex-based fn/impl/struct/enum/trait splitter).
//!
//! For P1 FTS-only indexing we use a simple brace-balance scanner that splits
//! at top-level `fn`, `impl`, `struct`, `enum`, `trait`, and `pub mod` items.
//! This avoids the `syn` dependency while giving per-item chunk granularity that
//! is good enough for keyword retrieval. A `syn`-based chunker is planned for
//! W-RAG P2 when semantic accuracy matters more.

use crate::types::Chunk;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

/// Keywords that start a top-level Rust item we want to split on.
static ITEM_KEYWORDS: &[&str] = &[
    "pub fn ",
    "pub async fn ",
    "pub unsafe fn ",
    "fn ",
    "async fn ",
    "impl ",
    "pub impl ",
    "pub struct ",
    "struct ",
    "pub enum ",
    "enum ",
    "pub trait ",
    "trait ",
    "pub mod ",
    "mod ",
    "pub type ",
    "type ",
    "pub const ",
    "const ",
    "pub static ",
    "static ",
    "/// ",
    "//! ",
];

fn is_item_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    ITEM_KEYWORDS.iter().any(|kw| trimmed.starts_with(kw))
}

/// Split Rust source into one chunk per top-level item.
///
/// Uses brace-balance tracking: accumulates lines until `{...}` are balanced,
/// then emits a chunk. Module-level docs (`///` or `//!`) are attached to the
/// following item.
pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut ord = 0usize;
    let mut current: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut in_item = false;

    for line in content.lines() {
        let open = line.chars().filter(|&c| c == '{').count() as i32;
        let close = line.chars().filter(|&c| c == '}').count() as i32;

        if !in_item {
            if is_item_start(line) {
                in_item = true;
                current.push(line);
                depth = open - close;
                // Single-line items (e.g. `type Alias = Foo;` or `const X: u32 = 1;`)
                if depth <= 0 && !line.contains('{') {
                    emit_chunks(path, &current.join("\n"), &mut ord, &mut chunks);
                    current.clear();
                    in_item = false;
                    depth = 0;
                }
            }
            // Non-item lines (use, extern crate, attributes) are skipped
            // unless already accumulating
        } else {
            current.push(line);
            depth += open - close;
            if depth <= 0 {
                emit_chunks(path, &current.join("\n"), &mut ord, &mut chunks);
                current.clear();
                depth = 0;
                in_item = false;
            }
        }
    }

    // Flush any trailing content
    if !current.is_empty() {
        let text = current.join("\n");
        if !text.trim().is_empty() {
            emit_chunks(path, &text, &mut ord, &mut chunks);
        }
    }

    chunks
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
use std::collections::HashMap;

pub struct Config {
    pub name: String,
    pub value: u32,
}

impl Config {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned(), value: 0 }
    }
}

pub fn process(config: &Config) -> String {
    format!("{}: {}", config.name, config.value)
}

pub trait Processable {
    fn process(&self) -> String;
}
"#;

    #[test]
    fn produces_non_empty_chunks() {
        let chunks = chunk("src/lib.rs", FIXTURE);
        assert!(!chunks.is_empty(), "should produce at least one chunk");
    }

    #[test]
    fn captures_struct_and_impl_and_fn() {
        let chunks = chunk("src/lib.rs", FIXTURE);
        let joined: String = chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(joined.contains("Config"), "should contain struct Config");
        assert!(joined.contains("process"), "should contain fn process");
    }

    #[test]
    fn chunks_have_ascending_ord() {
        let chunks = chunk("src/lib.rs", FIXTURE);
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.ord, i);
        }
    }

    #[test]
    fn token_count_matches_word_count() {
        let chunks = chunk("src/lib.rs", FIXTURE);
        for c in &chunks {
            let actual = c.content.split_whitespace().count();
            assert_eq!(c.token_count, actual);
        }
    }

    #[test]
    fn empty_file_produces_no_chunks() {
        let chunks = chunk("src/empty.rs", "");
        assert!(chunks.is_empty());
    }
}
