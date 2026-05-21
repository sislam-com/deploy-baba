# Quality Gates — deploy-baba

**Command:** `just quality` → `cargo xtask quality all`
**CI:** `.github/workflows/ci.yml` — runs on every PR

---

## Coverage Floors

All 15 library crates must meet these minimums (enforced by `cargo xtask coverage`):

```
config-core:    90%   (current: 98.3%)
api-core:       90%   (current: 96.5%)
config-toml:    85%   (current: 96.7%)
config-yaml:    85%   (current: 95.5%)
config-json:    85%   (current: 97.0%)
api-openapi:    80%   (current: 80.2%)
api-graphql:    80%   (current: 95.4%)
api-grpc:       80%   (current: 94.6%)
api-merger:     80%   (current: 96.5%)
infra-types:    75%   (current: 89.5%)
llm-core:       70%   (current: 95.9%)
llm-anthropic:  70%   (current: ~70%)
llm-openai:     70%   (current: 81.9%)
rag-core:       70%   (current: 89.0%)
rag-sqlite:     70%   (current: 96.1%)
```

Original 10 floors passing as of 2026-05-03. See `plans/drift/DRL-2026-05-03-coverage-floors.md`
for the root-cause analysis and fix (per-file aggregation in `xtask/src/coverage.rs`).
LLM + RAG crate floors added 2026-05-21 as part of W-RAG embedding enhancement.

**Tool:** `cargo-llvm-cov` (`cargo install cargo-llvm-cov`)
**Binary excluded:** `services/ui/` and `xtask/` are excluded from coverage floors
(binary crates, tested via integration).

---

## `just quality` Pipeline

```
just quality
  └─► cargo xtask quality all
        ├─ cargo xtask build format --check    (formatting)
        ├─ cargo xtask build lint           (clippy, warnings = errors)
        ├─ cargo xtask test unit            (unit tests, no external deps)
        ├─ cargo xtask coverage check       (per-crate floors)
        └─ cargo audit                      (dependency security audit)
```

Must pass completely before any deploy:
```
just deploy PROFILE  →  just quality && push-image && update Lambda
```

---

## CI Gate (`.github/workflows/ci.yml`)

Triggered on: `push` to `main`, all pull requests.

```yaml
jobs:
  check:
    - cargo fmt --check
    - cargo clippy -- -D warnings
    - cargo test --workspace
    - cargo doc --no-deps --workspace  (doc-check)
    - cargo audit
```

Coverage floors are checked locally with `just quality` but not in CI
(avoids slow coverage instrumentation on every PR). Coverage is a pre-deploy gate.

---

## `cargo audit` Policy

- Zero known vulnerabilities in direct dependencies
- `cargo audit` is run as part of `just quality` and as a standalone `just audit`
- Unmaintained crate warnings do not fail the gate (only vulnerabilities do)

**Current state (2026-05-03):** 0 vulnerabilities. 1 allowed warning: `proc-macro-error`
(RUSTSEC-2024-0370, via `utoipa-gen 4.3.1`). Deferred to W-UI.4.1 (utoipa 4 → 5 migration).
RUSTSEC-2026-0098/0099/0104 (`rustls-webpki` CVEs) resolved — see `DRL-2026-05-03-rustsec-webpki-cves`.

---

## Doc Coverage

All public items in library crates must have rustdoc documentation.
Enforced by `cargo doc --no-deps --workspace` (warns on missing docs).
The CI `doc-check` step fails on warnings via `RUSTDOCFLAGS="-D warnings"`.

---

## Known Gaps (Phase 0 fixes — see W-QA)

These 5 deviations were found and fixed (Phase 0 complete):

| ID | Gap | Location | Fix |
|----|-----|----------|-----|
| W-QA.0.1 | `just test-crate` passes `--crate` flag but clap expects `crate` subcommand | `justfile:36` | `cargo xtask test crate {{CRATE}}` — FIXED |
| W-QA.0.2 | `cargo audit` step is missing from `quality.rs` | `xtask/src/quality.rs:54` | Add step 5 — FIXED |
| W-QA.0.3 | Quality gate uses global 80% threshold instead of per-crate floors | `xtask/src/quality.rs:51` | Switch to `CoverageAction::Floors` — FIXED |
| W-QA.0.4 | `just quality` calls `quality gate` but subcommand is `all` | `justfile:48` | `cargo xtask quality all` — FIXED |
| W-QA.0.5 | `just fmt` calls `build fmt` but subcommand is `format` | `justfile:16` | `cargo xtask build format` — FIXED |

Full checklist: → `plans/cross-cutting/integration-tests.md`

## Web / SPA Gates (once `web/` exists — ADR-019)

| Gate | Command | When |
|---|---|---|
| Type-check | `pnpm --dir web run typecheck` | CI + pre-merge |
| Lint | `pnpm --dir web run lint` | CI + pre-merge |
| Unit tests | `pnpm --dir web run test` | CI + pre-merge |
| Build | `pnpm --dir web run build` | CI + pre-merge |

The `web` job in `ci.yml` is conditional on `web/package.json` existing — no-op until Phase D.1 lands.

---

## OpenTofu Gates

| Gate | Command | When |
|---|---|---|
| Format | `tofu -chdir=infra fmt -check -recursive` | CI + pre-merge |
| Validate | `tofu -chdir=infra init -backend=false && tofu -chdir=infra validate` | CI + pre-merge |
| Plan | `just infra-plan-dev` | Pre-apply only |

`tofu apply` is never run from CI without a manual approval step (see `plans/adr/ADR-020-github-actions-ci-oidc.md`).

---

## Cross-References
- → `plans/modules/xtask.md` — W-XT quality/coverage implementation
- → `plans/modules/dx-justfile.md` — W-DX justfile recipe wiring
- → `plans/modules/ci.md` — W-CI CI job definitions
- → `plans/cross-cutting/dependency-graph.md` — crate list for coverage
- → `plans/cross-cutting/integration-tests.md` — W-QA full test infrastructure plan
- → `plans/cross-cutting/ai-dlc.md` — quality gate matrix in Session Lifecycle §4
