//! Per-corpus chunkers for the RAG index pipeline.
//!
//! Each submodule implements a strategy tuned to its file format:
//!
//! | Module | Format | Strategy |
//! |--------|--------|----------|
//! | [`markdown`] | `.md` | H2/H3 heading split |
//! | [`hcl`] | `.tf` | Brace-balance top-level block split |
//! | [`rust`] | `.rs` | Keyword + brace-balance item split |
//! | [`claude_cache`] | `.json` / `.md` / other | JSON-leaf or heading-split or sliding window |
//!
//! All chunkers return `Vec<Chunk>` with ascending `ord` values and approximate
//! word-count `token_count`. Oversize chunks are further split with a 50-word
//! sliding-window overlap.

pub mod claude_cache;
pub mod hcl;
pub mod markdown;
pub mod rust;

use std::path::Path;

use crate::types::{Chunk, SourceKind};

/// Dispatch to the appropriate chunker based on `kind` and file extension.
pub fn chunk_file(kind: &SourceKind, path: &Path, content: &str) -> Vec<Chunk> {
    let path_str = path.to_string_lossy();
    match kind {
        SourceKind::Rust => rust::chunk(&path_str, content),
        SourceKind::Hcl => hcl::chunk(&path_str, content),
        SourceKind::Plan => markdown::chunk(&path_str, content),
        SourceKind::Cache => claude_cache::chunk(&path_str, content),
    }
}
