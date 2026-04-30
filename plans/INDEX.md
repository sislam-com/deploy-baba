# deploy-baba — Plan Index
**GitHub:** `shantopagla/deploy-baba` | **Last updated:** 2026-04-30
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
| dashboard-sync | W-SYNC | `plans/modules/dashboard-sync.md`, `services/ui/migrations/`, `services/ui/src/db.rs`, `services/ui/src/routes/api/admin.rs`, `.claude/skills/sync-dashboard-data/` | DONE | 4.1–4.5 complete; zero drift on first run 2026-04-08; .4.6/.4.7 deferred (on-demand) |
| llm-core + llm-anthropic | W-LLM | `crates/llm-core/`, `crates/llm-anthropic/` | TODO | All items; W-LLM.4.5/4.6 DEFERRED |
| resume-tailor | W-RST | `services/ui/src/tailor/`, `crates/api-openapi/src/models/tailor.rs`, `services/ui/migrations/016` | TODO | All items; BLOCKED-on-deploy for 4.3/4.4/4.5 until W-SEC deployed + `anthropic-api-key` in SM |
| rag | W-RAG | `crates/rag-core/`, `crates/rag-sqlite/` | PROPOSED | P1 CLI → P2 deploy-failure diagnosis → P3 /api/ask; blocked on W-LLM for generation. |
| gdrive-planning | W-GDR | `justfile`, `.claude/settings.json`, `.github/workflows/` | TODO | Drive MCP plan export/import (W-GDR.4.1–4.3); Stop hook quality gate (W-GDR.4.4); evaluated from Gemini proposal 2026-04-15 |
| ai-dlc | W-AIL | `.claude/agents/`, `.claude/skills/` | TODO | plan-doctor + drift-detector subagents; /plan-sync, /cache-refresh, /memory-curate skills; weekly schedule |
| ci | W-CI | `.github/workflows/` | TODO | GitHub OIDC roles; deploy-dev.yml + deploy-prod.yml; xtask release subcommand; SPA sync |
| web (SPA) | W-WEB | `web/` | TODO | Vite 6 + React 18 SPA replacing all 15 Askama templates; openapi-fetch client; phases D.1–D.5 |
| dev-environment | W-DEV | `scripts/`, `.devcontainer/` | TODO | bootstrap-tfstate.sh; dev-doctor.sh; devcontainer; initial-setup.md |

---

## Remaining Work — Priority Order

### P0.1 — AI-DLC + Deployment Automation + Full SPA (this import, branch: `feat/llm-core`)

1. **W-AIL.4.1–4.5** — Anti-rot agents (`plan-doctor`, `drift-detector`) and skills (`/plan-sync`, `/cache-refresh`, `/memory-curate`). Phase B.
2. **W-DEV.4.1–4.6** — Dev-environment scripts + devcontainer + initial-setup.md. Phase E.
3. **W-CI.4.1–4.10** — CI/CD with OIDC + xtask release subcommand + dev Lambda + workflows (Lambda-only, Phase C.1).
4. **W-WEB.4.1–4.3** — SPA scaffold (`web/`) + missing JSON API endpoints. Phase D.1.
5. **W-CI.4.11–4.12** — Extend workflows with SPA sync steps. Phase C.2.
6. **W-WEB.4.4–4.5** — Port `/ask` + `/dashboard/*` to React. Phase D.2.
7. **W-WEB.4.6** — Port marketing routes. Phase D.3.
8. **W-WEB.4.7–4.8** — Flip Axum router + SPA sync handler + s3-spa.tf. Phase D.4.
9. **W-WEB.4.9** — Remove Askama. Phase D.5.
10. **W-AIL.4.7** — Wire weekly schedule (`dbb-plan-sync`, `dbb-memory-curate`) via `/schedule`.

---

### P0 — New Feature (in progress on `cognito-login` branch)
1. ~~**W-AUTH.4.1–4.15**~~ — Cognito auth + admin dashboard — **DONE** (code compiles clean, Cognito infra deployed to `us-east-1_I7c15vLHE`)
2. ~~**W-AUTH.4.20**~~ — Fix Lambda 504: lazy JWKS fetch — **SUPERSEDED** by W-AUTH.4.21
3. ~~**W-AUTH.4.21**~~ — Fix callback 504: implicit grant + JWKS from env — **DONE** (`allowed_oauth_flows=["implicit"]`; `data "http" cognito_jwks`; `COGNITO_JWKS` env var; HTML callback + `/auth/set-session`; self-sign-up disabled)
4. **W-AUTH.4.19** — OpenAPI security scheme + admin endpoint docs (`cookieAuth`/`bearerAuth`, 12 admin paths, `ToSchema` on input types)
5. ~~**W-AUTH.4.22–4.28**~~ — Dashboard master/detail refactoring — **DONE** (6 routes, 5 templates, type-ahead nav, dashboard.html monolith deleted)

