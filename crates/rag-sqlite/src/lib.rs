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

#[cfg(feature = "llm-bridge")]
pub mod embed_bridge;

use async_trait::async_trait;
use rag_core::{RagError, RankedChunk, Retriever};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
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

/// Statistics returned by [`RagStore::upsert_embeddings`].
pub struct EmbedStats {
    pub embedded: usize,
    pub skipped: usize,
    pub errors: usize,
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

        // Validate that FTS5 xConnect works. If the shadow tables are
        // corrupted or missing, drop + recreate the virtual table.
        if let Err(e) = conn.prepare("SELECT rowid FROM rag_chunks_fts LIMIT 0") {
            tracing::warn!("FTS5 vtable broken ({e}), dropping and recreating");
            let _ = conn.execute_batch(
                "DROP TABLE IF EXISTS rag_chunks_fts;
                 DROP TABLE IF EXISTS rag_chunks_fts_data;
                 DROP TABLE IF EXISTS rag_chunks_fts_idx;
                 DROP TABLE IF EXISTS rag_chunks_fts_content;
                 DROP TABLE IF EXISTS rag_chunks_fts_docsize;
                 DROP TABLE IF EXISTS rag_chunks_fts_config;",
            );
            conn.execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS rag_chunks_fts
                     USING fts5(content, content=rag_chunks, content_rowid=id);
                 INSERT INTO rag_chunks_fts(rag_chunks_fts) VALUES('rebuild');",
            )
            .map_err(|e| RagError::Database(format!("FTS5 recovery failed: {e}")))?;
            tracing::info!("FTS5 vtable recovered successfully");
        }

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

    // ── Embeddings ─────────────────────────────────────────────────────────

    /// Upsert embeddings for all chunks, skipping those whose content hash
    /// has not changed. Requires migration 024 (`rag_embeddings` table).
    pub async fn upsert_embeddings(
        &self,
        embedder: &dyn rag_core::Embedder,
    ) -> Result<EmbedStats, RagError> {
        let chunks_to_embed: Vec<(i64, String)> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, content FROM rag_chunks ORDER BY id")
                .map_err(|e| RagError::Database(e.to_string()))?;

            let rows: Vec<(i64, String)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| RagError::Database(e.to_string()))?
                .filter_map(|r| r.ok())
                .collect();

            // Filter out chunks whose hash matches the stored embedding
            let mut need_embed = Vec::new();
            for (id, content) in rows {
                let hash = content_hash(&content);
                let existing_hash: Option<String> = conn
                    .query_row(
                        "SELECT content_hash FROM rag_embeddings WHERE chunk_id = ?1",
                        params![id],
                        |row| row.get(0),
                    )
                    .ok();
                if existing_hash.as_deref() != Some(&hash) {
                    need_embed.push((id, content));
                }
            }
            need_embed
        };

        let mut stats = EmbedStats {
            embedded: 0,
            skipped: 0,
            errors: 0,
        };

        let total = chunks_to_embed.len();
        if total == 0 {
            let conn = self.conn.lock().unwrap();
            let existing: i64 = conn
                .query_row("SELECT COUNT(*) FROM rag_embeddings", [], |row| row.get(0))
                .unwrap_or(0);
            stats.skipped = existing as usize;
            return Ok(stats);
        }

        // Batch embed (max 100 per call to stay under API limits)
        for batch in chunks_to_embed.chunks(100) {
            let texts: Vec<&str> = batch.iter().map(|(_, c)| c.as_str()).collect();
            match embedder.embed(&texts).await {
                Ok(vectors) => {
                    let conn = self.conn.lock().unwrap();
                    for ((id, content), vec) in batch.iter().zip(vectors.iter()) {
                        let hash = content_hash(content);
                        let blob = embedding_to_blob(vec);
                        conn.execute(
                            "INSERT INTO rag_embeddings (chunk_id, content_hash, embedding, model, dim)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(chunk_id) DO UPDATE SET
                               content_hash = excluded.content_hash,
                               embedding = excluded.embedding,
                               model = excluded.model,
                               dim = excluded.dim,
                               updated_at = datetime('now')",
                            params![*id, hash, blob, embedder.provider_id(), embedder.dim() as i64],
                        )
                        .map_err(|e| RagError::Database(e.to_string()))?;
                        stats.embedded += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!("Embedding batch failed: {e}");
                    stats.errors += batch.len();
                }
            }
        }

        // Count skipped (already-cached embeddings not in the embed set)
        let conn = self.conn.lock().unwrap();
        let total_chunks: i64 = conn
            .query_row("SELECT COUNT(*) FROM rag_chunks", [], |row| row.get(0))
            .unwrap_or(0);
        stats.skipped = total_chunks as usize - stats.embedded - stats.errors;

        Ok(stats)
    }

    /// Retrieve using both FTS5 and ANN (if embeddings exist), merged via RRF.
    pub async fn retrieve_hybrid(
        &self,
        query: &str,
        query_embedding: Option<&[f32]>,
        top_k: usize,
    ) -> Result<Vec<RankedChunk>, RagError> {
        // FTS5 lane
        let fts_results = self.retrieve_filtered(query, top_k * 2, None)?;

        // ANN lane (only if we have a query embedding and embeddings exist)
        let ann_results = if let Some(qe) = query_embedding {
            self.retrieve_ann(qe, top_k * 2)?
        } else {
            Vec::new()
        };

        if ann_results.is_empty() {
            // No ANN results — fall back to pure FTS
            return Ok(fts_results.into_iter().take(top_k).collect());
        }

        Ok(rrf_merge(&fts_results, &ann_results, top_k))
    }

    /// ANN search over stored embeddings using cosine distance.
    fn retrieve_ann(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<RankedChunk>, RagError> {
        let conn = self.conn.lock().unwrap();

        // Check if rag_embeddings has any rows
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM rag_embeddings", [], |row| row.get(0))
            .unwrap_or(0);
        if count == 0 {
            return Ok(Vec::new());
        }

        // Brute-force cosine similarity (no sqlite-vec needed for this approach).
        // For the current corpus size (~2000 chunks), this is fast enough.
        // sqlite-vec ANN can be added later for larger corpora.
        let mut stmt = conn
            .prepare(
                "SELECT re.chunk_id, re.embedding, rc.document_id,
                        rd.source_kind, rd.source_path, rd.git_sha,
                        rc.ord, rc.content
                 FROM rag_embeddings re
                 JOIN rag_chunks rc ON rc.id = re.chunk_id
                 JOIN rag_documents rd ON rd.id = rc.document_id",
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        let rows: Vec<RankedChunk> = stmt
            .query_map([], |row| {
                let chunk_id: i64 = row.get(0)?;
                let blob: Vec<u8> = row.get(1)?;
                let document_id: i64 = row.get(2)?;
                let source_kind: String = row.get(3)?;
                let source_path: String = row.get(4)?;
                let git_sha: String = row.get(5)?;
                let ord: i64 = row.get(6)?;
                let content: String = row.get(7)?;

                let stored = blob_to_embedding(&blob);
                let score = cosine_similarity(query_embedding, &stored);

                Ok(RankedChunk {
                    chunk_id,
                    document_id,
                    source_kind,
                    source_path,
                    git_sha,
                    ord,
                    content,
                    score,
                })
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        // Sort by score descending and take top_k
        let mut sorted = rows;
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(top_k);
        Ok(sorted)
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
static STOP_WORDS: &[&str] = &[
    "how", "does", "what", "is", "the", "in", "a", "an", "of", "to", "for", "and", "or", "this",
    "that", "it", "are", "was", "be", "has", "with", "on", "at", "do", "can", "my", "your", "me",
    "we", "they", "its", "by", "from", "not", "but", "if", "about", "which", "when", "there",
    "tell", "describe", "explain",
];

fn to_fts5_or_query(query: &str) -> String {
    let all_terms: Vec<String> = query
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
    if all_terms.is_empty() {
        return query.to_owned();
    }

    let content_terms: Vec<&str> = all_terms
        .iter()
        .filter(|t| !STOP_WORDS.contains(&t.as_str()))
        .map(|t| t.as_str())
        .collect();

    // If all terms are stop words, use all terms as fallback
    let terms = if content_terms.is_empty() {
        all_terms.iter().map(|t| t.as_str()).collect::<Vec<_>>()
    } else {
        content_terms
    };

    if terms.len() == 1 {
        return terms[0].to_string();
    }

    // Prepend a quoted phrase of all content terms for phrase-boost ranking
    let phrase = format!("\"{}\"", terms.join(" "));
    let or_terms = terms.join(" OR ");
    format!("{phrase} OR {or_terms}")
}

#[async_trait]
impl Retriever for RagStore {
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RankedChunk>, RagError> {
        self.retrieve_filtered(query, top_k, None)
    }
}

// ── Embedding helpers ────────────────────────────────────────────────────

fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn embedding_to_blob(vec: &[f32]) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut norm_a, mut norm_b) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (x, y) in a.iter().zip(b.iter()) {
        let (x, y) = (*x as f64, *y as f64);
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-10 {
        0.0
    } else {
        dot / denom
    }
}

/// Reciprocal Rank Fusion: merge two ranked lists into one.
///
/// RRF_score(d) = sum(1 / (k + rank_i(d))) for each lane i that contains d.
/// k = 60 is the standard constant from the original RRF paper.
fn rrf_merge(
    fts_results: &[RankedChunk],
    ann_results: &[RankedChunk],
    top_k: usize,
) -> Vec<RankedChunk> {
    const K: f64 = 60.0;
    let mut scores: HashMap<i64, (f64, RankedChunk)> = HashMap::new();

    for (rank, chunk) in fts_results.iter().enumerate() {
        let rrf = 1.0 / (K + rank as f64 + 1.0);
        scores
            .entry(chunk.chunk_id)
            .and_modify(|(s, _)| *s += rrf)
            .or_insert((rrf, chunk.clone()));
    }

    for (rank, chunk) in ann_results.iter().enumerate() {
        let rrf = 1.0 / (K + rank as f64 + 1.0);
        scores
            .entry(chunk.chunk_id)
            .and_modify(|(s, _)| *s += rrf)
            .or_insert((rrf, chunk.clone()));
    }

    let mut merged: Vec<RankedChunk> = scores
        .into_values()
        .map(|(score, mut chunk)| {
            chunk.score = score;
            chunk
        })
        .collect();

    merged.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    merged.truncate(top_k);
    merged
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
        // Two content words: phrase boost + OR
        assert_eq!(
            to_fts5_or_query("SQLite database"),
            "\"sqlite database\" OR sqlite OR database"
        );
        // Hyphenated compound: cleaned to single token
        assert_eq!(
            to_fts5_or_query("FTS5 full-text"),
            "\"fts5 fulltext\" OR fts5 OR fulltext"
        );
        // Single content word
        assert_eq!(to_fts5_or_query("single"), "single");
    }

    #[test]
    fn to_fts5_or_query_filters_stop_words() {
        // Stop words filtered, content words preserved
        let result = to_fts5_or_query("How does error handling work?");
        assert!(result.contains("error"));
        assert!(result.contains("handling"));
        assert!(result.contains("work"));
        assert!(!result.contains(" how "));
        assert!(!result.contains(" does "));
    }

    #[test]
    fn to_fts5_or_query_all_stop_words_fallback() {
        // When all words are stop words, use them all
        let result = to_fts5_or_query("what is it");
        assert!(!result.is_empty());
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

    // ── Embedding tests ──────────────────────────────────────────────────

    #[test]
    fn content_hash_deterministic() {
        let h1 = content_hash("hello world");
        let h2 = content_hash("hello world");
        assert_eq!(h1, h2);
        assert_ne!(h1, content_hash("different"));
    }

    #[test]
    fn embedding_blob_roundtrip() {
        let original = vec![0.1_f32, -0.5, 2.78, 0.0];
        let blob = embedding_to_blob(&original);
        let recovered = blob_to_embedding(&blob);
        assert_eq!(original.len(), recovered.len());
        for (a, b) in original.iter().zip(recovered.iter()) {
            assert!((a - b).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn cosine_similarity_identical() {
        let a = vec![1.0_f32, 0.0, 0.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_orthogonal() {
        let a = vec![1.0_f32, 0.0, 0.0];
        let b = vec![0.0_f32, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_opposite() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![-1.0_f32, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn rrf_merge_two_lists() {
        let a = vec![make_ranked(1, "rust", 0.9), make_ranked(2, "plan", 0.8)];
        let b = vec![make_ranked(2, "plan", 0.95), make_ranked(3, "hcl", 0.85)];
        let merged = rrf_merge(&a, &b, 10);

        // chunk 2 appears in both lists → highest RRF score
        assert_eq!(merged[0].chunk_id, 2);
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn rrf_merge_disjoint() {
        let a = vec![make_ranked(1, "a", 1.0)];
        let b = vec![make_ranked(2, "b", 1.0)];
        let merged = rrf_merge(&a, &b, 10);
        assert_eq!(merged.len(), 2);
        // Both should have the same RRF score (each rank-1 in their respective list)
        assert!((merged[0].score - merged[1].score).abs() < 1e-10);
    }

    #[test]
    fn rrf_merge_single_lane() {
        let a = vec![make_ranked(1, "rust", 0.9), make_ranked(2, "plan", 0.8)];
        let merged = rrf_merge(&a, &[], 10);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].chunk_id, 1);
    }

    #[test]
    fn migration_creates_embeddings_table() {
        let store = RagStore::in_memory().unwrap();
        let conn = store.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM rag_embeddings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn upsert_embeddings_stores_vectors() {
        let store = RagStore::in_memory().unwrap();
        store
            .upsert_document(
                "rust",
                "crates/lib.rs",
                "sha1",
                &fixture_chunks(&["Rust code one.", "Rust code two.", "Rust code three."]),
            )
            .unwrap();

        let embedder = StubEmbedder { dim: 4 };
        let stats = store.upsert_embeddings(&embedder).await.unwrap();
        assert_eq!(stats.embedded, 3);
        assert_eq!(stats.errors, 0);

        let conn = store.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM rag_embeddings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn upsert_embeddings_content_hash_skip() {
        let store = RagStore::in_memory().unwrap();
        store
            .upsert_document(
                "rust",
                "lib.rs",
                "sha1",
                &fixture_chunks(&["Same content."]),
            )
            .unwrap();

        let embedder = StubEmbedder { dim: 4 };
        let stats1 = store.upsert_embeddings(&embedder).await.unwrap();
        assert_eq!(stats1.embedded, 1);

        // Second call should skip (content unchanged)
        let stats2 = store.upsert_embeddings(&embedder).await.unwrap();
        assert_eq!(stats2.embedded, 0);
        assert_eq!(stats2.skipped, 1);
    }

    #[tokio::test]
    async fn retrieve_hybrid_fts_only_fallback() {
        let store = RagStore::in_memory().unwrap();
        store
            .upsert_document(
                "plan",
                "test.md",
                "sha1",
                &fixture_chunks(&["SQLite is the database engine."]),
            )
            .unwrap();

        // No embeddings, no query embedding → pure FTS
        let results = store.retrieve_hybrid("SQLite", None, 5).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("SQLite"));
    }

    #[tokio::test]
    async fn retrieve_hybrid_with_embeddings() {
        let store = RagStore::in_memory().unwrap();
        store
            .upsert_document(
                "plan",
                "test.md",
                "sha1",
                &fixture_chunks(&[
                    "SQLite is the database engine.",
                    "Lambda functions run on AWS.",
                ]),
            )
            .unwrap();

        let embedder = StubEmbedder { dim: 4 };
        store.upsert_embeddings(&embedder).await.unwrap();

        let query_emb = vec![0.1_f32; 4];
        let results = store
            .retrieve_hybrid("SQLite", Some(&query_emb), 5)
            .await
            .unwrap();
        assert!(!results.is_empty());
    }

    fn make_ranked(id: i64, kind: &str, score: f64) -> RankedChunk {
        RankedChunk {
            chunk_id: id,
            document_id: 1,
            source_kind: kind.to_string(),
            source_path: format!("test/{id}"),
            git_sha: "abc".to_string(),
            ord: 0,
            content: format!("content {id}"),
            score,
        }
    }

    struct StubEmbedder {
        dim: usize,
    }

    #[async_trait]
    impl rag_core::Embedder for StubEmbedder {
        fn provider_id(&self) -> &'static str {
            "stub"
        }
        fn dim(&self) -> usize {
            self.dim
        }
        async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, RagError> {
            Ok(texts.iter().map(|_| vec![0.1_f32; self.dim]).collect())
        }
    }
}
