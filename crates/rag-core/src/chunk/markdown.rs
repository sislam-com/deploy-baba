//! Markdown heading-split chunker.
//!
//! Splits a Markdown file at H2 (`##`) and H3 (`###`) headings. Each section
//! (heading + content up to the next heading) becomes one [`Chunk`]. The preamble
//! before the first heading is included as chunk 0 if non-empty.

use crate::types::Chunk;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

/// Split `content` into chunks at H2/H3 boundaries.
///
/// Chunks that exceed `MAX_TOKENS` words are further split with a sliding
/// window to stay within the limit.
pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    let mut sections: Vec<(String, String)> = Vec::new(); // (heading, body)
    let mut current_heading = String::new();
    let mut current_body = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") || line.starts_with("### ") {
            if !current_body.is_empty() || !current_heading.is_empty() {
                sections.push((current_heading.clone(), current_body.join("\n")));
            }
            current_heading = line.to_owned();
            current_body = Vec::new();
        } else {
            current_body.push(line);
        }
    }
    // Flush the last section
    if !current_body.is_empty() || !current_heading.is_empty() {
        sections.push((current_heading, current_body.join("\n")));
    }

    let mut chunks = Vec::new();
    let mut ord = 0usize;

    for (heading, body) in sections {
        let full_text = if heading.is_empty() {
            body.trim().to_owned()
        } else {
            format!("{}\n\n{}", heading, body.trim())
        };

        if full_text.is_empty() {
            continue;
        }

        let words: Vec<&str> = full_text.split_whitespace().collect();
        if words.len() <= MAX_TOKENS {
            let token_count = words.len();
            chunks.push(Chunk {
                ord,
                content: full_text,
                token_count,
                meta: serde_json::json!({ "path": path, "heading": heading }),
            });
            ord += 1;
        } else {
            // Sliding window split
            let mut start = 0;
            while start < words.len() {
                let end = (start + MAX_TOKENS).min(words.len());
                let text = words[start..end].join(" ");
                let token_count = end - start;
                chunks.push(Chunk {
                    ord,
                    content: text,
                    token_count,
                    meta: serde_json::json!({
                        "path": path,
                        "heading": heading,
                        "window_start": start,
                    }),
                });
                ord += 1;
                if end == words.len() {
                    break;
                }
                start += MAX_TOKENS - OVERLAP_WORDS;
            }
        }
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
# Top Title

Preamble text before any H2.

## Section One

Content of section one. This is some text about Rust and async systems.

## Section Two

Content of section two. SQLite and EFS storage.

### Sub-section

A sub-section under two.
";

    #[test]
    fn produces_non_empty_chunks() {
        let chunks = chunk("plans/test.md", FIXTURE);
        assert!(!chunks.is_empty(), "should produce at least one chunk");
    }

    #[test]
    fn sections_are_separate_chunks() {
        let chunks = chunk("plans/test.md", FIXTURE);
        // Should have preamble + section-one + section-two + sub-section
        assert!(chunks.len() >= 3);
    }

    #[test]
    fn chunks_have_ascending_ord() {
        let chunks = chunk("plans/test.md", FIXTURE);
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.ord, i);
        }
    }

    #[test]
    fn token_count_matches_word_count() {
        let chunks = chunk("plans/test.md", FIXTURE);
        for c in &chunks {
            let actual = c.content.split_whitespace().count();
            assert_eq!(c.token_count, actual);
        }
    }

    #[test]
    fn oversize_chunk_is_split() {
        let many_words = "word ".repeat(1600);
        let content = format!("## Long Section\n\n{}", many_words);
        let chunks = chunk("test.md", &content);
        for c in &chunks {
            assert!(
                c.token_count <= MAX_TOKENS,
                "chunk {} has {} tokens > {MAX_TOKENS}",
                c.ord,
                c.token_count
            );
        }
        assert!(
            chunks.len() >= 2,
            "oversize section should split into multiple chunks"
        );
    }
}
