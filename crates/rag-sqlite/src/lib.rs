//! SQLite + FTS5 retrieval backend for the deploy-baba RAG pipeline.
//!
//! # P1 — FTS-only mode
//!
//! In P1 no embedding provider is required. Retrieval runs purely via SQLite
//! FTS5 BM25 ranking. The schema is forward-compatible: the `rag_vec` table
//! is created when an `Embedder` is wired (W-RAG P2), but the `RagStore` runs
//! fully without it.
//!
//! # Schema
//!
//! Three tables created by [`MIGRATION_SQL`]:
//!
//! - `rag_documents` — one row per indexed file (path + sha + kind)
//! - `rag_chunks` — one row per chunk within a document
//! - `rag_chunks_fts` — FTS5 virtual table backed by `rag_chunks.content`
//!
//! All `INSERT` statements use `ON CONFLICT DO UPDATE` (ADR-010).

use async_trait::async_trait;
use rag_core::{RagError, RankedChunk, Retriever};
use rusqlite::{params, Connection};
use std::sync::Mutex;

// ── Schema migration ──────────────────────────────────────────────────────

/// DDL for the RAG index tables. Consumed by `services/ui/db.rs` via
/// `include_str!("../migrations/016_rag_index.sql")`.
///
/// Exposed here so the migration file stays the single canonical source but
/// tests can also run it in-memory.
pub const MIGRATION_SQL: &str = include_str!("migration.sql");

// ── RagStore ──────────────────────────────────────────────────────────────

/// SQLite-backed RAG store.
///
/// Wraps a `Mutex<Connection>` (same pattern as `services/ui`'s `Db`).
/// `Arc<RagStore>` can be shared across tokio tasks.
pub struct RagStore {
    conn: Mutex<Connection>,
}

