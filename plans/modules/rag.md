# W-RAG: rag-core + rag-sqlite
**Crate(s):** `crates/rag-core/`, `crates/rag-sqlite/` | **Status:** WIP (P1 DONE, P2/P3 TODO)
**Coverage floor:** 70% | **Depends on:** W-LLM, W-UI, W-OTF | **Depended on by:** (none yet)

## W-RAG.1 Purpose

Retrieval-Augmented Generation (RAG) over the deploy-baba repository's own artifacts: Rust source,
OpenTofu HCL, plan modules/ADRs/drift logs, and the `.claude/` agent cache. The system is phased:

- **P1 — Internal dev assistant:** `just rag-index` + `just ask` CLI for the developer working on
  the repo locally. No Lambda involvement. FTS-only retrieval works offline before embeddings are wired.
- **P2 — Deploy-failure diagnosis:** on `just deploy` failure, auto-retrieve relevant code/infra
  snippets and produce a root-cause explanation via `llm-core::generate`.
- **P3 — Public `/api/ask` demo:** live endpoint on sislam.com lets visitors query the repo.
  Rate-limited; gated on `RAG_PUBLIC_ENABLED` env var.

The `.claude/` cache corpus (L1 fast-path) is scoped to local CLI only — it is gitignored and must
not be bundled into the Lambda.

## W-RAG.2 Public API Surface

### `crates/rag-core` traits

```rust
// Splits a source artifact into indexable chunks
pub trait ChunkSource {
    fn chunks(&self, path: &Path, content: &str) -> Vec<Chunk>;
}

// Embeds text into a dense vector (wired to llm-anthropic or Voyage via W-LLM)
pub trait Embedder: Send + Sync {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dim(&self) -> usize;
}

// Hybrid retrieval: ANN (sqlite-vec) + BM25 (FTS5), merged by RRF
pub trait Retriever {
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RankedChunk>>;
}

// Assembles a grounded prompt from retrieved chunks (grounding contract: ADR-016)
pub trait PromptAssembler {
    fn assemble(&self, query: &str, chunks: &[RankedChunk]) -> PromptBundle;
}
```

### `crates/rag-sqlite`

- `RagStore` — implements `Retriever`; wraps `rusqlite::Connection` with `sqlite-vec` loaded via
  `Connection::load_extension()`.
- Schema exposed as a typed migration string consumed by `services/ui/db.rs`.

### xtask subcommands (via `just`)

```
just rag-index              # walk all 4 corpora, chunk, upsert, optionally embed
just rag-query "..."        # hybrid retrieve, print ranked chunks with paths + sha
just ask "..."              # retrieve + generate via llm-core (P2; requires W-LLM)
```

### HTTP (P3)

```
POST /api/ask
  Body:  { "query": "..." }
  200:   { "answer": "...", "citations": [{ "kind": "...", "path": "...", "sha": "..." }] }
  429:   rate limited
```

## W-RAG.3 Implementation Notes

### Retrieval backend — SQLite + sqlite-vec (ADR-016)

`sqlite-vec` is loaded as a runtime extension — no recompile of `rusqlite`. It adds a `vec0` virtual
table for approximate nearest-neighbour search over float embeddings. FTS5 (already available in the
bundled SQLite) provides BM25 keyword retrieval. Scores are fused via Reciprocal Rank Fusion (RRF)
before returning the top-k results.

### Schema (migration `015_rag_index.sql`, ADR-010 upsert)

```sql
CREATE TABLE IF NOT EXISTS rag_documents (
    id          INTEGER PRIMARY KEY,
    source_kind TEXT NOT NULL,   -- "rust" | "hcl" | "plan" | "cache"
    source_path TEXT NOT NULL,
    git_sha     TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE(source_kind, source_path)
);

CREATE TABLE IF NOT EXISTS rag_chunks (
    id           INTEGER PRIMARY KEY,
    document_id  INTEGER NOT NULL REFERENCES rag_documents(id) ON DELETE CASCADE,
    ord          INTEGER NOT NULL,
    content      TEXT NOT NULL,
    token_count  INTEGER NOT NULL,
    meta_json    TEXT NOT NULL DEFAULT '{}',
    UNIQUE(document_id, ord)
);

CREATE VIRTUAL TABLE IF NOT EXISTS rag_chunks_fts
    USING fts5(content, content=rag_chunks, content_rowid=id);

-- vec0 table (N = embedding dimension, filled by rag-sqlite at init)
-- CREATE VIRTUAL TABLE rag_vec USING vec0(embedding FLOAT[N]);
-- N is determined at runtime when an Embedder is present; skipped in FTS-only mode.
```

All `INSERT` statements use `ON CONFLICT DO UPDATE` (ADR-010).

### Chunkers (per corpus)

| Corpus | Chunker | Chunk unit |
|--------|---------|------------|
| Rust (`crates/**/*.rs`, `services/**/*.rs`) | `syn`-based AST walk | fn / impl block / module doc |
| Infra HCL (`infra/*.tf`) | brace-balance regex splitter | single resource or variable block |
| Plans/ADRs/drift (`plans/**/*.md`) | markdown H2/H3 heading split | section |
| `.claude/` cache + memory | JSON-leaf + MD heading split | cache entry (local CLI only) |

