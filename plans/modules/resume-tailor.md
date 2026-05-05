# W-RST: resume-tailor
**Path:** `services/ui/src/tailor/`, `crates/api-openapi/src/models/tailor.rs`, `services/ui/migrations/016` | **Status:** TODO
**Coverage floor:** N/A (binary) | **Depends on:** W-RSM, W-LLM, W-SEC (deployed), W-APIO, W-UI | **Depended on by:** —

---

## W-RST.1 Purpose

Interactive job-description → tailored-resume pipeline. A user (the author,
via the admin dashboard) pastes a job description; the system extracts
keywords, matches them against existing SQLite resume data, uses an LLM to
rewrite matched bullets in a grounded manner, renders the result to DOCX +
PDF, and stores the outputs in S3 for download.

This is **distinct** from W-RSM (the static resume generator). W-RSM
generates a full chronological/functional resume from the DB on demand.
W-RST generates a targeted, JD-specific variant — reordering and
rephrasing, never inventing. W-RSM remains the static-generation authority;
W-RST is its interactive tailoring complement.

---

## W-RST.2 Public API Surface

```
POST /api/admin/tailor          (Cognito-gated via require_auth)
  Body:    { "job_description": "<string>" }
  Returns: TailorResponse { summary, ordered_bullets, matched_competencies,
                             docx_url, pdf_url }

GET  /dashboard/tailor          Admin UI — paste form, submit, history list
```

All request/response types live in `crates/api-openapi/src/models/tailor.rs`
and register in `ALL_MODELS` per ADR-012.

### OpenAPI model shapes (not binding until W-RST.4.1)

```rust
// crates/api-openapi/src/models/tailor.rs

pub struct TailorRequest {
    pub job_description: String,
}

pub struct TailorResponse {
    pub summary: String,
    pub ordered_bullets: Vec<MatchedBullet>,
    pub matched_competencies: Vec<String>,  // competency slugs
    pub docx_url: String,                   // S3 presigned URL
    pub pdf_url: String,                    // S3 presigned URL
    pub cache_hit: bool,
}

pub struct MatchedBullet {
    pub job_slug: String,
    pub detail_text: String,               // original source text
    pub rewritten_text: String,            // LLM-grounded rewrite
    pub score: f32,                        // keyword overlap score
    pub category: Option<String>,          // achievement | responsibility | sub-engagement
}
```

---

## W-RST.3 Implementation Notes

### Architecture

One Lambda, four internal handler modules under
`services/ui/src/tailor/`:

```
services/ui/src/tailor/
├── mod.rs          # orchestrates parser → matcher → generator → renderer
├── parser.rs       # LLM call: extract keywords + skill categories from JD
├── matcher.rs      # pure Rust: keyword / token-overlap scorer over DB rows
├── generator.rs    # LLM call: grounded rewrite of matched bullets
└── renderer.rs     # reuse xtask DOCX/PDF path; accept tailored JSON
```

Four separate Lambdas were considered and rejected (four cold starts +
four IAM roles for a single-user admin tool — not justified). If latency
ever demands extraction, the internal module split makes it mechanical.

All LLM calls (`parser.rs`, `generator.rs`) go through the `LlmProvider`
trait from `crates/llm-core`. Active provider selected at compile time via
cargo feature flag (`llm-anthropic` by default). The modules program
against the trait — never against `AnthropicProvider` directly.

### Routing

**Lambda Function URL** (ADR-003), NOT API Gateway (ADR-009). The
full pipeline (JD parse + keyword search + Claude rewrite + DOCX/PDF render)
can exceed API Gateway's 29 s cap. Function URL allows up to 15 min.
This is a settled decision; do not revisit without benchmarking real latency.

### Matching (pure Rust, no LLM)

Signal sources:
- `job_details.detail_text` — primary match surface
- `jobs.tech_stack` (comma-separated) — tech keyword signals
- `competencies.slug` + `competencies.description` — category signals
- `competency_evidence.highlight_text` — supplemental overrides

