//! Vendor-agnostic RAG traits, chunkers, and prompt assembly for deploy-baba.
//!
//! # Design
//!
//! Mirrors the workspace `-core` + adapter pattern. Zero vendor SDK dependencies.
//! `rag-sqlite` is the first concrete implementation of [`Retriever`].
//!
//! The grounding contract (ADR-016 / `cross-cutting/llm-policy.md`) is enforced
//! by [`PromptAssembler`]: every retrieved chunk is wrapped in a `<source …>`
//! citation tag; the system prompt requires the model to cite all claims.

pub mod chunk;
pub mod error;
pub mod types;

pub use error::RagError;
pub use types::{Chunk, CitationRef, PromptBundle, RankedChunk, SourceKind};

use async_trait::async_trait;

// ── Traits ────────────────────────────────────────────────────────────────

/// Text embedding provider — produces dense float vectors from text.
///
/// In P1 FTS-only mode there is no concrete implementation; this trait is
/// declared for forward-compatibility. `rag-sqlite` retrieves purely via
/// FTS5 BM25 when no `Embedder` is wired.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Stable identifier, e.g. `"anthropic"`.
    fn provider_id(&self) -> &'static str;

    /// Dimension of the embedding vectors produced.
    fn dim(&self) -> usize;

    /// Embed a batch of texts into float vectors.
    ///
    /// # Errors
    ///
    /// Returns [`RagError::Embedder`] on upstream failure.
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, RagError>;
}

/// Hybrid retriever — returns ranked chunks for a natural-language query.
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve the top `top_k` chunks most relevant to `query`.
    ///
    /// In P1 (FTS-only) this uses SQLite FTS5 BM25. In P2 the result is
    /// fused with ANN (sqlite-vec) scores via RRF.
    ///
    /// # Errors
    ///
    /// Returns [`RagError::Database`] on SQLite failure.
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RankedChunk>, RagError>;
}

/// Assembles a grounded prompt from retrieved chunks.
pub trait PromptAssembler {
    /// Wrap `chunks` in citation tags and build a [`PromptBundle`] for the LLM.
    ///
    /// The system prompt enforces the grounding contract:
    /// - All claims must cite a `[source N]` marker.
    /// - The model may not invent content not present in the provided sources.
    fn assemble(&self, query: &str, chunks: &[RankedChunk]) -> PromptBundle;
}

// ── Default PromptAssembler ───────────────────────────────────────────────

/// A simple [`PromptAssembler`] implementation that wraps each chunk in an
/// XML-style `<source …>` citation tag (ADR-016 format).
pub struct DefaultPromptAssembler;

impl PromptAssembler for DefaultPromptAssembler {
    fn assemble(&self, query: &str, chunks: &[RankedChunk]) -> PromptBundle {
        let mut citations = Vec::new();
        let mut sources_text = String::new();

        for (i, chunk) in chunks.iter().enumerate() {
            let n = i + 1;
            sources_text.push_str(&format!(
                "<source n=\"{n}\" kind=\"{}\" path=\"{}\" sha=\"{}\">\n{}\n</source>\n\n",
                chunk.source_kind, chunk.source_path, chunk.git_sha, chunk.content
            ));
            citations.push(CitationRef {
                kind: chunk.source_kind.clone(),
                path: chunk.source_path.clone(),
                sha: chunk.git_sha.clone(),
                ord: chunk.ord,
            });
        }

        let system_prompt = format!(
            "You are an expert on the deploy-baba codebase. Answer the user's question \
             using ONLY the sources provided below. Cite every claim with [source N] \
             where N matches the source number. Do not invent information not present \
             in the sources.\n\n{sources_text}"
        );

        let user_message = query.to_owned();

        PromptBundle {
            system_prompt,
            user_message,
            citations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RankedChunk;

    fn make_chunk(n: usize, content: &str) -> RankedChunk {
        RankedChunk {
            chunk_id: n as i64,
            document_id: 1,
            source_kind: "plan".to_string(),
            source_path: format!("plans/test{n}.md"),
            git_sha: "abc123".to_string(),
            ord: n as i64,
            content: content.to_string(),
            score: 1.0,
        }
    }

    #[test]
    fn assembler_includes_all_citations() {
        let assembler = DefaultPromptAssembler;
        let chunks = vec![
            make_chunk(0, "SQLite is used for storage."),
            make_chunk(1, "ADR-002 says no PostgreSQL."),
        ];
        let bundle = assembler.assemble("What database does deploy-baba use?", &chunks);

        assert_eq!(bundle.citations.len(), 2);
        assert!(bundle.system_prompt.contains("<source n=\"1\""));
        assert!(bundle.system_prompt.contains("<source n=\"2\""));
        assert!(bundle.system_prompt.contains("SQLite is used for storage."));
        assert!(bundle.user_message.contains("deploy-baba"));
    }

    #[test]
    fn assembler_empty_chunks_has_no_citations() {
        let assembler = DefaultPromptAssembler;
        let bundle = assembler.assemble("what?", &[]);
        assert!(bundle.citations.is_empty());
    }
}
