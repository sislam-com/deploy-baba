# Execution Roadmap — W-LLM / W-RST / W-RAG / W-GDR

**Last updated:** 2026-04-15
**Status:** Active
**Owner:** project lead

This file sequences the implementation work across four freshly-planned domains
(W-LLM, W-RST, W-RAG, W-GDR) so each phase lands a *demoable, reversible
increment* and no phase starts until its hard dependencies are on disk.

For the *what and why* of each domain, see the module plans — this doc only
covers *when and in what order*.

---

## Design principles

1. **Unblock the graph first.** W-LLM has no dependencies and unblocks both
   W-RST (fully) and W-RAG (P2+). It lands first.
2. **Prove end-to-end before going wide.** The smallest-possible real LLM call
   (W-RST.4.3 — `polish_bio_to_summary`) ships as Phase 2 so we validate the
   entire stack (secrets → adapter → grounding → response) before writing the
   big pipelines.
3. **Free before paid.** W-RAG P1 (FTS-only retrieval) has no LLM cost and no
   secrets dependency — land it in parallel with W-RST's no-LLM items
   (W-RST.4.1 / 4.2). This gives the `just ask` CLI immediate utility.
4. **One phase = one PR = one reversible commit on main.** Every phase ends
   with `just quality` green and `just cache-refresh` run.
5. **Parallelism is an explicit budget.** W-GDR is independent; at most one
   other phase may run concurrently with W-GDR to keep review bandwidth
   manageable.

---

## Dependency graph (compact)

```
                ┌─────────────┐
                │   W-SEC     │  (DONE, needs deploy of anthropic-api-key)
                └──────┬──────┘
                       │
                ┌──────▼──────┐
                │   W-LLM     │  ← foundation; zero upstream deps
                └───┬─────┬───┘
                    │     │
          ┌─────────┘     └─────────┐
          │                         │
    ┌─────▼─────┐            ┌──────▼──────┐
    │  W-RST    │            │   W-RAG     │
    │ (needs    │            │  P1: no LLM │
    │  LLM for  │            │  P2/P3: LLM │
    │  4.3-4.5) │            └─────────────┘
    └───────────┘

    ┌───────────┐
    │  W-GDR    │  ← fully independent, can ship anytime
    └───────────┘
```

Solid arrows = hard dependency. W-RAG P1 crosses no arrows (FTS-only works
without W-LLM).

---

## Phase 0 — Plan hygiene (current PR, zero code)

Before any crate lands. Pre-requisite for everyone to execute against a
single source of truth.

| # | Task | Owner | Exit criteria |
|---|------|-------|--------------|
| 0.1 | Reconcile the two `ADR-015-*.md` files (short-name from feat/rag-impl, long-name from feat/llm-provider) into one | design | `ls plans/adr/ADR-015*` returns exactly one file; INDEX.md ADR table matches |
| 0.2 | Confirm W-RAG, W-LLM, W-RST, W-GDR all registered in `INDEX.md` + `CONVENTIONS.md` | design | `grep -n "W-RAG\|W-LLM\|W-RST\|W-GDR" plans/INDEX.md plans/CONVENTIONS.md` shows all four |
| 0.3 | Merge feat/rag-impl (design docs) → main | review | PR green, ADR-016 referenced from `plans/modules/rag.md` |

**Gate:** nothing in Phase 1 starts until 0.3 lands.

---

## Phase 1 — W-LLM foundation (unblocks everything LLM-touching)

Smallest possible trait crate + first adapter. Ships before any consumer code.

| # | Work item | Task | Notes |
|---|-----------|------|-------|
| 1.1 | W-LLM.4.1 | `crates/llm-core` — `LlmProvider`, `EmbeddingProvider` traits, `LlmRequest/Response`, `GroundingContract`, `LlmError`, `StubLlmProvider` in `testing.rs` | Zero vendor deps. Template: `crates/api-core/` |
| 1.2 | W-LLM.4.2 | `crates/llm-anthropic` — `AnthropicProvider` impl, constructor injection, model constants (`claude-haiku-4-5-20251001`) | Uses `anthropic-sdk` or `reqwest`; check crates.io at impl time |
| 1.3 | W-LLM.4.3 | Workspace plumbing: add to `[workspace.members]`; feature flags in `services/ui/Cargo.toml` (`default = ["llm-anthropic"]`) | No runtime use yet — wiring only |
| 1.4 | W-LLM.4.4 | Per-crate READMEs for both crates (W-DX.3 alignment) | Secret name, feature flag, minimal usage example |
| 1.5 | W-SEC deploy | Run `just infra-apply` + `just secret-put anthropic-api-key $KEY prod` | Prerequisite for Phase 2 — can happen in parallel with 1.1–1.4 |