Scoring: weighted overlap of normalized (lowercased, stop-word-stripped)
tokens from the JD against each candidate row. Returns ranked
`Vec<MatchedBullet>` — top-N fed to the generator. Deterministic,
unit-testable, ships without any LLM key (proves the pipeline backbone
independently).

### Grounded Generation (via W-LLM)

`generator.rs` invokes `LlmProvider::generate()` with a
`GroundingContract` that carries `allowed_source_text` (exact bullet text
of the matched rows). The grounding contract is enforced in `llm-core`'s
prompt-assembly layer — the generator may only rephrase or reorder the
whitelisted bullets, never invent skills or add roles not present in the
source. See ADR-015 for the full rationale.

### Cache

Table `tailor_cache` (migration 016). Cache key:
`sha256(jd_text || provider_id || model || prompt_version)`.

Both `provider_id` AND `prompt_version` are in the key — swapping
providers, updating models, or bumping prompt semantics all correctly
invalidate entries without requiring manual cache purges.

### Renderer Reuse

`renderer.rs` reuses the existing DOCX/PDF rendering path in
`xtask/src/resume/generate.rs`. The path currently accepts a static
DB read — W-RST.4.6 refactors it to also accept a pre-assembled tailored
JSON struct, keeping the rendering logic in one place.

### Claude SDK usage

Reference the workspace-level **`claude-api` skill** at implementation
time for Anthropic SDK setup, prompt caching, streaming, and tool-use
patterns.

### Dependency on W-SEC deployment

Items W-RST.4.3, 4.4, and 4.5 require the real Anthropic API key in AWS
Secrets Manager (`deploy-baba/prod/anthropic-api-key`). They are
**BLOCKED-on-deploy** until:
1. W-SEC infra-apply has run (`just infra-apply`)
2. `just secret-put anthropic-api-key <key> <profile>` has executed

W-RST.4.1 and W-RST.4.2 have no LLM dependency and can ship first.

---

## W-RST.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-RST.4.1 | OpenAPI models in `crates/api-openapi/src/models/tailor.rs`: `TailorRequest`, `TailorResponse`, `MatchedBullet`. Register in `ALL_MODELS`. | DONE | ADR-012 SSOT. Registered in ALL_MODELS + AdminApiDoc schemas + apidoc.rs import. 50 api-openapi tests pass. |
| W-RST.4.2 | `services/ui/src/tailor/matcher.rs` — pure-Rust token-overlap scorer over job_details / competencies / tech_stack. Deterministic, unit-testable, no LLM, no secrets. Ships the pipeline backbone before any LLM adapter exists. | DONE | `rank_bullets()` + `tokenise()` with stop-word removal; 8 unit tests pass incl. in-memory DB test |
| W-RST.4.3 | Replace `polish_bio_to_summary()` stub in `xtask/src/resume/generate.rs` with a real `LlmProvider::generate()` call through W-LLM. Smallest-possible LLM call; validates trait + adapter + secrets wiring end-to-end before the big generator lands. Closes ADR-014 "v1 seam". Consider `--ai` flag to keep the offline path available. | DONE | `--ai` flag added; `polish_bio_to_summary_ai()` calls `AnthropicProvider`; static fallback retained; `just resume-generate --ai` reads `ANTHROPIC_API_KEY` from env |
| W-RST.4.4 | `services/ui/src/tailor/parser.rs` — `LlmProvider::generate()` call to extract keyword list + implied skill categories from JD. Uses tool-use / structured output where the adapter supports it. | TODO | BLOCKED on W-LLM.4.1/4.2 + W-SEC deployed |
| W-RST.4.5 | `services/ui/src/tailor/generator.rs` — grounded rewrite of matched bullets via `LlmProvider::generate()` with `GroundingContract`. Enforces "no invention" at prompt layer (ADR-015). Streams response if adapter supports. | TODO | BLOCKED on 4.3 + 4.4 |
| W-RST.4.6 | `services/ui/src/tailor/renderer.rs` — refactor `xtask/src/resume/generate.rs` to accept tailored JSON (alongside the existing static DB-read path). Reuse DOCX/PDF rendering logic unchanged. Refactor, not rewrite. | TODO | |
| W-RST.4.7 | `POST /api/admin/tailor` handler in `services/ui/src/routes/api/admin.rs` — orchestrate parser → matcher → generator → renderer → S3 upload → presigned URL response. | TODO | |
| W-RST.4.8 | Migration 016: `tailor_cache(cache_key TEXT PK, jd_hash TEXT, provider_id TEXT, model TEXT, prompt_version TEXT, result_json TEXT NOT NULL, docx_s3_key TEXT, pdf_s3_key TEXT, created_at TEXT NOT NULL)`. Wire into `MIGRATIONS` array in `services/ui/src/db.rs`. `cache_key = sha256(jd_text‖provider_id‖model‖prompt_version)`. | TODO | Provider ID + prompt version in key are mandatory |
| W-RST.4.9 | `/dashboard/tailor` — React component in `web/src/routes/dashboard/Tailor.tsx` (ADR-019) — paste form, async submit (polling or Server-Sent Events), download links. Must honor ADR-013 dark theme. | TODO | |
| W-RST.4.10 | Cost cap + rate limit enforcement (daily token ceiling, per-request budget via `llm-core` middleware). Implementation references `plans/cross-cutting/llm-policy.md`. | TODO | |
| W-RST.4.11 | **Future**: if/when keyword-overlap matcher proves insufficient, add a concrete `EmbeddingProvider` impl — likely `crates/llm-fastembed` (local ONNX, zero-network) + sidecar table `job_detail_embeddings` keyed by `detail_id`. ADR-016 created at that point. | DEFERRED | Preserves same-vendor + zero-cost invariants; ADR-010 sidecar rationale already understood |

