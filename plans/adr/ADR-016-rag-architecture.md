# ADR-016: RAG Architecture

**Date:** 2026-04-15
**Status:** Proposed
**Affected modules:** W-RAG, W-UI, W-LLM

## Context

The project needs the ability to retrieve and explain its own artifacts — Rust source, OpenTofu HCL,
plan modules, ADRs, drift logs, and the `.claude/` agent cache. Three concrete use-cases drive this:

1. **Internal dev CLI** (`just ask`) — developer queries the repo while working locally.
2. **Deploy-failure diagnosis** — on `just deploy` failure, automatically retrieve relevant
   code/infra context and generate a root-cause explanation.
3. **Public portfolio demo** (`POST /api/ask`) — site visitors can query the codebase live.

Constraints:
- Must not introduce new managed infrastructure (ADR-005 zero-cost).
- Persistence must stay in the existing EFS-mounted SQLite DB (ADR-002).
- All generation and embedding calls must go through `llm-core` (ADR-015).
- The `.claude/` cache is gitignored and machine-local — it must not be indexed in the Lambda.
- The Lambda zip size budget is limited; any new native dependency must be justified.

## Decision

> We will build a hybrid retrieval system (vector ANN + BM25 keyword, merged by Reciprocal Rank
> Fusion) stored entirely in the existing SQLite database using the `sqlite-vec` extension and
> SQLite's built-in FTS5. All embedding and generation flows through `llm-core` (ADR-015).

Key choices:

- **`sqlite-vec`** is loaded as a runtime `dlopen` extension via `rusqlite::Connection::load_extension()`.
  No re-linking of SQLite. The extension ships as a single ~300 KB shared object bundled into the
  Lambda zip for aarch64/AL2023 and discovered from `$SQLITE_VEC_PATH` locally.
- **FTS5** (already in bundled SQLite) provides BM25 keyword retrieval as a free second retrieval
  lane. This means P1 (local CLI) works offline without an embedding provider.
- **Reciprocal Rank Fusion (RRF)** merges ANN and BM25 scores without requiring a trained re-ranker.
- **Per-corpus chunkers:** Rust source uses `syn` AST walk (fn/impl/doc chunks); HCL uses
  brace-balance regex (resource/variable blocks); Markdown uses H2/H3 heading split (sections);
  `.claude/` JSON uses leaf-value + MD heading split (local-CLI only).
- **Content-hash embedding cache** in `meta_json` eliminates redundant embedding API calls on
  re-index when chunk content is unchanged.
- **Grounding contract** (shared with ADR-015 / `cross-cutting/llm-policy.md`): retrieved chunks
  are wrapped as `<source kind="..." path="..." sha="...">…</source>` and the system prompt
  requires the model to cite all claims.

## Consequences

### Positive
- Zero new managed infra — the RAG store lives in the same EFS SQLite file as all other data.
- FTS-only mode works immediately (P1 CLI) before an embedding API key is provisioned.
- Content-hash caching makes re-indexing cheap after incremental code changes.
- Phased delivery: P1 (local CLI) → P2 (deploy-failure hook) → P3 (public endpoint) can each ship
  independently.

### Negative / Trade-offs
- `sqlite-vec` aarch64 binary must be bundled in the Lambda zip; adds ~300 KB and a build step.
- `dlopen` extension loading requires `unsafe` code and is sensitive to path configuration.
- ANN quality degrades at very small corpus sizes — pure BM25 is often better below ~1000 chunks.
  RRF mitigates but doesn't eliminate this.
- Chunking quality directly affects retrieval quality; chunker bugs are hard to catch without eval.

### Neutral
- The `rag_vec` virtual table dimension `N` is fixed at index-init time to the chosen embedding
  model's output dimension. Changing embedding models requires dropping and rebuilding the vec table.
- `.claude/` cache corpus is explicitly excluded from the Lambda (P3) index — local CLI gets a richer
  retrieval context than the public endpoint.

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Qdrant (managed or Docker) | Violates ADR-005; new managed infra; Docker not available in Lambda |
| Pinecone / Weaviate | Violates ADR-005 zero-cost principle |
| Lexical-only (tantivy / BM25) | Loses semantic recall; no embedding story for future W-RST grounding |
| Separate LLM path in rag-core (bypass llm-core) | Creates divergence from ADR-015; duplicates API key management |
| pgvector / PostgreSQL | Violates ADR-002 (no PostgreSQL); heavy for a serverless setup |

## Cross-References

- → ADR-002 (SQLite on EFS — rag tables co-located in same DB)
- → ADR-004 (dual-mode init — sqlite-vec load path differs local vs Lambda)
- → ADR-005 (zero-cost philosophy — no new managed infra)
- → ADR-010 (upsert convention — all `INSERT` into `rag_*` tables use `ON CONFLICT DO UPDATE`)
- → ADR-015 (W-LLM — `Embedder` + `Completer` traits consumed by rag-core)
- → W-RAG (implementing module)
- → W-UI (P3: `/api/ask` route)
- → `cross-cutting/llm-policy.md` (grounding contract and citation format)
