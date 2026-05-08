# ADR-005: Zero-Cost Philosophy

**Status:** Accepted
**Date:** 2026-03-10
**Affected modules:** W-CFG, W-API, W-INFR, W-TF

---

## Context

deploy-baba demonstrates Rust's zero-cost abstractions in a real production deployment.
The library crates must uphold this promise: using them should generate no runtime overhead
compared to hand-written code. This is both a marketing claim for the portfolio and a
technical design constraint.

---

## Decision

All library crates (`config-*`, `api-*`, `infra-types`) are designed around Rust's
zero-cost abstraction model:

1. **Monomorphization over dynamic dispatch** — traits are generic parameters, not
   `dyn Trait`, unless the use case genuinely requires runtime polymorphism.

2. **Compile-time template embedding** — `services/ui/` originally used Askama for
   compile-time HTML templates. [Superseded by ADR-019 (2026-04-30): HTML is now
   rendered by a React/Vite SPA in `web/`; `services/ui` serves JSON only. The
   zero-cost principle still holds — the SPA produces content-hashed static assets
   served from S3/EFS with no runtime overhead.]

3. **No hidden allocations** — public API surfaces avoid unnecessary `String` copies;
   prefer `&str` and `Cow<'_, str>` at boundaries.

4. **`thiserror` not `anyhow` in library crates** — `thiserror` generates zero-overhead
   error types. `anyhow` is permitted only in binary crates (`services/ui/`, `xtask/`).

5. **AWS infrastructure mirrors the philosophy** — Lambda Function URL (free, no
   middleman), SQLite on EFS (no database server overhead), EventBridge (event-driven,
   no polling daemon). See ADR-002 and ADR-003.

---

## Consequences

**Positive:**
- Library crates are suitable for embedding in high-performance applications
- The portfolio site demonstrates the philosophy end-to-end (code + infrastructure)
- Clear rule: if a crate needs `anyhow`, it belongs in the binary layer

**Negative:**
- More complex trait bounds in some public APIs
- `thiserror` requires more upfront error type design vs just using `anyhow::Error`
- Monomorphization can increase binary size when there are many instantiation sites
  (acceptable tradeoff for a portfolio binary)

---

## Cross-References
- `plans/modules/config-core.md` — W-CFG trait design
- `plans/modules/api-core.md` — W-API trait design
- `plans/modules/infra-types.md` — W-INFR type design
- `plans/cross-cutting/quality-gates.md` — coverage floors that enforce this
- `docs/zero-cost-philosophy.md` — user-facing explanation (TODO: W-DX.6)