Hard max per chunk: ~800 tokens; oversize blocks fall through to sliding-window with 50% overlap.

### Embedding caching

Chunks cache their content hash in `meta_json.content_hash`. On re-index, chunks whose hash matches
the stored value skip the `Embedder` call — re-indexing is free when content is unchanged.

### Dual-mode sqlite-vec load (ADR-004)

The `sqlite-vec` shared object path differs between local dev and Lambda:

- **Local:** `$SQLITE_VEC_PATH` env var, or auto-discovered in `~/.cargo/bin/` (populated by a
  `just setup` step to be added in impl).
- **Lambda (aarch64/AL2023):** bundled as a layer at `opt/lib/libsqlite_vec.so`. Lambda zip build
  step adds it alongside the Rust binary.

The existing `main.rs` dual-mode detection (ADR-004) drives which path is used.

### Grounding contract (ADR-016 / `cross-cutting/llm-policy.md`)

Each retrieved chunk is wrapped as:
```
<source kind="rust" path="crates/api-core/src/lib.rs" sha="abc123">
fn example() { ... }
</source>
```
The system prompt requires the model to cite all claims with `[source N]` markers. The assembler
injects this contract via `PromptBundle.system_prompt`.

## W-RAG.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-RAG.1.1 | Land design PR (this plan module + ADR-016 + index/conventions updates) | DONE | feat/rag-impl |
| W-RAG.1.2 | Confirm W-LLM module + ADR-015 exist on disk; author them if not | DONE | W-LLM shipped (feat/llm-core); ADR-015 in plans/adr/ |
| W-RAG.2.1 | Create `crates/rag-core` with trait surface | DONE | ChunkSource→chunk_file, Embedder, Retriever, PromptAssembler, DefaultPromptAssembler; 21 unit tests pass |
| W-RAG.2.2 | Create `crates/rag-sqlite` + FTS5 retrieval | DONE | RagStore (Mutex<Connection>, upsert_document, FTS5 BM25 retrieve); 5 tests pass. sqlite-vec deferred to P2 |
| W-RAG.2.3 | Migration `016_rag_index.sql` | DONE | Added to services/ui/migrations/ + db.rs MIGRATIONS array |
| W-RAG.3.1 | Chunker impls: rust (regex), hcl (brace-balance), markdown (H2/H3), claude-cache (JSON-leaf+MD) | DONE | 4 chunkers, each with 4–5 unit tests; oversize sliding-window split |
| W-RAG.3.2 | `xtask rag ingest` — walk, chunk, upsert | DONE | `cargo xtask rag ingest`; skips `.` dirs + target/; best-effort git SHA |
| W-RAG.3.3 | `xtask rag query` — FTS5 retrieve, print citations | DONE | `cargo xtask rag query "..."` prints ranked chunks with path+score+preview |
| W-RAG.3.4 | Justfile verbs: `rag-index`, `rag-query`, `rag-index-full` | DONE | `just rag-index`, `just rag-index-full`, `just rag-query QUERY` |
| W-RAG.4.1 | Wire `Embedder` impl from `llm-anthropic` (or Voyage) | BLOCKED | W-LLM shipped |
| W-RAG.4.2 | `PromptAssembler` + `llm-core::generate` integration | BLOCKED | W-LLM shipped |
| W-RAG.5.1 | `xtask deploy` failure hook → RAG explain | TODO | Needs W-RAG.4.2 |
| W-RAG.6.1 | `services/ui/src/routes/api/ask.rs` + router wiring | TODO | skill: add-route; needs W-RAG.4.2 |
| W-RAG.6.2 | Bundle `sqlite-vec` aarch64 SO into Lambda zip | TODO | Confirm binary size (~300 KB) |
| W-RAG.6.3 | Rate-limit + `RAG_PUBLIC_ENABLED` feature flag | TODO | Needs W-RAG.6.1 |

## W-RAG.5 Test Strategy

- **Unit:** one test per `ChunkSource` impl against a fixture file; assert non-empty chunks, max
  token count, correct metadata fields. Lives in `crates/rag-core/src/chunk/`.
- **Integration:** spin up a temp SQLite DB with `rag-sqlite`, ingest a fixture slice of the repo
  (one Rust file, one `.tf`, one `.md`), run a lexical query, assert the correct chunk is ranked
  first. Lives in `crates/rag-sqlite/tests/`.
- **Smoke (P1 CI gate):** `just rag-index && just rag-query "ADR-016"` — expect at least one result
  citing `plans/adr/ADR-016-rag-architecture.md`.
- **Coverage floor:** 70% (relaxed from the library 80% floor because P3 Lambda path is harder to
  cover in unit tests).

## W-RAG.6 Cross-References

- → ADR-002 (SQLite on EFS — rag store co-located in same DB)
- → ADR-004 (dual-mode init — sqlite-vec load path)
- → ADR-005 (zero-cost — no new managed infra)
- → ADR-010 (upsert convention — all rag_* INSERTs)
- → ADR-015 (W-LLM — Embedder + generate traits consumed here)
- → ADR-016 (RAG architecture decision record)
- → `cross-cutting/llm-policy.md` (grounding contract, citation format)
- ← W-UI (P3: `/api/ask` route lives in ui-service)
- ← W-XT (xtask rag subcommands)