### P0.5 — Live Site Post-Incident
1. ~~**W-AUTH.POST-FIX**~~ — **RESOLVED** for `POST /api/contact` via API Gateway HTTP API (ADR-009). Dashboard edit forms (PUT/PATCH via OAC path) remain broken — out of scope for now. See `DRL-2026-03-27-function-url-auth`.

### P1 — Must Fix (blocking clean CI)
1. ~~**W-SYNC.4.5**~~ — **DONE 2026-04-08:** pulled live EFS DB via dump endpoint; zero drift — live matches seeds exactly. ~~`.4.2`~~ + ~~`.4.3`~~ + ~~`.4.4`~~ + ~~`.4.5`~~ DONE. W-SYNC is now on-demand (run `/sync-dashboard-data` after dashboard edits).
2. ~~**W-XT.4.1**~~ — CLI naming: 3 justfile mismatches fixed (`fmt`→`format`, `--crate`→`crate` subcommand, `gate`→`all`) — **RESOLVED**
3. ~~**W-TF.4.1**~~ — `infra/eventbridge.tf`: already uses `state = "ENABLED"` — **RESOLVED** (see DRL-2026-03-25-opentofu)
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
17. **W-LLM.4.1–4.4** — LLM provider abstraction + Claude reference adapter (see `plans/modules/llm-core.md`) — TODO
18. **W-RST.4.1–4.10** — AI Resume Tailor pipeline on W-LLM (see `plans/modules/resume-tailor.md`) — TODO; BLOCKED-on-deploy for items 4.3/4.4/4.5 until W-SEC deployed + `anthropic-api-key` in SM

### P3 — Polish & Publish
9. **W-GDR.4.1–4.4** — Google Drive MCP setup + `plan-export`/`plan-import` justfile recipes + `Stop` hook quality gate (see `plans/modules/gdrive-planning.md`)
10. **W-PUB.1** — `just publish-dry` passes for all 10 library crates
11. **W-PUB.2** — Tag `v0.1.0` + `just publish`
11. **W-UI.4.1** — Wire utoipa-rapidoc properly (currently using inline HTML)

