# W-RAG: rag-core + rag-sqlite
**Crate(s):** `crates/rag-core/`, `crates/rag-sqlite/` | **Status:** WIP (P1 DONE, P4 Phase 9 DONE, P5 Phases 10–11 DONE, P2/P3 TODO)
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
- **P4 — Extended knowledge corpora:** Index the portfolio's own API spec (OpenAPI JSON) and domain
  data (jobs, competencies, about sections) alongside code/plans. Live-data retrieval at ask-time via
  `PortfolioDataProvider` trait ensures answers reflect dashboard edits without re-indexing.
- **P5 — Agentic portfolio assistant:** Tool-dispatch loop in llm-proxy Lambda (ADR-023); portfolio
  tools (HTTP call-back to UI Lambda API); Claude selects tools based on query intent. Transforms
  `/api/ask` from static RAG Q&A to an agentic assistant that can query live portfolio data.

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
| OpenAPI spec (P4) | JSON path-operation splitter | one chunk per endpoint + per component schema |
| Portfolio data (P4) | entity-to-prose serializer | one chunk per job/competency/about section |

Hard max per chunk: ~800 tokens; oversize blocks fall through to sliding-window with 50% overlap.

### Extended corpora (P4)

**OpenAPI chunker** (`crates/rag-core/src/chunk/openapi.rs`): Parses the generated OpenAPI JSON spec,
emits one chunk per path-operation (e.g., `GET /api/jobs`) with method, description, parameters,
request/response schemas rendered as readable text. Component schemas emit as separate chunks. Meta
carries `{"endpoint": "GET /api/jobs"}`.

**Portfolio data chunker** (`crates/rag-core/src/chunk/portfolio.rs`): Accepts JSON-serialized
portfolio entities from SQLite. Produces one chunk per job (with bullet details inlined), one per
competency (with evidence items), one per about section. Content is readable prose, not raw JSON.

### Filtered retrieval (P4)

`retrieve_filtered(&self, query, top_k, kinds: Option<&[&str]>)` adds a `WHERE rd.source_kind IN
(...)` clause to the FTS query. Default `retrieve()` delegates with `kinds: None` for backward
compatibility. Enables targeted queries (e.g., portfolio-only or API-only retrieval).

### Live-data retrieval (P4)

`PortfolioDataProvider` trait (`crates/rag-core/src/portfolio.rs`) provides live DB queries at
ask-time. `HybridRetriever` wraps FTS `Retriever` + `PortfolioDataProvider` — on query, injects
live DB data as virtual `RankedChunk`s with `source_kind="portfolio"`, `git_sha="live"`. Ensures
answers reflect dashboard edits without re-indexing.

### Agentic tool execution (P5, ADR-023)

