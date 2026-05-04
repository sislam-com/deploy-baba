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

    /// Retrieve chunks filtered by optional `source_kind` values.
    ///
    /// When `kinds` is `None`, retrieves from all source kinds (same as `retrieve`).
    /// When `kinds` is `Some(&["portfolio", "openapi"])`, restricts results to those kinds.
    pub fn retrieve_filtered(
        &self,
        query: &str,
        top_k: usize,
        kinds: Option<&[&str]>,
    ) -> Result<Vec<RankedChunk>, RagError> {
        let conn = self.conn.lock().unwrap();
        let fts_query = to_fts5_or_query(query);

        let (sql, bind_count) = match kinds {
            None => (
                "SELECT rc.id, rc.document_id, rd.source_kind, rd.source_path, rd.git_sha,
                        rc.ord, rc.content, bm25(rag_chunks_fts) AS score
                 FROM rag_chunks_fts
                 JOIN rag_chunks   rc ON rc.id  = rag_chunks_fts.rowid
                 JOIN rag_documents rd ON rd.id = rc.document_id
                 WHERE rag_chunks_fts MATCH ?1
                 ORDER BY score
                 LIMIT ?2"
                    .to_string(),
                0,
            ),
            Some([]) => return Ok(Vec::new()),
            Some(ks) => {
                let placeholders: Vec<String> =
                    (0..ks.len()).map(|i| format!("?{}", i + 3)).collect();
                let in_clause = placeholders.join(", ");
                (
                    format!(
                        "SELECT rc.id, rc.document_id, rd.source_kind, rd.source_path, rd.git_sha,
                                rc.ord, rc.content, bm25(rag_chunks_fts) AS score
                         FROM rag_chunks_fts
                         JOIN rag_chunks   rc ON rc.id  = rag_chunks_fts.rowid
                         JOIN rag_documents rd ON rd.id = rc.document_id
                         WHERE rag_chunks_fts MATCH ?1
                           AND rd.source_kind IN ({in_clause})
                         ORDER BY score
                         LIMIT ?2"
                    ),
                    ks.len(),
                )
            }
        };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| RagError::Database(e.to_string()))?;

        let rows = if bind_count == 0 {
            stmt.query_map(params![fts_query, top_k as i64], |row| {
                Ok(RankedChunk {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    source_kind: row.get(2)?,
                    source_path: row.get(3)?,
                    git_sha: row.get(4)?,
                    ord: row.get(5)?,
                    content: row.get(6)?,
                    score: -(row.get::<_, f64>(7)?),
                })
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| RagError::Database(e.to_string()))?
        } else {
            let kinds_slice = kinds.unwrap();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            param_values.push(Box::new(fts_query.clone()));
            param_values.push(Box::new(top_k as i64));
            for k in kinds_slice {
                param_values.push(Box::new(k.to_string()));
            }
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|p| p.as_ref()).collect();

            stmt.query_map(param_refs.as_slice(), |row| {
                Ok(RankedChunk {
                    chunk_id: row.get(0)?,
                    document_id: row.get(1)?,
                    source_kind: row.get(2)?,
                    source_path: row.get(3)?,
                    git_sha: row.get(4)?,
                    ord: row.get(5)?,
                    content: row.get(6)?,
                    score: -(row.get::<_, f64>(7)?),
                })
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| RagError::Database(e.to_string()))?
        };

        Ok(rows)
    }
}

// ── Retriever impl ────────────────────────────────────────────────────────

/// Convert a natural-language query string into an FTS5 OR expression.
///
/// FTS5 treats raw multi-word input as an implicit AND — every term must appear
/// in a row for it to match. For short natural-language queries this is too
/// strict and returns zero results when any term is absent. Joining with `OR`
/// lets BM25 rank partial matches so even queries that mix known and unknown
/// terms surface the best available chunks.
///
/// Terms are lowercased and stripped to alphanumeric characters to avoid
/// FTS5 syntax errors from punctuation in the user query.
fn to_fts5_or_query(query: &str) -> String {
    let terms: Vec<String> = query
        .split_whitespace()
        .filter_map(|t| {
            let clean: String = t
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if clean.is_empty() {
                None
            } else {
                Some(clean)
            }
        })
        .collect();
    if terms.is_empty() {
        // Fallback: use the raw query and let SQLite return an error if malformed
        return query.to_owned();
    }
    terms.join(" OR ")
}

#[async_trait]
impl Retriever for RagStore {
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RankedChunk>, RagError> {
        self.retrieve_filtered(query, top_k, None)
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

    #[test]
    fn to_fts5_or_query_joins_with_or() {
        assert_eq!(to_fts5_or_query("SQLite database"), "sqlite OR database");
        assert_eq!(to_fts5_or_query("FTS5 full-text"), "fts5 OR fulltext");
        assert_eq!(to_fts5_or_query("single"), "single");
    }

    #[tokio::test]
    async fn partial_match_returns_results_via_or_query() {
        // "retrieval" is not in the corpus but "sqlite" is — OR query should still match.
        let store = RagStore::in_memory().unwrap();
        let chunks = fixture_chunks(&["SQLite is the database engine used by deploy-baba."]);
        store
            .upsert_document("plan", "plans/adr/ADR-002.md", "abc", &chunks)
            .unwrap();

        let results = store
            .retrieve("SQLite retrieval nonexistentterm", 5)
            .await
            .unwrap();
        assert!(
            !results.is_empty(),
            "OR query should match on 'sqlite' even when other terms miss"
        );
    }

    #[test]
    fn retrieve_filtered_by_kind() {
        let store = RagStore::in_memory().unwrap();
        store
            .upsert_document(
                "rust",
                "crates/lib.rs",
                "sha1",
                &fixture_chunks(&["Rust systems programming language."]),
            )
            .unwrap();
        store
            .upsert_document(
                "portfolio",
                "portfolio/jobs.json",
                "sha1",
                &fixture_chunks(&["Job: Senior Rust Engineer at Acme Corp."]),
            )
            .unwrap();

        let all = store.retrieve_filtered("Rust", 10, None).unwrap();
        assert_eq!(all.len(), 2, "unfiltered should return both");

        let portfolio_only = store
            .retrieve_filtered("Rust", 10, Some(&["portfolio"]))
            .unwrap();
        assert_eq!(portfolio_only.len(), 1);
        assert_eq!(portfolio_only[0].source_kind, "portfolio");

        let rust_only = store
            .retrieve_filtered("Rust", 10, Some(&["rust"]))
            .unwrap();
        assert_eq!(rust_only.len(), 1);
        assert_eq!(rust_only[0].source_kind, "rust");
    }

    #[test]
    fn retrieve_filtered_empty_kinds_returns_empty() {
        let store = RagStore::in_memory().unwrap();
        store
            .upsert_document("plan", "test.md", "sha", &fixture_chunks(&["content"]))
            .unwrap();
        let results = store.retrieve_filtered("content", 10, Some(&[])).unwrap();
        assert!(results.is_empty());
    }
}
