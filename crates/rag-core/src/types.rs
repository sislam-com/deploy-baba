//! Core data types shared across the RAG pipeline.

use serde::{Deserialize, Serialize};

/// The kind of source artifact a chunk came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Rust,
    Hcl,
    Plan,
    Cache,
    OpenApi,
    Portfolio,
    TypeScript,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::Rust => "rust",
            SourceKind::Hcl => "hcl",
            SourceKind::Plan => "plan",
            SourceKind::Cache => "cache",
            SourceKind::OpenApi => "openapi",
            SourceKind::Portfolio => "portfolio",
            SourceKind::TypeScript => "typescript",
        }
    }
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single indexable unit extracted from a source artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Ordinal position within the source file (0-based).
    pub ord: usize,
    /// The text content of this chunk.
    pub content: String,
    /// Approximate token count (word-count proxy; exact count via embedder).
    pub token_count: usize,
    /// Freeform metadata (e.g. `{"fn_name": "...", "item_kind": "fn"}`).
    pub meta: serde_json::Value,
}

/// A chunk retrieved from the index, annotated with retrieval rank + score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedChunk {
    pub chunk_id: i64,
    pub document_id: i64,
    pub source_kind: String,
    pub source_path: String,
    pub git_sha: String,
    pub ord: i64,
    pub content: String,
    /// Combined RRF score (higher = more relevant).
    pub score: f64,
}

/// A grounded prompt ready to be sent to an LLM.
#[derive(Debug, Clone)]
pub struct PromptBundle {
    /// System prompt with grounding contract and citation instructions.
    pub system_prompt: String,
    /// User-visible question with injected source citations.
    pub user_message: String,
    /// The source citations included, for logging and attribution.
    pub citations: Vec<CitationRef>,
}

/// Reference to a source chunk included in a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRef {
    pub kind: String,
    pub path: String,
    pub sha: String,
    pub ord: i64,
}