### P3 — LLM + RAG Subsystem (new, phased)
12. **W-LLM** — Author `crates/llm-core` + `crates/llm-anthropic` + ADR-015 (prerequisite for W-RAG generation and W-RST)
13. **W-RAG.2.1–3.4** — `rag-core` + `rag-sqlite` crates, chunkers, `xtask rag ingest/query`, justfile verbs (P1: FTS-only CLI, no embedder needed)
14. **W-RAG.4.1–4.2** — Wire embedder + generate via W-LLM (BLOCKED on W-LLM)
15. **W-RAG.5.1** — Deploy-failure diagnosis hook (P2; BLOCKED on W-RAG.4.2)
16. **W-RAG.6.1–6.3** — Public `/api/ask` endpoint + rate-limit + Lambda bundle (P3; BLOCKED on W-RAG.4.2)

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
| ADR-010 | SQLite Upsert as the Canonical Re-Seed Convention | W-SYNC, W-RSM, W-ABT, W-SL, W-XT |
| ADR-011 | Synchronous Email Lambda Invocation with Typed Response Propagation + Acknowledgement Email | W-CTF, W-UI |
| ADR-012 | OpenAPI SSOT + Public/Admin Spec Split | W-APIO, W-UI, W-LLM, W-RST |
| ADR-013 | Admin Dashboard Dark Theme Convention — light-theme tokens banned in `dashboard_*.html`; canonical dark-palette class table for all dashboard list/detail/form views | W-AUTH, W-ABT, W-SL, W-RSM, W-UI |
| ADR-014 | Resume Professional Summary Sourced from DB (`about_sections.me-bio`) — hardcoded `SUMMARY` const deleted; generator loads + polishes bio at generation time; errors on missing row | W-RSM, W-XT, W-RST |
| ADR-015 | LLM Provider Abstraction + Grounding Contract — `crates/llm-core` (vendor-agnostic trait) + `crates/llm-anthropic` (first impl); universal grounding contract at prompt-assembly layer; Claude as MVP provider; cargo feature-flag selection | W-LLM, W-RST, W-RAG, W-RSM, W-SEC, W-APIO, W-UI |
| ADR-016 | RAG Architecture — SQLite + sqlite-vec + FTS5 hybrid retrieval; all embedding/generation via llm-core (ADR-015); per-corpus chunkers; `.claude/` cache local-CLI only | W-RAG |
| ADR-017 | AI-Assisted Development Lifecycle (AI-DLC) — 6-stage session lifecycle; agent-cache startup protocol; quality gates; maintenance stage anti-rot | all |
| ADR-018 | Anti-rot Agents — plan-doctor + drift-detector subagents; /plan-sync, /cache-refresh, /memory-curate skills; weekly schedule | W-AIL |
| ADR-019 | React/Vite SPA Replaces Askama — full replacement; hybrid JSON API + asset server; phases D.1–D.5; SEO prerender P3 | W-WEB, W-UI, W-AUTH, W-ABT, W-SL, W-RSM, W-RAG, W-OTF, W-CI |
| ADR-020 | GitHub Actions CI with OIDC — two IAM roles (dev/prod); no long-lived keys; deploy-dev.yml + deploy-prod.yml | W-CI, W-OTF, W-WEB |
| ADR-021 | Automated Release Tagging via xtask — `dev-vX.Y.Z` on CI; `vX.Y.Z` via `just release-promote`; conventional-commits versioning | W-CI, W-XT |
| ADR-022 | Developer First-Run Environment — scripts/bootstrap-tfstate.sh; scripts/dev-doctor.sh; .devcontainer/; initial-setup.md | W-DEV, W-DX |

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
| DRL-2026-04-03-secrets-manager | 2026-04-03 | W-SEC/W-CTF: POW_SECRET + cognito_temp_password migrated from Lambda env vars to AWS Secrets Manager | Code complete; deploy: `just infra-apply` + `just secret-put pow-secret` + `just lambda-deploy` |
| DRL-2026-04-07-ses-sandbox-ack | 2026-04-07 | SES sandbox blocks ack emails to unverified recipients | **RESOLVED 2026-04-08** — production access granted; W-CTF.4.13 DONE; SES_ACK_FROM_EMAIL restored |
| DRL-2026-04-08-api-openapi-orphan | 2026-04-08 | api-openapi was orphaned from services/ui (W-APIO SSOT) | **RESOLVED 2026-04-08** — SSOT refactor complete; 29 models, dual-spec, 84 tests |

---

## Dependency Graph Summary

```
config-core  ←── config-toml, config-yaml, config-json, infra-types (optional), services/ui
api-core     ←── api-openapi, api-graphql, api-grpc, api-merger, services/ui
api-openapi  ←── api-merger, services/ui
api-graphql  ←── api-merger
api-grpc     ←── api-merger
llm-core     ←── llm-anthropic, services/ui (via W-RST tailor pipeline)
```

Full dependency order: `plans/cross-cutting/dependency-graph.md`

Implementation sequencing for W-LLM/W-RST/W-RAG/W-GDR: `plans/cross-cutting/execution-roadmap.md`

---

## Cross-Cutting Docs

| Doc | Purpose |
|-----|---------|
| `aws-architecture.md` | AWS resource topology |
| `aws-setup-spec.md` | IAM policy + profile bootstrapping |
| `dependency-graph.md` | Crate dependency order |
| `execution-roadmap.md` | W-LLM/W-RST/W-RAG/W-GDR sequencing |
| `llm-policy.md` | LLM ops: provider registry, prompt versioning, cost caps, retry/fallback, PII |
| `publishing.md` | crates.io release plan (W-PUB) |
| `quality-gates.md` | Quality gate definitions (Rust + Web + OpenTofu) |
| `integration-tests.md` | W-QA test infrastructure plan |
| `ai-dlc.md` | AI Development Lifecycle — session protocol + quality gates + maintenance agents |
| `initial-setup.md` | Developer first-run guide (prerequisites, bootstrap, local dev loop) |

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