impl RagStore {
    /// Open (or create) a RAG store backed by the given SQLite connection.
    ///
    /// Applies the RAG schema migration immediately.
    ///
    /// # Errors
    ///
    /// Returns [`RagError::Database`] if the migration fails.
    pub fn new(conn: Connection) -> Result<Self, RagError> {
        conn.execute_batch(MIGRATION_SQL)
            .map_err(|e| RagError::Database(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an in-memory store for testing.
    pub fn in_memory() -> Result<Self, RagError> {
        let conn = Connection::open_in_memory().map_err(|e| RagError::Database(e.to_string()))?;
        Self::new(conn)
    }

    // ── Ingest ────────────────────────────────────────────────────────────

    /// Upsert a document and its chunks into the index.
    ///
    /// Uses ADR-010 `ON CONFLICT DO UPDATE` semantics so re-indexing is
    /// idempotent. After upserting chunks the FTS5 table is rebuilt to stay
    /// in sync.
    ///
    /// # Errors
    ///
    /// Returns [`RagError::Database`] on any SQLite error.
    pub fn upsert_document(
        &self,
        source_kind: &str,
        source_path: &str,
        git_sha: &str,
        chunks: &[rag_core::Chunk],
    ) -> Result<(), RagError> {
        let conn = self.conn.lock().unwrap();

        // Upsert the document row
        conn.execute(
            "INSERT INTO rag_documents (source_kind, source_path, git_sha, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT(source_kind, source_path) DO UPDATE SET
               git_sha    = excluded.git_sha,
               updated_at = excluded.updated_at",
            params![source_kind, source_path, git_sha],
        )
        .map_err(|e| RagError::Database(e.to_string()))?;

        let doc_id: i64 = conn
            .query_row(
                "SELECT id FROM rag_documents WHERE source_kind = ?1 AND source_path = ?2",
                params![source_kind, source_path],
                |row| row.get(0),
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        // Delete old chunks for this document (cascade not guaranteed on FTS)
        conn.execute(
            "DELETE FROM rag_chunks WHERE document_id = ?1",
            params![doc_id],
        )
        .map_err(|e| RagError::Database(e.to_string()))?;

        // Insert new chunks
        for chunk in chunks {
            conn.execute(
                "INSERT INTO rag_chunks (document_id, ord, content, token_count, meta_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(document_id, ord) DO UPDATE SET
                   content     = excluded.content,
                   token_count = excluded.token_count,
                   meta_json   = excluded.meta_json",
                params![
                    doc_id,
                    chunk.ord as i64,
                    chunk.content,
                    chunk.token_count as i64,
                    chunk.meta.to_string(),
                ],
            )
            .map_err(|e| RagError::Database(e.to_string()))?;
        }

        // Rebuild FTS5 index for this document's chunks
        conn.execute_batch("INSERT INTO rag_chunks_fts(rag_chunks_fts) VALUES('rebuild')")
            .map_err(|e| RagError::Database(e.to_string()))?;

        Ok(())
    }

    /// Return the total number of chunks in the index.
    pub fn chunk_count(&self) -> Result<i64, RagError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM rag_chunks", [], |row| row.get(0))
            .map_err(|e| RagError::Database(e.to_string()))
    }

    /// Return the total number of documents in the index.
    pub fn document_count(&self) -> Result<i64, RagError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM rag_documents", [], |row| row.get(0))
            .map_err(|e| RagError::Database(e.to_string()))
    }
}

// ── Retriever impl ────────────────────────────────────────────────────────

#[async_trait]
impl Retriever for RagStore {
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RankedChunk>, RagError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT rc.id, rc.document_id, rd.source_kind, rd.source_path, rd.git_sha,
                        rc.ord, rc.content, bm25(rag_chunks_fts) AS score
                 FROM rag_chunks_fts
                 JOIN rag_chunks   rc ON rc.id  = rag_chunks_fts.rowid
                 JOIN rag_documents rd ON rd.id = rc.document_id
                 WHERE rag_chunks_fts MATCH ?1
                 ORDER BY score
                 LIMIT ?2",
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![query, top_k as i64], |row| {
                Ok(RankedChunk {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    source_kind: row.get(2)?,
                    source_path: row.get(3)?,
                    git_sha: row.get(4)?,
                    ord: row.get(5)?,
                    content: row.get(6)?,
                    // BM25 in SQLite returns negative values (lower = more relevant).
                    // Negate so that higher score = more relevant (conventional).
                    score: -(row.get::<_, f64>(7)?),
                })
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| RagError::Database(e.to_string()))?;

        Ok(rows)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rag_core::Chunk;

    fn fixture_chunks(texts: &[&str]) -> Vec<Chunk> {
        texts
            .iter()
            .enumerate()
            .map(|(i, &text)| Chunk {
                ord: i,
                content: text.to_owned(),
                token_count: text.split_whitespace().count(),
                meta: serde_json::json!({}),
            })
            .collect()
    }

    #[test]
    fn open_in_memory_succeeds() {
        let store = RagStore::in_memory().unwrap();
        assert_eq!(store.chunk_count().unwrap(), 0);
        assert_eq!(store.document_count().unwrap(), 0);
    }

    #[test]
    fn upsert_and_count() {
        let store = RagStore::in_memory().unwrap();
        let chunks = fixture_chunks(&["SQLite is the database.", "FTS5 enables full-text search."]);
        store
            .upsert_document("plan", "plans/adr/ADR-002.md", "abc123", &chunks)
            .unwrap();
        assert_eq!(store.document_count().unwrap(), 1);
        assert_eq!(store.chunk_count().unwrap(), 2);
    }

    #[test]
    fn upsert_is_idempotent() {
        let store = RagStore::in_memory().unwrap();
        let chunks = fixture_chunks(&["First content."]);
        store
            .upsert_document("plan", "plans/test.md", "sha1", &chunks)
            .unwrap();
        // Re-upsert with updated sha and content
        let chunks2 = fixture_chunks(&["Updated content."]);
        store
            .upsert_document("plan", "plans/test.md", "sha2", &chunks2)
            .unwrap();
        assert_eq!(store.document_count().unwrap(), 1);
        assert_eq!(store.chunk_count().unwrap(), 1);
    }

    #[tokio::test]
    async fn fts_retrieve_returns_matching_chunk() {
        let store = RagStore::in_memory().unwrap();
        let chunks = fixture_chunks(&[
            "SQLite is the database engine used by deploy-baba.",
            "Lambda functions run on AWS aarch64.",
            "OpenTofu manages HCL infrastructure.",
        ]);
        store
            .upsert_document("plan", "plans/adr/ADR-002.md", "abc", &chunks)
            .unwrap();

        let results = store.retrieve("SQLite database", 5).await.unwrap();
        assert!(!results.is_empty(), "should find at least one result");
        assert!(
            results[0].content.contains("SQLite"),
            "top result should mention SQLite"
        );
    }

    #[tokio::test]
    async fn retrieve_no_match_returns_empty() {
        let store = RagStore::in_memory().unwrap();
        let chunks = fixture_chunks(&["Rust is a systems language."]);
        store
            .upsert_document("rust", "crates/api-core/src/lib.rs", "sha", &chunks)
            .unwrap();

        let results = store.retrieve("python django flask", 5).await.unwrap();
        assert!(results.is_empty(), "no match should return empty vec");
    }
}
