# ADR-015: LLM Provider Abstraction

**Date:** 2026-04-15
**Status:** Proposed
**Affected modules:** W-LLM, W-RST, W-RAG

## Context

Two upcoming features — resume tailoring (W-RST) and retrieval-augmented generation (W-RAG) — both
require calling a large language model. Without a shared abstraction, each feature would wire its
own HTTP client to the Anthropic API, creating duplicated secrets management, retry logic, and
token-counting code. The project already uses AWS Secrets Manager (W-SEC) and has a zero-cost
principle (ADR-005) that discourages managed AI services or fine-tuned models.

Constraints:
- Only one LLM provider is needed right now (Anthropic Claude); the architecture should not
  over-engineer for N providers.
- Embeddings and completion are distinct use-cases with different rate limits and cost profiles.
- All secrets must go through AWS Secrets Manager (`deploy-baba/prod/anthropic-api-key`); no
  hardcoded keys.
- The Lambda cold-start budget is tight; the LLM client must not perform blocking I/O at init time.

## Decision

> We will introduce two crates — `llm-core` (vendor-agnostic traits) and `llm-anthropic` (first
> impl) — that all LLM-touching code must use. Direct `reqwest` calls to Anthropic from
> feature crates are banned.

`llm-core` exposes three traits:

- `Completer` — streaming or batched text generation
- `Embedder` — dense vector embeddings over a slice of texts
- `PromptAssembler` — composes a `PromptBundle` (system + user messages + grounding block)

`llm-anthropic` implements all three against the Anthropic Messages API. Provider selection is done
via a Cargo feature flag (`default = ["anthropic"]`); alternative providers are added as additional
features without touching `llm-core`.

A **universal grounding contract** is enforced at the `PromptAssembler` layer: every bundle wraps
retrieved context in `<source kind="..." path="..." sha="...">…</source>` tags and the system
prompt requires the model to cite all claims with `[source N]` markers. This contract lives in
`plans/cross-cutting/llm-policy.md` and is shared by W-RST and W-RAG.

## Consequences

### Positive
- Single secrets path: `anthropic-api-key` fetched once at Lambda cold start via `init_api_key()`,
  mirroring the existing `init_pow_secret()` pattern (W-SEC).
- W-RST and W-RAG share retry logic, token counting, and error types.
- Swapping embedding provider (e.g. Voyage AI) is isolated to `llm-anthropic` without touching
  consumer crates.

### Negative / Trade-offs
- Two new crates add workspace overhead; both are PROPOSED until W-LLM.1.2 confirms them on disk.
- Feature-flag selection adds compile-time complexity for a codebase currently with no optional
  features in library crates.

### Neutral
- `llm-anthropic` becomes a dependency of `xtask` (for resume generation and `just ask`) and of
  `services/ui` (for `/api/ask`). This is acceptable given the existing pattern of `services/ui`
  importing many workspace crates.

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Direct `reqwest` calls per feature | Duplicates retry logic, secrets management, and token counting across W-RST and W-RAG |
| OpenAI / Bedrock as first provider | No existing account; Claude is already in use for development tooling (Anthropic is the right default) |
| Managed AI service (AWS Bedrock, etc.) | Violates ADR-005 zero-cost philosophy; adds IAM complexity |
| Single monolithic `llm` crate (traits + impls together) | Prevents compile-time provider swaps; breaks the project's core/impl separation pattern |

## Cross-References

- → ADR-005 (zero-cost philosophy — no managed AI services)
- → ADR-002 (SQLite — llm-core does not touch the DB; rag-sqlite does)
- → W-SEC (secrets management — `anthropic-api-key` stored in AWS Secrets Manager)
- → W-RAG (primary consumer of `Embedder` + `Completer`)
- → W-RST (resume tailoring — primary consumer of `Completer`)
- → `cross-cutting/llm-policy.md` (grounding contract and citation format)
