# deploy-baba — Plan Index
**GitHub:** `shantopagla/deploy-baba` | **Last updated:** 2026-03-26
**Source repo:** `~/shanto` (Baba Toolchain, ~85K LOC) | **Status:** ~85% complete

See `plans/CONVENTIONS.md` for notation system, domain codes, and file naming rules.

---

## Module Status Table

| Module | Domain | Path | Status | Key Remaining Work |
|--------|--------|------|--------|--------------------|
| config-core | W-CFG | `crates/config-core/` | DONE | Per-crate README (W-DX.3) |
| config-toml | W-CFGT | `crates/config-toml/` | DONE | Per-crate README |
| config-yaml | W-CFGY | `crates/config-yaml/` | DONE | Per-crate README |
| config-json | W-CFGJ | `crates/config-json/` | DONE | Per-crate README |
| api-core | W-API | `crates/api-core/` | DONE | Per-crate README |
| api-openapi | W-APIO | `crates/api-openapi/` | DONE | Per-crate README |
| api-graphql | W-APIG | `crates/api-graphql/` | DONE | Per-crate README |
| api-grpc | W-APIR | `crates/api-grpc/` | DONE | Per-crate README |
| api-merger | W-APIM | `crates/api-merger/` | DONE | Per-crate README |
| infra-types | W-INFR | `crates/infra-types/` | DONE | Per-crate README |
| ui-service | W-UI | `services/ui/` | DONE | utoipa-rapidoc wiring (using inline HTML) |
| resume | W-RSM | `services/ui/migrations/`, `routes/resume.rs`, `routes/api/jobs.rs`, `routes/api/competencies.rs` | DONE | Functional view grouping (W-RSM.8.1), print CSS (W-RSM.8.3) |
| xtask | W-XT | `xtask/` | WIP | CLI naming mismatch (`fmt` vs `Format`), `EnvironmentInterpolator` unused |
| terraform | W-TF | `infra/` | SUPERSEDED | Replaced by W-OTF (OpenTofu). W-TF.4.1 and W-TF.4.2 already fixed in code. |
| opentofu | W-OTF | `infra/` + `xtask/src/infra/` | WIP | Install `tofu` binary (W-OTF.4.1 OPEN); smoke test (W-OTF.4.7 BLOCKED); docs (W-OTF.4.9 TODO) |
| dx-justfile | W-DX | `justfile`, `docs/`, `examples/` | WIP | Per-crate READMEs, integration tests |
| auth | W-AUTH | `services/ui/src/auth.rs`, `routes/auth.rs`, `routes/api/admin.rs`, `routes/dashboard.rs`, `infra/cognito.tf` | WIP | W-AUTH.4.22–4.28 dashboard master/detail refactoring (summary tiles, slug-routed job detail, inline sub-record editing, type-ahead navigation) |

---

## Remaining Work — Priority Order

### P0 — New Feature (in progress on `cognito-login` branch)
1. ~~**W-AUTH.4.1–4.15**~~ — Cognito auth + admin dashboard — **DONE** (code compiles clean, Cognito infra deployed to `us-east-1_I7c15vLHE`)
2. ~~**W-AUTH.4.20**~~ — Fix Lambda 504: lazy JWKS fetch — **SUPERSEDED** by W-AUTH.4.21
3. ~~**W-AUTH.4.21**~~ — Fix callback 504: implicit grant + JWKS from env — **DONE** (`allowed_oauth_flows=["implicit"]`; `data "http" cognito_jwks`; `COGNITO_JWKS` env var; HTML callback + `/auth/set-session`; self-sign-up disabled)
4. **W-AUTH.4.19** — OpenAPI security scheme + admin endpoint docs (`cookieAuth`/`bearerAuth`, 12 admin paths, `ToSchema` on input types)
5. **W-AUTH.4.22–4.28** — Dashboard master/detail refactoring (summary tiles, slug-routed job detail, inline sub-record editing, type-ahead navigation)

### P1 — Must Fix (blocking clean CI)
1. ~~**W-XT.4.1**~~ — CLI naming: 3 justfile mismatches fixed (`fmt`→`format`, `--crate`→`crate` subcommand, `gate`→`all`) — **RESOLVED**
2. ~~**W-TF.4.1**~~ — `infra/eventbridge.tf`: already uses `state = "ENABLED"` — **RESOLVED** (see DRL-2026-03-25-opentofu)
3. ~~**W-TF.4.2**~~ — `infra/s3.tf`: `filter {}` already present — **RESOLVED** (see DRL-2026-03-25-opentofu)
4. **W-XT.4.2** — Remove or wire up `EnvironmentInterpolator` (dead code)
5. **W-OTF.4.1–4.7** — Migrate infrastructure tooling from Terraform → OpenTofu (see `plans/modules/opentofu.md`)

