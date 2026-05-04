# DRL-2026-05-03-coverage-floors

**Date:** 2026-05-03
**Status:** RESOLVED
**Domain:** W-XT, W-QA
**Discovered by:** `cargo xtask coverage floors`

---

## Observations

`just quality` failed at Step 4 (coverage floors) across 9 of 10 library crates.

Starting measurements vs. required floors:

| Crate | Measured | Floor | Gap |
|-------|----------|-------|-----|
| api-grpc | 79.7% | 80% | –0.3% |
| api-graphql | 75.6% | 80% | –4.4% |
| api-core | 86.2% | 90% | –3.8% |
| config-core | 83.5% | 90% | –6.5% |
| config-json | 73.5% | 85% | –11.5% |
| config-yaml | 72.5% | 85% | –12.5% |
| config-toml | 71.3% | 85% | –13.7% |
| api-openapi | 71.0% | 80% | –9.0% |
| api-merger | 34.2% | 80% | –45.8% |

`infra-types` (89.5% ≥ 75%) was the only passing crate.

---

## Root Causes

### Root Cause A — Missing tests

All 9 crates lacked tests for recently-added paths:
- `api-merger`: merge conflict tracking, `UnifiedApiSpec::to_json/content`, metadata fields, non-FailOnConflict strategies
- `api-openapi`: demo.rs models (5 types never exercised), `default_top_k()` serde default function, schema conflict detection
- `api-graphql`: `Default` impl, error conversions, interface type handling
- `api-core`: `SpecMetadata` construction, multi-spec merge, validation error display
- `config-*`: blanket `validate_*` impls, `load_*/save_*` file helpers, error conversions

### Root Cause B — `get_crate_coverage` TOTAL line pollution

`cargo llvm-cov --package X --summary-only` instruments all workspace crates compiled during X's test run. The `TOTAL` line therefore aggregates all instrumented files, not just X. `api-merger` (34.2% reported) had its own lib.rs at 96.6% — the low number was from dependency crates pulling down the aggregate.

---

## Fixes Applied

**Fix A — Tests added to all 9 crates** (`crates/*/src/lib.rs`, `crates/api-openapi/src/models/demo.rs`, `crates/api-openapi/src/models/ask.rs`):

Notable patterns used:
- Inline `struct`/`impl` inside test functions to avoid polluting public surface (config-core, api-core)
- Raw `utoipa::openapi::{OpenApiBuilder, ComponentsBuilder, PathsBuilder}` instead of `#[derive(OpenApi)]` proc macros (which don't work inside test fn scope)
- `1.5_f32`/`1.5_f64` instead of `3.14` for float test values (avoids `clippy::approx_constant`)
- `[ConflictType::A, ...]` array literal instead of `vec![...]` to avoid `clippy::useless_vec`

**Fix B — `xtask/src/coverage.rs` `get_crate_coverage` rewrite**

Replaced TOTAL-line parsing with per-file aggregation. New logic:
1. Parse every file line from `cargo llvm-cov --summary-only` output
2. Keep only lines whose path starts with `"{crate_name}/"` prefix
3. Sum `total_lines` and `missed_lines` across matching files
4. Compute coverage from the filtered sum

This eliminates dependency code inflation from the measurement entirely.

---

## Final State (all pass)

| Crate | Coverage | Floor |
|-------|----------|-------|
| config-core | 98.3% | ≥ 90% |
| api-core | 96.5% | ≥ 90% |
| api-openapi | 80.2% | ≥ 80% |
| api-graphql | 95.4% | ≥ 80% |
| api-merger | 96.5% | ≥ 80% |
| config-json | 97.0% | ≥ 85% |
| config-toml | 96.7% | ≥ 85% |
| api-grpc | 94.6% | ≥ 80% |
| infra-types | 89.5% | ≥ 75% |
| config-yaml | 95.5% | ≥ 85% |

`just quality` exits 0. `cargo xtask coverage floors` reports all ✅.

---

## Cross-References
- → `xtask/src/coverage.rs` — `get_crate_coverage` function (fixed per-file aggregation)
- → `plans/modules/xtask.md` — W-XT.4.8 (coverage.rs fix)
- → `plans/cross-cutting/quality-gates.md` — coverage floor definitions
