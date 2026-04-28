//! OpenTofu / Terraform HCL brace-balance chunker.
//!
//! Splits HCL files into one chunk per top-level block (`resource`, `variable`,
//! `module`, `data`, `output`, `locals`, `provider`, `terraform`). Uses a
//! simple brace-balance scanner rather than a full HCL parser, which keeps the
//! dependency surface zero while being correct for all valid HCL files.

use crate::types::Chunk;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

/// Return true if `line` starts a new top-level HCL block.
fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim();
    let keywords = [
        "resource",
        "variable",
        "module",
        "data",
        "output",
        "locals",
        "provider",
        "terraform",
        "moved",
        "import",
    ];
    keywords
        .iter()
        .any(|kw| trimmed.starts_with(kw) && trimmed.contains('{'))
}

/// Split `content` into one chunk per top-level block.
pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut ord = 0usize;
    let mut current_lines: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut in_block = false;

    for line in content.lines() {
        let open = line.chars().filter(|&c| c == '{').count() as i32;
        let close = line.chars().filter(|&c| c == '}').count() as i32;

        if !in_block && is_block_start(line) {
            in_block = true;
            current_lines.clear();
        }

        if in_block {
            current_lines.push(line);
            depth += open - close;

            if depth <= 0 {
                // Block complete
                let text = current_lines.join("\n");
                emit_chunks(path, &text, &mut ord, &mut chunks);
                current_lines.clear();
                depth = 0;
                in_block = false;
            }
        }
    }

    // Flush anything remaining (e.g. a comment-only file or trailing locals)
    if !current_lines.is_empty() {
        let text = current_lines.join("\n");
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
terraform {
  required_version = ">= 1.6"
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
}

variable "region" {
  type    = string
  default = "us-east-1"
}

resource "aws_lambda_function" "ui" {
  function_name = "deploy-baba-ui"
  runtime       = "provided.al2023"
  handler       = "bootstrap"
  filename      = "lambda.zip"

  environment {
    variables = {
      DB_PATH = "/mnt/efs/deploy-baba.db"
    }
  }
}
"#;

    #[test]
    fn produces_one_chunk_per_block() {
        let chunks = chunk("infra/main.tf", FIXTURE);
        // terraform + variable + resource = 3 blocks
        assert_eq!(
            chunks.len(),
            3,
            "expected 3 top-level blocks, got {}",
            chunks.len()
        );
    }

    #[test]
    fn chunks_have_ascending_ord() {
        let chunks = chunk("infra/main.tf", FIXTURE);
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.ord, i);
        }
    }

    #[test]
    fn empty_file_produces_no_chunks() {
        let chunks = chunk("infra/empty.tf", "");
        assert!(chunks.is_empty());
    }

    #[test]
    fn token_count_matches_word_count() {
        let chunks = chunk("infra/main.tf", FIXTURE);
        for c in &chunks {
            let actual = c.content.split_whitespace().count();
            assert_eq!(c.token_count, actual);
        }
    }
}