**Exit criteria:**
- `cargo test -p llm-core -p llm-anthropic` passes with only `StubLlmProvider`
- `just quality` green on the whole workspace
- `services/ui` builds with `--features llm-anthropic` (not yet *using* it)
- `anthropic-api-key` readable via `just secret-get anthropic-api-key prod`

**Reversibility:** remove the two crate directories and feature-flag deps.

---

## Phase 2 — Smallest real LLM call (risk spike, ~1 hour of work)

Validates the entire stack on the smallest imaginable surface before building
the large W-RST and W-RAG pipelines.

| # | Work item | Task | Notes |
|---|-----------|------|-------|
| 2.1 | W-RST.4.3 | Replace `polish_bio_to_summary()` stub in `xtask/src/resume/generate.rs` with a real `LlmProvider::generate()` call; add `--ai` flag to keep offline path | Closes ADR-014 v1 seam; single API call, <100 output tokens |

**Exit criteria:**
- `just resume-generate --ai` produces a real Claude-polished Professional
  Summary from `about_sections.me-bio`
- Offline mode (`just resume-generate`) still works without any API key
- CloudWatch logs show the call completed without exposing the API key

**Reversibility:** revert one xtask file; the trait surface remains.

**Why this before anything bigger:** if the trait abstraction is wrong, we
find out here — on a 1-file change — not 500 lines into the tailor pipeline.

---

## Phase 3 — W-RAG P1 (local CLI, no LLM, no cost)

**Runs in parallel with Phase 4** — no shared files, no shared dependencies.

| # | Work item | Task | Notes |
|---|-----------|------|-------|
| 3.1 | W-RAG.2.1 | `crates/rag-core` — `ChunkSource`, `Retriever`, `PromptAssembler` traits; re-exports `Embedder` from `llm-core` | Pure traits; no I/O |
| 3.2 | W-RAG.2.2 | `crates/rag-sqlite` — `RagStore` with sqlite-vec extension loader (via `Connection::load_extension()`) | Vec table creation deferred until dim known; FTS5 wired immediately |
| 3.3 | W-RAG.2.3 | Migration `015_rag_index.sql` (ADR-010 upsert) | Skill: `add-migration` |
| 3.4 | W-RAG.3.1 | Chunker impls: rust (`syn` AST), hcl (brace-balance), markdown (H2/H3 split), claude-cache (JSON leaf + MD heading) | 4 independent sub-tasks; one test per chunker |
| 3.5 | W-RAG.3.2 | `xtask rag ingest` — walks 4 corpora, chunks, upserts into `rag_documents`/`rag_chunks`/FTS5 | No embedder call in FTS-only mode |
| 3.6 | W-RAG.3.3 | `xtask rag query` — FTS5 BM25 retrieval, prints ranked chunks with path + sha citations | Lexical only at this stage |
| 3.7 | W-RAG.3.4 | Justfile verbs: `rag-index`, `rag-query`, `ask` (ask = alias for query at this stage) | |

**Exit criteria:**
- `just rag-index` populates the 4 corpora in <30s on a clean DB
- `just rag-query "ADR-016"` returns `plans/adr/ADR-016-rag-architecture.md` as top result
- Unit tests cover each chunker against a fixture file
- Zero LLM calls; works offline

**Reversibility:** drop the two crates and migration 015; tables are in a
separate namespace from existing data.

---

## Phase 4 — W-RST no-LLM items (pure Rust, parallel with Phase 3)

These ship independently of Phase 2/3 because they touch different files.

