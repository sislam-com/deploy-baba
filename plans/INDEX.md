# deploy-baba — Plan Index
**GitHub:** `shantopagla/deploy-baba` | **Last updated:** 2026-04-08
**Source repo:** `~/shanto` (Baba Toolchain, ~85K LOC) | **Status:** ~93% complete

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
| resume | W-RSM | `services/ui/migrations/`, `routes/resume.rs`, `routes/api/jobs.rs`, `routes/api/competencies.rs` | DONE | 7 migrations (007 personal-projects seed added); xtask resume generate/upload done; Functional view grouping (W-RSM.8.1), print CSS (W-RSM.8.3) |
| xtask | W-XT | `xtask/` | WIP | Resume generate + S3 upload done (W-XT.4.5); `EnvironmentInterpolator` unused (W-XT.4.2) |
| terraform | W-TF | `infra/` | SUPERSEDED | Replaced by W-OTF (OpenTofu). W-TF.4.1 and W-TF.4.2 already fixed in code. |
| opentofu | W-OTF | `infra/` + `xtask/src/infra/` | WIP | Install `tofu` binary (W-OTF.4.1 OPEN); smoke test (W-OTF.4.7 BLOCKED); docs (W-OTF.4.9 TODO) |
| dx-justfile | W-DX | `justfile`, `docs/`, `examples/` | WIP | Per-crate READMEs, integration tests |
| auth | W-AUTH | `services/ui/src/auth.rs`, `routes/auth.rs`, `routes/api/admin.rs`, `routes/dashboard.rs`, `infra/cognito.tf` | DONE | W-AUTH.POST-FIX (CloudFront OAC body hash) |
| about | W-ABT | `services/ui/src/routes/about.rs`, `services/ui/templates/about_*.html`, `services/ui/migrations/008-009` | DONE | — |
| social-links | W-SL | `services/ui/src/db.rs`, `services/ui/src/routes/dashboard.rs`, `services/ui/src/routes/api/admin.rs`, `services/ui/migrations/010-011` | DONE | — |
| contact-form | W-CTF | `services/email/`, `services/ui/src/routes/contact.rs`, `infra/ses.tf`, `infra/email-lambda.tf`, `infra/apigateway.tf` | WIP | e2e test (W-CTF.4.12) — deploy step pending |
| secrets-manager | W-SEC | `xtask/src/secret.rs`, `infra/secrets.tf`, `infra/vpc-endpoints.tf`, `services/ui/src/routes/contact.rs` | DONE | Deploy: `just infra-apply` + `just secret-put pow-secret $(openssl rand -hex 32)` + `just lambda-deploy` |

---

## Remaining Work — Priority Order

### P0 — New Feature (in progress on `cognito-login` branch)
1. ~~**W-AUTH.4.1–4.15**~~ — Cognito auth + admin dashboard — **DONE** (code compiles clean, Cognito infra deployed to `us-east-1_I7c15vLHE`)
2. ~~**W-AUTH.4.20**~~ — Fix Lambda 504: lazy JWKS fetch — **SUPERSEDED** by W-AUTH.4.21
3. ~~**W-AUTH.4.21**~~ — Fix callback 504: implicit grant + JWKS from env — **DONE** (`allowed_oauth_flows=["implicit"]`; `data "http" cognito_jwks`; `COGNITO_JWKS` env var; HTML callback + `/auth/set-session`; self-sign-up disabled)
4. **W-AUTH.4.19** — OpenAPI security scheme + admin endpoint docs (`cookieAuth`/`bearerAuth`, 12 admin paths, `ToSchema` on input types)
5. ~~**W-AUTH.4.22–4.28**~~ — Dashboard master/detail refactoring — **DONE** (6 routes, 5 templates, type-ahead nav, dashboard.html monolith deleted)

### P0.5 — Live Site Post-Incident
1. ~~**W-AUTH.POST-FIX**~~ — **RESOLVED** for `POST /api/contact` via API Gateway HTTP API (ADR-009). Dashboard edit forms (PUT/PATCH via OAC path) remain broken — out of scope for now. See `DRL-2026-03-27-function-url-auth`.

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

### P2.5 — Content Features
12. ~~**W-ABT.4.1–4.10**~~ — DB-driven About section + admin CRUD — **DONE** (migrations 008–009, `/about/me`, `/about/repo`, dashboard routes, `POST/PUT/DELETE /api/admin/about`)
13. ~~**W-SL**~~ — DB-managed social links in top nav — **DONE** (migrations 010–011, `social_links` table, nav loop in `base.html`, dashboard CRUD, `POST/PUT/DELETE /api/admin/social-links`)
14. ~~**W-CTF.4.1–4.10**~~ — Contact form + SES + PoW + API Gateway — **DONE** (deployed 2026-04-03)
15. ~~**W-CTF.4.11 + W-SEC**~~ — Migrate `POW_SECRET` from Lambda env var → AWS Secrets Manager + xtask secret commands — **DONE** (code complete; `just infra-apply` + `just secret-put pow-secret ...` + `just lambda-deploy` still needed)
16. ~~**W-CTF.4.13**~~ — Acknowledgement email to submitter — **DONE** (SES production access granted 2026-04-08; `SES_ACK_FROM_EMAIL` restored; external Gmail delivery verified. See `DRL-2026-04-07-ses-sandbox-ack` (RESOLVED).)

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
| ADR-003 | Lambda Function URL (No API Gateway) — exception: POST /api/contact uses API Gateway (ADR-009) | W-TF, W-UI, W-CTF |
| ADR-004 | Dual-Mode Entry Point | W-UI |
| ADR-005 | Zero-Cost Philosophy | W-CFG, W-API, W-INFR |
| ADR-006 | EFS + SQLite + S3 Backup | W-INFR, W-TF, W-XT |
| ADR-007 | OpenTofu Over Terraform | W-OTF, W-XT |
| ADR-008 | Cognito Authentication for Admin Dashboard | W-AUTH, W-UI, W-OTF |
| ADR-009 | API Gateway HTTP API for POST /api/contact (OAC body hash workaround) | W-CTF, W-UI |
| ADR-010 | Synchronous Email Lambda Invocation with Typed Response Propagation + Acknowledgement Email | W-CTF, W-UI |

---

## Drift Log Index

| ID | Date | Topic | Items |
|----|------|-------|-------|
| DRL-2026-03-18-terraform | 2026-03-18 | Terraform first-run gaps | 6 entries |
| DRL-2026-03-18-xtask | 2026-03-18 | xtask/justfile gaps | 7 entries |
| DRL-2026-03-25-opentofu | 2026-03-25 | OpenTofu migration audit | 6 entries |
| DRL-2026-03-27-function-url-auth | 2026-03-27 | Lambda Function URL auth incident + revert | 2 entries + 2 open items (W-AUTH.POST-FIX, DRL-FUA-2) |
| DRL-2026-04-03-contact-form | 2026-04-03 | Contact Form + SES Email Lambda implementation | 4 entries, resolved |
| DRL-2026-04-03-pow-apigateway | 2026-04-03 | POST+PoW via API Gateway — replaces GET+query params | OAC body hash workaround, ADR-009 |
| DRL-2026-04-07-ses-sandbox-ack | 2026-04-07 | SES sandbox blocks ack emails to unverified recipients | **RESOLVED 2026-04-08** — production access granted; W-CTF.4.13 DONE; SES_ACK_FROM_EMAIL restored |

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
├── services/email/         # Email Lambda (SES sender, no VPC)
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

**Overall: ~90% complete**
