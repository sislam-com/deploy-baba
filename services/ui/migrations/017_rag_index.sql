-- RAG index schema (ADR-016, ADR-010 upsert convention)
-- Applied by rag-sqlite RagStore::new() and by services/ui migration 016_rag_index.sql.

CREATE TABLE IF NOT EXISTS rag_documents (
    id          INTEGER PRIMARY KEY,
    source_kind TEXT    NOT NULL,   -- "rust" | "hcl" | "plan" | "cache"
    source_path TEXT    NOT NULL,
    git_sha     TEXT    NOT NULL,
    updated_at  TEXT    NOT NULL,
    UNIQUE(source_kind, source_path)
);

CREATE TABLE IF NOT EXISTS rag_chunks (
    id           INTEGER PRIMARY KEY,
    document_id  INTEGER NOT NULL REFERENCES rag_documents(id) ON DELETE CASCADE,
    ord          INTEGER NOT NULL,
    content      TEXT    NOT NULL,
    token_count  INTEGER NOT NULL,
    meta_json    TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(document_id, ord)
);

-- FTS5 content table backed by rag_chunks — BM25 retrieval.
-- Trigger-based sync is handled by explicit rebuild calls in RagStore.
CREATE VIRTUAL TABLE IF NOT EXISTS rag_chunks_fts
    USING fts5(content, content=rag_chunks, content_rowid=id);