| # | Work item | Task | Notes |
|---|-----------|------|-------|
| 4.1 | W-RST.4.1 | OpenAPI models in `crates/api-openapi/src/models/tailor.rs`: `TailorRequest`, `TailorResponse`, `MatchedBullet`; register in `ALL_MODELS` | ADR-012 SSOT; no LLM |
| 4.2 | W-RST.4.2 | `services/ui/src/tailor/matcher.rs` — pure-Rust token-overlap scorer over `job_details`/`competencies`/`tech_stack` | Deterministic, unit-testable, no secrets |

**Exit criteria:**
- `TailorRequest`/`TailorResponse` round-trip JSON in contract tests
- `matcher.rs` scores a fixture JD against seeded DB rows and returns expected
  ranked bullets

**Reversibility:** delete the `tailor` module and the openapi model file.

---

## Phase 5 — W-RST LLM pipeline (depends on Phase 1 + 2)

With Phase 2 proving the stack, the full pipeline is mechanical.

| # | Work item | Task | Notes |
|---|-----------|------|-------|
| 5.1 | W-RST.4.4 | `tailor/parser.rs` — extract keywords + skill categories from JD via `LlmProvider::generate()` (tool-use where supported) | |
| 5.2 | W-RST.4.5 | `tailor/generator.rs` — grounded rewrite of matched bullets with `GroundingContract.allowed_source_text` | Prompt-layer enforcement (ADR-015) |
| 5.3 | W-RST.4.6 | Refactor `xtask/src/resume/generate.rs` renderer to accept tailored JSON alongside static DB read | Refactor, not rewrite |
| 5.4 | W-RST.4.8 | Migration 016: `tailor_cache` table with cache key `sha256(jd_text ‖ provider_id ‖ model ‖ prompt_version)` | ADR-010 upsert NOT required — this is computed state |
| 5.5 | W-RST.4.7 | `POST /api/admin/tailor` handler — parser → matcher → generator → renderer → S3 upload → presigned URL | Cognito-gated; Lambda Function URL (ADR-003, NOT ADR-009) |
| 5.6 | W-RST.4.9 | `/dashboard/tailor` Askama template — paste form, async submit, download links | ADR-013 dark theme |
| 5.7 | W-RST.4.10 | Cost cap + rate limit enforcement via `llm-core` middleware | Per `plans/cross-cutting/llm-policy.md` |

**Exit criteria:**
- Integration test for `POST /api/admin/tailor` passes using `StubLlmProvider`
- One real end-to-end run against Anthropic returns a tailored resume with
  valid DOCX + PDF S3 URLs
- Rate limit returns 429 after 10 requests/24h for a single Cognito user

**Reversibility:** cached results live in `tailor_cache` — can be dropped
without affecting any other table. Route is Cognito-gated so public traffic
is not affected.

---

## Phase 6 — W-RAG P2 (embeddings + generation)

Upgrades W-RAG from FTS-only retrieval to full RAG. Depends on Phase 1.

| # | Work item | Task | Notes |
|---|-----------|------|-------|
| 6.1 | W-RAG.4.1 | Wire concrete `Embedder` impl from `llm-anthropic` (or `llm-voyage` if Anthropic recommends it for embeddings) | Adds `rag_vec` virtual table with correct dim |
| 6.2 | W-RAG.4.2 | `PromptAssembler` + `llm-core::generate` integration — `just ask` now returns generated answers with citations | |
| 6.3 | W-RAG.5.1 | `xtask deploy` failure hook — on non-zero exit, capture stderr + last N CloudWatch log lines, pass to RAG, print explanation | |

**Exit criteria:**
- `just ask "how does cognito auth work"` returns a cited answer referencing
  `plans/adr/ADR-008-cognito-authentication.md`
- `just deploy dev --force-fail` (synthetic) prints a RAG-generated
  explanation block after the error
- Embedding cache hits verified via content-hash key in `meta_json`

**Reversibility:** feature-flag the embedding path; fall back to FTS-only if
the embedder fails. The failure hook wraps existing logic — one try/catch to
remove.

---

## Phase 7 — W-RAG P3 (public `/api/ask`)

Ships RAG to the Lambda. Depends on Phase 6 and a Lambda zip rebuild.

