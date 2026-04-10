# ADR-015: LLM Provider Abstraction + Grounding Contract

**Date:** 2026-04-10
**Status:** Accepted
**Affected modules:** W-LLM (primary — new crate pair), W-RST (primary consumer), W-RSM (via `polish_bio_to_summary` upgrade), W-SEC (new secret), W-APIO (new models), W-UI (new route + feature flag), W-DX (per-crate READMEs)

## Context

The AI Resume Tailor pipeline (W-RST) requires LLM capabilities — keyword
extraction from job descriptions, grounded rewrite of resume bullets, and
(eventually) semantic embedding of resume data. The workspace already
contains 10 library crates structured as `-core` trait crate + concrete
adapter crates (`api-core` → `api-openapi` / `api-graphql` / `api-grpc`;
`config-core` → `config-toml` / `config-yaml` / `config-json`). The naive
implementation path — hardcoding the Anthropic SDK directly into
`services/ui/src/tailor/` — would abandon the workspace's core architectural
idiom and create vendor lock-in that a future provider swap would have to
pay down.

Separately, the generator function poses an integrity risk: a LLM that
fabricates resume content (invents skills, adds roles, inflates scope) would
damage the author's professional credibility. A grounding contract that
prevents invention must be enforced at a layer that applies uniformly across
providers — not in any individual adapter.

## Decision

> Introduce a **pluggable LLM provider abstraction** (`crates/llm-core` +
> adapter crates) mirroring the existing `-core` + adapter pattern. The
> generator operates under a **universal grounding contract** enforced at
> the `llm-core` prompt-assembly layer. The **reference implementation
> shipped with MVP is Anthropic Claude**, reusing the author's existing
> account. Provider selection is a cargo feature flag.

Specific rules:

1. **`crates/llm-core`** defines the vendor-agnostic trait surface:
   `LlmProvider`, `EmbeddingProvider` (forward-compat only, no concrete
   impl at MVP), `LlmRequest`, `LlmResponse`, `ChatMessage`, `ToolDef`,
   `ToolCall`, `GenerationConfig`, `GroundingContract`, `LlmError`. Zero
   vendor SDK dependencies. Follows `api-core` philosophy.

2. **`crates/llm-anthropic`** is the first concrete `LlmProvider`
   implementation, wrapping the Anthropic Rust SDK (or direct HTTP client).
   Depends on `llm-core`; has no dependency on `services/ui`. Default model:
   `claude-haiku-4-5-20251001`. Upgrade model: `claude-sonnet-4-6`.
   API key stored in AWS Secrets Manager as
   `deploy-baba/prod/anthropic-api-key` per W-SEC.

3. **`services/ui`** selects the active adapter at compile time via a cargo
   feature flag (`features = ["llm-anthropic"]` as default). Future adapters
   slot in by flipping the feature without touching `services/ui/src/tailor/`.

4. **Adapter injection**: adapters receive secrets via constructor injection
   (not env var lookup inside the adapter). The secret plumbing stays in
   `services/ui`; adapters remain unit-testable without real credentials.

5. **Grounding contract**: `LlmRequest::grounding: Option<GroundingContract>`
   carries a whitelist of `allowed_source_text` strings (exact bullet text
   from `job_details.detail_text`) and a `refusal_policy`. Prompt assembly
   helpers in `llm-core` enforce the contract before constructing the prompt
   for any adapter. The generator may only rephrase or reorder whitelisted
   bullets — never invent facts, never add skills absent from the source.

6. **Prompt version pinning**: every prompt has a `prompt_version` string
   constant compiled into the binary. The `tailor_cache` key is
   `sha256(jd_text || provider_id || model || prompt_version)`. Provider
   or model swaps — and prompt semantic changes — automatically invalidate
   cache entries.