The llm-proxy Lambda (non-VPC) executes portfolio tools by calling back to the UI Lambda's public
API endpoints over HTTP. Six tools map to existing endpoints (`list_jobs`, `get_job_details`,
`list_competencies`, `get_competency_details`, `get_resume`, `search_codebase`). The proxy Lambda
knows the API base URL via `PORTFOLIO_API_BASE_URL` env var. Safety: `max_turns=5`,
`token_budget=4000`.

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
| W-RAG.4.1 | Wire `Embedder` impl from `llm-anthropic` (or Voyage) | TODO | Deferred to P2; needs API key provisioned |
| W-RAG.4.2 | `PromptAssembler` + `llm-core::generate` integration | DONE (2026-04-15) | FTS retrieve → DefaultPromptAssembler → AnthropicProvider; both CLI + HTTP |
| W-RAG.5.1 | `xtask deploy` failure hook → RAG explain | TODO | Needs W-RAG.4.2 (DONE) |
| W-RAG.6.1 | `services/ui/src/routes/api/ask.rs` + router wiring | DONE (2026-04-15) | POST /api/ask; Arc<RagStore> in AppState; WAL concurrent reader |
| W-RAG.6.2 | Bundle `sqlite-vec` aarch64 SO into Lambda zip | TODO | P2; confirm binary size (~300 KB) |
| W-RAG.6.3 | Rate-limit + `RAG_PUBLIC_ENABLED` feature flag | DONE (updated 2026-05-01) | `ASK_RATE_LIMIT` env var (default 2/min); IP from `x-forwarded-for` first → `ConnectInfo` → `"unknown"` (Lambda fix — was 127.0.0.1 global bucket); `RAG_PUBLIC_ENABLED=1` gate |
| W-RAG.7.1 | Add `OpenApi` + `Portfolio` variants to `SourceKind` enum + `as_str()`/`Display` | DONE | `crates/rag-core/src/types.rs` — 6 variants |
| W-RAG.7.2 | OpenAPI chunker: parse JSON spec, emit one chunk per path-operation + per component schema | DONE | `crates/rag-core/src/chunk/openapi.rs`; 6 tests |
| W-RAG.7.3 | Portfolio data chunker: JSON-serialized jobs/competencies/about → readable prose chunks | DONE | `crates/rag-core/src/chunk/portfolio.rs`; 5 tests |
| W-RAG.7.4 | Wire new chunkers into `chunk_file()` dispatcher | DONE | `crates/rag-core/src/chunk/mod.rs` — 2 new match arms |
| W-RAG.7.5 | Extend `xtask rag ingest` to emit OpenAPI + portfolio corpora (6 total) | DONE | `xtask/src/rag.rs`; OpenAPI via `full_spec()`, portfolio via SQLite query |
| W-RAG.8.1 | Enhance `DefaultPromptAssembler` for portfolio/API-aware system prompt | DONE | `crates/rag-core/src/lib.rs`; portfolio-aware preamble; 2 tests |
| W-RAG.8.2 | Add `retrieve_filtered()` with optional `source_kind IN (...)` clause | DONE | `crates/rag-sqlite/src/lib.rs`; `source_kind IN` filter; 2 tests |
| W-RAG.9.1 | `PortfolioDataProvider` trait in `rag-core` for live DB queries at ask-time | DONE | `crates/rag-core/src/portfolio.rs`; `serde_json::Value` return |
| W-RAG.9.2 | Implement `PortfolioDataProvider` for `Db` (reuse existing SQL queries) | DONE | `services/ui/src/db.rs`; `#[async_trait] impl PortfolioDataProvider for Db` |
| W-RAG.9.3 | `HybridRetriever` combining FTS + live portfolio virtual chunks | DONE | `crates/rag-core/src/hybrid.rs`; 3 tests; `source_kind="portfolio"`, `git_sha="live"` |
| W-RAG.9.4 | Wire `HybridRetriever` into ask handler replacing raw `RagStore` | DONE | `services/ui/src/routes/api/ask.rs`; `Arc<RagStore>` + `Arc<Db>` → `HybridRetriever` |
| W-RAG.10.1 | Define portfolio tools in llm-proxy (`list_jobs`, `get_job_details`, etc.) | DONE | `services/llm-proxy/src/tools.rs`; 4 tools mapping to portfolio API endpoints |
| W-RAG.10.2 | `PortfolioToolExecutor` implementing `ToolExecutor` via HTTP to UI Lambda | DONE | `services/llm-proxy/src/tool_executor.rs`; `PORTFOLIO_API_BASE_URL` env var |
| W-RAG.10.3 | Wire agent loop into llm-proxy handler (when `tools` non-empty) | DONE | `services/llm-proxy/src/main.rs`; `max_turns=5`, `token_budget=4000` |
| W-RAG.10.4 | Extend `AskProxyRequest`/`AskProxyResponse` with `tools`, `api_base_url`, `tools_used`, `turns` | DONE | `crates/api-openapi/src/models/ask.rs`; backward-compatible `#[serde(default)]` |
| W-RAG.10.5 | Update ask handler for agentic mode (include tool defs in proxy request) | DONE | `services/ui/src/routes/api/ask.rs`; `PORTFOLIO_API_BASE_URL` env var gating |
| W-RAG.10.6 | Evolve system prompt for agentic portfolio assistant mode | DONE | `crates/rag-core/src/lib.rs`; portfolio-aware preamble (done in Phase 9.6) |

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

### P4 (Extended Corpora) tests

- **Unit:** OpenAPI chunker against a fixture JSON spec — assert correct number of chunks (one per
  endpoint + one per schema), assert chunk content contains endpoint method/path/parameters.
- **Unit:** Portfolio chunker against fixture job/competency JSON — assert one chunk per entity,
  assert content includes detail text as readable prose.
- **Unit:** Filtered retrieval — insert portfolio + rust chunks, retrieve with
  `kinds: Some(&["portfolio"])`, assert only portfolio chunks returned.
- **Unit:** Enhanced assembler — given mixed openapi + plan chunks, assert system prompt includes
  portfolio-aware instructions.
- **Integration:** `PortfolioDataProvider` impl against seeded test DB — assert correct JSON output.
- **Integration:** `HybridRetriever` with seeded DB + FTS index — query "what jobs does the owner
  have?", assert both indexed chunks and live virtual chunks appear.
- **Smoke:** `just rag-index && just rag-query "GET /api/jobs"` — expect OpenAPI chunks in results.

### P5 (Agentic) tests

- **Unit:** `run_agent_loop` with `StubLlmProvider` — stub returns `StopReason::ToolUse` on first
  call, `StopReason::EndTurn` on second; assert 2 turns, tool result fed back correctly.
- **Unit:** Max-turns safety — stub always returns ToolUse; assert capped at `max_turns`.
- **Unit:** Token budget — cumulative tracking across turns; assert enforcement.
- **Integration:** `PortfolioToolExecutor` against stub HTTP server — assert correct tool result
  from each portfolio tool.
- **E2E smoke:** `just ask "What AWS experience does the portfolio owner have?"` — assert answer
  cites job details with AWS tech_stack.

## W-RAG.6 Cross-References

- → ADR-002 (SQLite on EFS — rag store co-located in same DB)
- → ADR-004 (dual-mode init — sqlite-vec load path)
- → ADR-005 (zero-cost — no new managed infra)
- → ADR-010 (upsert convention — all rag_* INSERTs)
- → ADR-015 (W-LLM — Embedder + generate traits consumed here)
- → ADR-016 (RAG architecture decision record)
- → ADR-019 (SPA replaces Askama — RAG UI is React, not server-rendered)
- → ADR-023 (Agentic Tool-Dispatch Architecture — P5 agent loop + portfolio tools)
- → `cross-cutting/llm-policy.md` (grounding contract, citation format, agentic cost model)
- ← W-LLM (ToolExecutor trait + run_agent_loop in llm-core)
- ← W-UI (P3: `/api/ask` route lives in ui-service)
- ← W-XT (xtask rag subcommands)