| # | Work item | Task | Notes |
|---|-----------|------|-------|
| 7.1 | W-RAG.6.2 | Bundle `sqlite-vec` aarch64 SO into Lambda zip (~300 KB); wire `db.rs::init()` to load it | Confirm licensing; add to `infra/build/` manifest |
| 7.2 | W-RAG.6.1 | `services/ui/src/routes/api/ask.rs` — `POST /api/ask` handler | Skill: `add-route` |
| 7.3 | W-RAG.6.3 | Rate-limit (token-bucket per-IP) + `RAG_PUBLIC_ENABLED` env var gate | Ship dark (flag off) to prod first |

**Exit criteria:**
- `curl -X POST https://sislam.com/api/ask -d '{"query":"..."}'` returns
  `{answer, citations[]}` with the flag on
- `just infra-plan prod` shows zero diff beyond the Lambda zip hash
- Rate limit returns 429 after N requests/min from one IP

**Reversibility:** unset `RAG_PUBLIC_ENABLED`. Route still exists but returns
503 — no Lambda redeploy needed to disable.

**`.claude/` corpus reminder:** the Lambda index must exclude the `.claude/`
corpus (it's gitignored and machine-local). Enforced in `xtask rag ingest`
by skipping that chunker when `$AWS_LAMBDA_FUNCTION_NAME` is set at ingest
time, OR by rebuilding the index pre-deploy with an explicit `--public-only`
flag. Pick one in 7.1.

---

## Phase 8 — W-GDR (independent, any time)

Can land before, between, or after any other phase — no shared files.

| # | Work item | Task |
|---|-----------|------|
| 8.1 | W-GDR.4.1–4.3 | Drive MCP plan export/import justfile recipes |
| 8.2 | W-GDR.4.4 | Stop hook quality gate in `.claude/settings.json` |

---

## Parallelism map

```
Phase 0  ─┐
          ▼
Phase 1 ─┬─► (merge) ─┬─► Phase 2  ─► Phase 5 ──┐
         │            │                          │
         │            └─► Phase 3 ─┐             │
         │                         ├─► Phase 6 ─► Phase 7
         └─► Phase 4 ──────────────┘
                                                 │
Phase 8  ─ runs anytime ─────────────────────────┘
```

Rule of thumb: **one Phase-1-descendant + optionally Phase 8 concurrent.**
Never two LLM-touching phases in flight at once — they share the
`llm-policy.md` operational budget and would race on prompt version bumps.

---

## Gating conditions (do NOT advance past a phase without these)

| From | To | Gate |
|------|-----|-----|
| 0 → 1 | design → impl | feat/rag-impl merged to main; ADR-015 deduped |
| 1 → 2 | foundation → spike | `anthropic-api-key` deployed to AWS Secrets Manager |
| 2 → 5 | spike → W-RST pipeline | real Claude call confirmed via `just resume-generate --ai` |
| 3 → 6 | RAG P1 → RAG P2 | `just rag-query` returns correct FTS results across all 4 corpora |
| 5 → 6 | W-RST done → W-RAG gen | W-RST.4.10 rate-limit proven; token budget not breached in 24h test |
| 6 → 7 | RAG local → RAG public | sqlite-vec aarch64 binary size + license confirmed |

---

## Reversibility summary

| Phase | What happens if we need to back out |
|-------|-------------------------------------|
| 1 | Remove 2 crate dirs + feature-flag deps; no runtime impact |
| 2 | Revert 1 xtask file; trait surface remains |
| 3 | Drop 2 crate dirs + migration 015; rag_* tables in isolated namespace |
| 4 | Delete tailor module + openapi model file |
| 5 | Disable Cognito-gated route; tailor_cache is computed state, droppable |
| 6 | Feature-flag embedder; FTS-only fallback path stays live |
| 7 | Unset `RAG_PUBLIC_ENABLED` — no Lambda redeploy needed |

Every phase is reversible without data loss and without touching the
critical path (public portfolio, contact form, resume generation).

---

## Cross-References

- → `plans/modules/llm-core.md` (W-LLM work items)
- → `plans/modules/resume-tailor.md` (W-RST work items)
- → `plans/modules/rag.md` (W-RAG work items)
- → `plans/modules/gdrive-planning.md` (W-GDR work items)
- → `plans/cross-cutting/llm-policy.md` (operational rules shared by all LLM work)
- → ADR-015 (LLM Provider Abstraction + Grounding Contract)
- → ADR-016 (RAG Architecture)