7. **Embeddings deferred**: `EmbeddingProvider` trait is defined for
   forward-compatibility but no concrete impl ships with MVP. The resume
   matcher uses pure-Rust keyword / token-overlap scoring instead. If/when
   embeddings are needed, `crates/llm-fastembed` (local ONNX, zero-network)
   is the preferred path — no second vendor. ADR-016 (embeddings sidecar
   table) will be created at that point.

8. **Operational rules** (cost caps, rate limits, retry policy, PII
   scrubbing, prompt versioning, model migration runbook) live in the living
   policy doc `plans/cross-cutting/llm-policy.md`, not in this ADR.

## Consequences

### Positive
- Consistent with the workspace idiom: readers already familiar with
  `api-core` + adapters pattern immediately understand `llm-core` +
  adapters.
- Future provider swap is a one-crate write + one feature-flag flip.
  No changes to `services/ui/src/tailor/`.
- Grounding contract is inherited automatically by every future adapter —
  new providers cannot accidentally bypass it.
- Stub `LlmProvider` in `llm-core/src/testing.rs` eliminates network
  dependencies from CI tests. All `services/ui` integration tests use
  the stub.
- Dogfooding: building the resume tailor on the same model family that
  architected it tightens the feedback loop and validates the workflow
  described on the portfolio site itself.

### Negative / Trade-offs
- Introduces two new crates (`llm-core`, `llm-anthropic`) and a new domain
  code (`W-LLM`). Additional compile surface.
- No embeddings at MVP — purely keyword matching. Semantic relevance will
  be lower than embedding-based approaches, especially for non-obvious
  skill synonyms. Acceptable for a single-author admin tool.
- `polish_bio_to_summary()` upgrade (W-RST.4.3) adds a network dependency
  to `xtask resume generate`. Currently a local-only CLI tool; adding a
  required API call changes its runtime contract. Mitigation: make the LLM
  call optional behind a `--ai` flag so the local offline path remains
  available.

### Neutral
- `llm-anthropic` integration tests require `ANTHROPIC_API_KEY` env var;
  they skip in CI unless the secret is explicitly provided.
- The `EmbeddingProvider` trait carries no implementation cost at MVP.

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Hardcode Anthropic SDK in `services/ui/src/tailor/` | Abandons the workspace's core-plus-adapter idiom; creates vendor lock-in |
| OpenAI as the first provider | Would require a second vendor relationship (billing, API key, SDK); no qualitative advantage for this use case. Remains a future adapter option. |
| Bedrock (AWS-hosted Claude) | Keeps billing in AWS but loses account alignment with Claude Code; adds IAM model-access permissions and region constraints. Future adapter option. |
| Self-hosted via Ollama / ECS Fargate GPU | Violates ADR-005 (zero-cost philosophy). Future adapter option if the author ever wants to run private models. |
| Embeddings from OpenAI / Cohere at MVP | Introduces a second vendor just for embeddings. Breaks the pluggable-first-impl principle by coupling MVP to a non-Claude secret. Deferred to W-RST.4.11. |
| Single monolithic ADR covering embeddings | Embeddings are explicitly deferred and will require their own ADR (ADR-016 reserved) when a concrete impl ships. |

## Cross-References

- → W-LLM (primary — `crates/llm-core` + `crates/llm-anthropic`)
- → W-RST (primary consumer — resume-tailor pipeline)
- → W-RSM (via `polish_bio_to_summary` seam, W-RST.4.3)
- → W-SEC (new secret `deploy-baba/prod/anthropic-api-key`)
- → W-APIO (ADR-012 SSOT — `tailor.rs` models register in `ALL_MODELS`)
- → ADR-005 (zero-cost philosophy — guides provider selection)
- → ADR-010 (upsert-reseed — `tailor_cache` is not seeded, by design)
- → ADR-014 (`polish_bio_to_summary` v1 seam → claimed by W-RST.4.3)
- → `plans/cross-cutting/llm-policy.md` (operational rules)
- → workspace `claude-api` skill (SDK usage guidance at implementation time)
