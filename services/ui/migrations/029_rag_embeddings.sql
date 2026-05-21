-- Embedding storage for ANN retrieval (W-RAG.4.1).
-- Content-hash enables skip-on-reindex when chunk content unchanged.
CREATE TABLE IF NOT EXISTS rag_embeddings (
    chunk_id      INTEGER PRIMARY KEY REFERENCES rag_chunks(id) ON DELETE CASCADE,
    content_hash  TEXT NOT NULL,
    embedding     BLOB NOT NULL,
    model         TEXT NOT NULL DEFAULT 'text-embedding-3-small',
    dim           INTEGER NOT NULL DEFAULT 1536,
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