---

## W-RST.5 Test Strategy

- **Unit tests for `matcher.rs`** — deterministic, no secrets, no
  network. Given a known DB state + JD, assert exact scored candidates.
- **Contract tests for `api-openapi` models** — `TailorRequest`,
  `TailorResponse`, `MatchedBullet` round-trip JSON with all fields
  present and all absent (nullable variants).
- **Integration test for `POST /api/admin/tailor`** — uses
  `StubLlmProvider` from `llm-core/src/testing.rs`. No real API calls
  in CI. Asserts response shape and cache write.
- **Grounding contract test** — assert that `generator.rs` never
  produces output containing text strings NOT in
  `GroundingContract.allowed_source_text` (post-generation assertion
  against stub output).
- See `plans/cross-cutting/integration-tests.md` (W-QA) for alignment
  with the broader test infrastructure plan.

---

## W-RST.6 Cross-References

- → ADR-015 (structural decision: pluggable LLM framework + grounding contract)
- → W-LLM (`crates/llm-core` + `crates/llm-anthropic` — the LLM layer W-RST consumes)
- → W-RSM (static resume generator; W-RST is its interactive tailoring complement; shares `job_details` / `competencies` data)
- → W-SEC (`deploy-baba/prod/anthropic-api-key` — BLOCKED-on-deploy for LLM items)
- → W-APIO (ADR-012 SSOT — `tailor.rs` models in `crates/api-openapi/src/models/`)
- → W-UI (route surface, `require_auth` middleware, router registration)
- → W-SYNC (tailor_cache is NOT seeded and is therefore excluded from `/sync-dashboard-data` scope — it carries computed state, not edited content)
- → ADR-002 (SQLite on EFS — `tailor_cache` is a non-seeded table on the same EFS volume)
- → ADR-003 (Lambda Function URL — applies to the tailor endpoint, NOT ADR-009)
- → ADR-010 (upsert-reseed convention — `tailor_cache` is NOT seeded, by design; this ADR explains why that matters)
- → ADR-013 (admin dashboard dark theme — `/dashboard/tailor` must comply)
- → ADR-014 (`polish_bio_to_summary` v1 seam — claimed by W-RST.4.3)
- → `plans/cross-cutting/llm-policy.md` (cost caps, rate limits, retry, prompt versioning)
- → workspace `claude-api` skill (Anthropic SDK usage guidance at implementation time)