### P2 — Quality Gate
5. **W-DX.3** — Per-crate README files (10 library crates)
6. **W-DX.4** — 4 standalone examples under `examples/`
7. **W-DX.5** — Integration tests for `just dev` pipeline
8. **W-XT.4.3** — Implement `just infra-bootstrap` (xtask bootstrap.rs) — creates S3 + DynamoDB + SSM
9. **W-QA** — Integration tests & test infrastructure (`plans/cross-cutting/integration-tests.md`) — 5 Phase-0 fixes done, add ~39 tests across phases 1–6

### P3 — Polish & Publish
9. **W-PUB.1** — `just publish-dry` passes for all 10 library crates
10. **W-PUB.2** — Tag `v0.1.0` + `just publish`
11. **W-UI.4.1** — Wire utoipa-rapidoc properly (currently using inline HTML)

---

## ADR Index

| ID | Title | Affected Modules |
|----|-------|-----------------|
| ADR-001 | justfile Is the Only Interface | W-DX, W-XT |
| ADR-002 | SQLite Over PostgreSQL | W-INFR, W-TF, W-XT |
| ADR-003 | Lambda Function URL (No API Gateway) | W-TF, W-UI |
| ADR-004 | Dual-Mode Entry Point | W-UI |
| ADR-005 | Zero-Cost Philosophy | W-CFG, W-API, W-INFR |
| ADR-006 | EFS + SQLite + S3 Backup | W-INFR, W-TF, W-XT |
| ADR-007 | OpenTofu Over Terraform | W-OTF, W-XT |
| ADR-008 | Cognito Authentication for Admin Dashboard | W-AUTH, W-UI, W-OTF |

---

## Drift Log Index

| ID | Date | Topic | Items |
|----|------|-------|-------|
| DRL-2026-03-18-terraform | 2026-03-18 | Terraform first-run gaps | 6 entries |
| DRL-2026-03-18-xtask | 2026-03-18 | xtask/justfile gaps | 7 entries |
| DRL-2026-03-25-opentofu | 2026-03-25 | OpenTofu migration audit | 6 entries |

---

## Dependency Graph Summary

```
config-core  ←── config-toml, config-yaml, config-json, infra-types (optional), services/ui
api-core     ←── api-openapi, api-graphql, api-grpc, api-merger, services/ui
api-openapi  ←── api-merger, services/ui
api-graphql  ←── api-merger
api-grpc     ←── api-merger
```

Full dependency order: `plans/cross-cutting/dependency-graph.md`

---

## Repository Structure

```
shantopagla/deploy-baba/
├── Cargo.toml              # Workspace (resolver = "2")
├── justfile                # THE developer interface
├── stack.toml              # Example stack definition
├── crates/                 # 10 library crates (all publishable)
│   ├── config-core/
│   ├── config-toml/
│   ├── config-yaml/
│   ├── config-json/
│   ├── api-core/
│   ├── api-openapi/
│   ├── api-graphql/
│   ├── api-grpc/
│   ├── api-merger/
│   └── infra-types/
├── services/ui/            # Portfolio site + deployed Lambda binary
├── xtask/                  # Internal tooling (called by justfile)
├── infra/                  # OpenTofu (Lambda + EFS + S3 + EventBridge)
├── examples/               # 4 standalone examples
├── docs/                   # aws-setup.md, architecture.md, etc.
└── plans/                  # This plan system (replaces PLAN.md monolith)
    ├── INDEX.md            # ← you are here
    ├── CONVENTIONS.md
    ├── adr/
    ├── modules/
    ├── cross-cutting/
    └── drift/
```

---

## Build Phase Progress

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Scaffold (workspace, justfile, stubs) | DONE |
| 2 | Extract & clean library crates | DONE |
| 3 | Complete library stubs | DONE |
| 4 | xtask modules | WIP (EnvironmentInterpolator dead code) |
| 5 | UI service | DONE |
| 6 | OpenTofu + end-to-end deploy | WIP (Terraform→OpenTofu migration W-OTF) |
| 7 | Examples + docs | TODO |
| 8 | Quality pass | TODO |
| 9 | Publish | TODO |
