# deploy-baba — Plan Index
**GitHub:** `sislam-com/deploy-baba` | **Last updated:** 2026-05-14
**Source repo:** `~/shanto` (Baba Toolchain, ~85K LOC) | **Status:** ~95% complete

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
| ui-service | W-UI | `services/ui/` (API + asset server; no templates) | DONE | utoipa-rapidoc wiring (using inline HTML); sync.rs EFS swap handler added (D.4) |
| resume | W-RSM | `services/ui/migrations/`, `routes/api/jobs.rs`, `routes/api/competencies.rs`, `routes/api/resume.rs` | DONE | 7 migrations; xtask resume generate/upload done; Functional view grouping (W-RSM.8.1), print CSS (W-RSM.8.3) |
| xtask | W-XT | `xtask/` | WIP | release subcommand DONE; deploy spa.rs DONE; `EnvironmentInterpolator` unused (W-XT.4.2) |
| terraform | W-TF | `infra/` | SUPERSEDED | Replaced by W-OTF (OpenTofu). W-TF.4.1 and W-TF.4.2 already fixed in code. |
| opentofu | W-OTF | `infra/` + `xtask/src/infra/` | DONE | `tofu` v1.11.5 installed; plan runs clean (W-OTF.4.7 DONE 2026-05-01). `acm.tf` added; `cdn.tf` updated for `dev.sislam.com` + wildcard cert. |
| dx-justfile | W-DX | `justfile`, `docs/`, `examples/` | WIP | Per-crate READMEs DONE (10 crates, MIT license); examples TODO, integration tests TODO
| auth | W-AUTH | `services/ui/src/auth.rs`, `routes/auth.rs`, `routes/api/admin.rs`, `infra/cognito.tf` | DONE | W-AUTH.POST-FIX (CloudFront OAC body hash); dashboard now React (W-WEB) |
| about | W-ABT | `services/ui/src/routes/api/about.rs`, `services/ui/migrations/008-009` | DONE | Templates deleted (D.5); data served via JSON API to SPA |
| social-links | W-SL | `services/ui/src/db.rs`, `services/ui/src/routes/api/admin.rs`, `services/ui/migrations/010-011` | DONE | Templates deleted (D.5); nav loop now in React Layout.tsx |
| contact-form | W-CTF | `services/email/`, `services/ui/src/routes/contact.rs`, `infra/ses.tf`, `infra/email-lambda.tf`, `infra/apigateway.tf` | DONE | e2e test (W-CTF.4.12) — smoke tests created in services/ui/tests/contact_smoke.rs |
| challenges | W-CHL | `services/ui/src/routes/api/challenges.rs`, `services/ui/migrations/022`, `web/src/routes/dashboard/Challenges.tsx` | DONE | Basic CRUD DONE; RAG corpus integration DONE; admin UI DONE; public pages DONE (W-CHL.4.11); search/filter DONE (W-CHL.4.13); evaluation metrics deferred (W-CHL.4.12) |
| secrets-manager | W-SEC | `xtask/src/secret.rs`, `infra/secrets.tf`, `infra/vpc-endpoints.tf`, `services/ui/src/routes/contact.rs` | DONE | Deploy: `just infra-apply` + `just secret-put pow-secret $(openssl rand -hex 32)` + `just lambda-deploy` |
| dashboard-sync | W-SYNC | `plans/modules/dashboard-sync.md`, `services/ui/migrations/`, `services/ui/src/db.rs`, `services/ui/src/routes/api/admin.rs`, `.claude/skills/sync-dashboard-data/` | DONE | 4.1–4.5 complete; zero drift on first run 2026-04-08; .4.6/.4.7 deferred (on-demand) |
| llm-core + llm-anthropic + llm-openai | W-LLM | `crates/llm-core/`, `crates/llm-anthropic/`, `crates/llm-openai/` | DONE | W-LLM.4.1–4.16 all DONE; LlmProvider + EmbeddingProvider traits; Anthropic + OpenAI adapters; tool-dispatch loop (ADR-023) |
| resume-tailor | W-RST | `services/ui/src/tailor/`, `crates/api-openapi/src/models/tailor.rs`, `services/ui/migrations/016` | TODO | All items; BLOCKED-on-deploy for 4.3/4.4/4.5 until W-SEC deployed + `anthropic-api-key` in SM |
| rag | W-RAG | `crates/rag-core/`, `crates/rag-sqlite/` | DONE | P1–P5 complete; 7 corpora; eval.rs live; embedding + hybrid FTS/ANN retrieval via RRF (W-RAG.4.1 DONE); sqlite-vec native ANN deferred (W-RAG.6.2) |
| gdrive-planning | W-GDR | `justfile`, `.claude/settings.json`, `.github/workflows/` | TODO | Drive MCP plan export/import (W-GDR.4.1–4.3); Stop hook quality gate (W-GDR.4.4); evaluated from Gemini proposal 2026-04-15 |
| ai-dlc | W-AIL | `.claude/agents/`, `.claude/skills/` | DONE | plan-doctor + drift-detector subagents; /plan-sync, /cache-refresh, /memory-curate skills; weekly schedule |
| ci | W-CI | `.github/workflows/` | WIP | Code complete (C.1 + C.2 DONE). W-CI.4.9 RESOLVED 2026-05-04 — GH Variables replaced by SM fetch (DRL-2026-05-04-sislam-outage); bootstrap ARNs set. Remaining: W-CI.4.5 (dev Lambda workspace), W-CI.4.10 (production env gate) |
| web | W-WEB | `web/` | DONE | All 15 Askama templates replaced; Askama removed; CF→S3 direct serving (EFS sync dropped 2026-05-04, DRL-2026-05-04-sislam-outage); SEO prerender deferred to W-WEB.5 (P3) |
| dev-environment | W-DEV | `scripts/`, `.devcontainer/` | DONE | bootstrap-tfstate.sh; dev-doctor.sh; devcontainer; initial-setup.md |
| api-versioning | W-VER | `services/ui/src/middleware/`, `services/ui/src/router.rs` | DONE | URL-based versioning with /api/v1/ paths; backward-compatible redirects; deprecation middleware; OpenAPI version metadata |
| observability | W-OBS | `services/ui/src/telemetry.rs`, `services/ui/migrations/` | DONE | `metrics_middleware` (fire-and-forget SQLite writes); `GET /api/v1/metrics` (p50/p95/p99 + error rate); admin-gated |
| resilience | W-RES | `services/ui/src/middleware/` | DONE | `rate_limit_middleware` (100 req/60s per IP+endpoint); `CircuitBreaker` around LLM calls (5 failures → 60s open); `validate_request_middleware` (1 MB body guard); `RetryPolicy` available for handler retry |
| module-decomposition | W-MOD | `services/ui/src/modules/` | TODO | Logical module separation (portfolio, rag, admin, auth); independent testing per module; module-specific metrics |
| mcp-cloud | W-MCP | `crates/mcp-rs/`, `services/mcp-gateway/` | WIP | Private MCP gateway; local mcp-rs + cloud Cognito-authenticated Lambda gateway (ADR-028) |
| env-promote | W-PROM | `xtask/src/deploy/promote.rs`, `infra/*.tf`, `.github/workflows/` | TODO | Dev/prod separation via OT workspaces; `just promote` artifact promotion; xtask workspace refactoring (ADR-029) |
| saas-onboard | W-SAAS | `xtask/src/onboard.rs`, `crates/portfolio-rag-mcp/`, `services/ui/src/routes/api/eval.rs` | WIP | Project onboarding for external repos; eval dashboard; project_health MCP tool (ADR-030) |

---

## Remaining Work — Priority Order

### P0.1 — AI-DLC + Deployment Automation + Full SPA (branch: `feat/challenges`)

1. ~~**W-AIL.4.1–4.5**~~ **DONE** — Anti-rot agents + skills (Phase B complete).
2. ~~**W-DEV.4.1–4.6**~~ **DONE** — Dev-environment scripts + devcontainer (Phase E complete).
3. ~~**W-CI.4.1–4.4, 4.6–4.8**~~ **DONE** — xtask release subcommand + OIDC infra + workflows (Phase C.1).
4. ~~**W-WEB.4.1–4.3**~~ **DONE** — SPA scaffold (`web/`) + missing JSON API endpoints (Phase D.1).
5. ~~**W-CI.4.11–4.12**~~ **DONE** — Extended deploy-dev.yml + deploy-prod.yml with SPA sync steps (Phase C.2).
6. ~~**W-WEB.4.4–4.5**~~ **DONE** — `/ask` + `/dashboard/*` ported to React (Phase D.2).
7. ~~**W-WEB.4.6**~~ **DONE** — Marketing routes ported to React (Phase D.3).
8. ~~**W-WEB.4.7–4.8**~~ **DONE** — Axum router flipped to SPA asset server + sync.rs + s3-spa.tf (Phase D.4).
9. ~~**W-WEB.4.9**~~ **DONE** — Askama removed; 15 templates deleted (Phase D.5).
10. ~~**Local deploy pipeline**~~ **DONE** — `xtask deploy spa`, `just deploy-full/spa-deploy/lambda-wait`, `/deploy --full` skill extended.
11. **W-AIL.4.7** — Wire weekly schedule (`dbb-plan-sync`, `dbb-memory-curate`) via `/schedule`.
12. **W-CI.4.5, 4.9, 4.10** — Manual one-time steps: dev Lambda workspace, GitHub Variables, `production` environment gate.

---

### P0 — New Feature (in progress on `cognito-login` branch)
1. ~~**W-AUTH.4.1–4.15**~~ — Cognito auth + admin dashboard — **DONE** (code compiles clean, Cognito infra deployed to `us-east-1_I7c15vLHE`)
2. ~~**W-AUTH.4.20**~~ — Fix Lambda 504: lazy JWKS fetch — **SUPERSEDED** by W-AUTH.4.21
3. ~~**W-AUTH.4.21**~~ — Fix callback 504: implicit grant + JWKS from env — **DONE** (`allowed_oauth_flows=["implicit"]`; `data "http" cognito_jwks`; `COGNITO_JWKS` env var; HTML callback + `/auth/set-session`; self-sign-up disabled)
4. ~~**W-AUTH.4.19**~~ — OpenAPI security scheme + admin endpoint docs — **DONE** (`cookieAuth`/`bearerAuth`, 12 admin paths, `ToSchema` on input types)
5. ~~**W-AUTH.4.22–4.28**~~ — Dashboard master/detail refactoring — **DONE** (6 routes, 5 templates, type-ahead nav, dashboard.html monolith deleted)

### P0.5 — Live Site Post-Incident
1. ~~**W-AUTH.POST-FIX**~~ — **RESOLVED** for `POST /api/contact` via API Gateway HTTP API (ADR-009). Dashboard edit forms (PUT/PATCH via OAC path) remain broken — out of scope for now. See `DRL-2026-03-27-function-url-auth`.

### P1 — Must Fix (blocking clean CI)
1. ~~**W-SYNC.4.5**~~ — **DONE 2026-04-08:** pulled live EFS DB via dump endpoint; zero drift — live matches seeds exactly. ~~`.4.2`~~ + ~~`.4.3`~~ + ~~`.4.4`~~ + ~~`.4.5`~~ DONE. W-SYNC is now on-demand (run `/sync-dashboard-data` after dashboard edits).
2. ~~**W-XT.4.1**~~ — CLI naming: 3 justfile mismatches fixed (`fmt`→`format`, `--crate`→`crate` subcommand, `gate`→`all`) — **RESOLVED**
3. ~~**W-TF.4.1**~~ — `infra/eventbridge.tf`: already uses `state = "ENABLED"` — **RESOLVED** (see DRL-2026-03-25-opentofu)
3. ~~**W-TF.4.2**~~ — `infra/s3.tf`: `filter {}` already present — **RESOLVED** (see DRL-2026-03-25-opentofu)
4. ~~**W-XT.4.2**~~ — Remove or wire up `EnvironmentInterpolator` — **DEFERRED** (kept as intentional placeholder in config-core; location corrected from xtask)
5. ~~**W-OTF.4.1–4.7**~~ — **DONE 2026-05-01** — `tofu` v1.11.5 installed; `just infra-plan deploy-baba` clean. HCL fixes: duplicate `aws_caller_identity`, duplicate `file_system_config`, lifecycle `filter {}`. See DRL-2026-05-01-infra-plan-blockers.

### P2 — Quality Gate
5. ~~**W-DX.3**~~ — Per-crate README files (10 library crates) — **DONE** (MIT license for all 10 crates: config-core, config-toml, config-yaml, config-json, api-core, api-openapi, api-graphql, api-grpc, api-merger, infra-types)
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
17. ~~**W-CHL.4.1–4.9**~~ — Challenges feature + admin CRUD + RAG corpus integration — **DONE** (migration 022, API routes, admin dashboard, RAG 7th corpus integration)
18. **W-CHL.4.10–4.13** — Challenges remaining features — TODO (admin edit/delete forms, public portfolio pages, evaluation metrics, search/filter)
19. **W-LLM.4.1–4.4** — LLM provider abstraction + Claude reference adapter (see `plans/modules/llm-core.md`) — TODO
18. **W-RST.4.1–4.10** — AI Resume Tailor pipeline on W-LLM (see `plans/modules/resume-tailor.md`) — TODO; BLOCKED-on-deploy for items 4.3/4.4/4.5 until W-SEC deployed + `anthropic-api-key` in SM

### P2.7 — SaaS AI-DLC (ADR-030)
26. **W-SAAS.4.3** — `project_health` MCP tool — **DONE** (combines plan coverage + drift + cache age + eval score)
27. **W-SAAS.4.4–4.6** — Eval dashboard: `rag_eval_results` table exists (migration 023); `just rag-eval` persistence + `GET /api/v1/eval/dashboard` endpoint — TODO
28. **W-SAAS.4.7–4.9** — `xtask onboard <repo-url>` external repo onboarding: language detection, artifact generation, RAG indexing — TODO
29. **W-SAAS.4.10–4.11** — `onboard_project` MCP tool + justfile recipes — TODO

### P3 — Polish & Publish
9. **W-GDR.4.1–4.4** — Google Drive MCP setup + `plan-export`/`plan-import` justfile recipes + `Stop` hook quality gate (see `plans/modules/gdrive-planning.md`)
10. **W-PUB.1** — `just publish-dry` passes for all 10 library crates
11. **W-PUB.2** — Tag `v0.1.0` + `just publish`
11. ~~**W-UI.4.1**~~ — Wire utoipa-rapidoc properly — **DONE** (inline HTML approach works fine; loads RapiDoc from CDN)

### P2.6 — Zero-Cost Microservices Enhancements
22. ~~**W-VER.4.1–4.4**~~ **DONE** — API versioning strategy (ADR-024) — URL-based /api/v1/ routing, version extraction middleware, deprecation headers, OpenAPI version metadata
23. ~~**W-OBS.4.1–4.4**~~ **DONE** — SQLite-based observability (ADR-025) — `metrics_middleware` fire-and-forget writes; `GET /api/v1/metrics` p50/p95/p99 + error rate; admin-gated
24. ~~**W-RES.4.1–4.4**~~ **DONE** — Code-level resilience patterns (ADR-026) — `rate_limit_middleware` (100 req/60s); `CircuitBreaker` around LLM calls; `validate_request_middleware` (1 MB guard); `RetryPolicy` available for handler use
25. **W-MOD.4.1–4.3** — Module-based service decomposition (ADR-027) — TODO (logical module separation, independent testing per module, module-specific metrics collection)

### P3 — LLM + RAG Subsystem (phased)
12. ~~**W-LLM**~~ — `crates/llm-core` + `crates/llm-anthropic` + ADR-015 — **DONE** (W-LLM.4.1–4.5)
13. ~~**W-RAG.2.1–3.4**~~ — `rag-core` + `rag-sqlite` crates, chunkers, xtask rag, justfile verbs — **DONE** (P1 FTS-only)
14. ~~**W-RAG.4.1**~~ — Wire embedder (OpenAI text-embedding-3-small via `LlmEmbedder` bridge) — **DONE** (2026-05-21)
15. ~~**W-RAG.4.2 + 5.1**~~ — PromptAssembler + generate integration + deploy-failure diagnosis hook — **DONE** (2026-04-15)
16. ~~**W-RAG.6.1–6.3**~~ — Public `/api/ask` endpoint + rate-limit — **DONE** (2026-05-01)
17. ~~**W-RAG.7.1–7.5**~~ — Extended RAG corpora: OpenAPI spec + portfolio data chunkers, 7-corpus ingest — **DONE**
18. ~~**W-RAG.8.1–8.2**~~ — Portfolio-aware prompt assembly + filtered retrieval — **DONE**
19. ~~**W-RAG.9.1–9.6**~~ — Live-data retrieval: `PortfolioDataProvider`, `HybridRetriever`, hybrid fix — **DONE** (2026-05-09)
20. ~~**W-LLM.4.8–4.14**~~ — Tool-dispatch loop: `ToolExecutor` trait, `run_agent_loop()`, `ChatMessage` content enum, Anthropic adapter update (ADR-023) — **DONE**
21. ~~**W-RAG.10.1–10.6 + 11.1–11.5**~~ — Agentic portfolio assistant + challenges corpus — **DONE**
22. ~~**W-LLM.4.15**~~ — OpenAI `LlmProvider` adapter — **DONE** (347 LOC)
23. ~~**W-LLM.4.16**~~ — OpenAI `EmbeddingProvider` adapter (text-embedding-3-small) — **DONE** (2026-05-21)
24. **W-RAG.6.2** — Bundle `sqlite-vec` aarch64 SO into Lambda zip — DEFERRED (brute-force ANN sufficient for current corpus)
25. ~~**W-RAG.12.1**~~ — Deterministic groundedness scoring (`eval.rs`) — **DONE** (live in ask handler)

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
| ADR-023 | Agentic Tool-Dispatch Architecture — provider-agnostic agent loop in llm-core; HTTP call-back from llm-proxy to UI Lambda for tool execution; ChatMessage breaking change | W-LLM, W-RAG, W-UI, W-APIO |
| ADR-024 | API Versioning Strategy — URL-based versioning with Function URL routing; version extraction middleware; deprecation headers; OpenAPI version metadata | W-VER, W-UI, W-APIO |
| ADR-025 | SQLite-Based Metrics Collection — Zero-cost observability via SQLite metrics tables; structured logging with tracing; p50/p95/p99 latency calculation; no CloudWatch Metrics cost | W-OBS, W-UI, W-RAG |
| ADR-026 | Code-Level Resilience Patterns — In-memory rate limiting; retry with exponential backoff; circuit breaker for LLM calls; request validation middleware; zero infra cost | W-RES, W-UI, W-LLM |
| ADR-027 | Module-Based Service Decomposition — Logical separation within single Lambda (portfolio, rag, admin, auth modules); independent testing per module; future extraction path to separate Lambdas if needed | W-MOD, W-UI, W-RAG, W-AUTH |
| ADR-028 | Private Cloud MCP Gateway — Cognito-authenticated MCP server on Lambda; API Gateway routing for POST /mcp + GET /mcp/health | W-MCP, W-CI, W-OTF |
| ADR-029 | Dev/Prod Environment Separation with Artifact Promotion — OT workspaces for dev/prod; `just promote` copies artifacts instead of rebuilding; singleton resource sharing (VPC endpoints, OIDC, ACM) | W-PROM, W-CI, W-OTF, W-XT |
| ADR-030 | SaaS AI-DLC Pattern — Six-pillar replicable AI-DLC (onboarding, session lifecycle, anti-rot, RAG, agentic tools, health dashboard); external repo onboarding; eval-driven accuracy loop | W-SAAS, W-RAG, W-MCP, W-AIL, W-LLM |

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
| DRL-2026-05-08-migration-019-upsert | 2026-05-08 | Migration 019 uses banned INSERT OR IGNORE instead of ADR-010 upsert pattern | 1 entry, RESOLVED |
| DRL-2026-04-07-ses-sandbox-ack | 2026-04-07 | SES sandbox blocks ack emails to unverified recipients | **RESOLVED 2026-04-08** — production access granted; W-CTF.4.13 DONE; SES_ACK_FROM_EMAIL restored |
| DRL-2026-04-08-api-openapi-orphan | 2026-04-08 | api-openapi was orphaned from services/ui (W-APIO SSOT) | **RESOLVED 2026-04-08** — SSOT refactor complete; 29 models, dual-spec, 84 tests |
| DRL-2026-05-01-infra-plan-blockers | 2026-05-01 | Three HCL bugs blocked `just infra-plan` (duplicate caller_identity, duplicate file_system_config, missing lifecycle filter) | **RESOLVED 2026-05-01** — all fixed; plan clean |
| DRL-2026-05-02-bootstrap-terraform-docstring | 2026-05-02 | `bootstrap.rs` doc comment still says "terraform init"; LOCK_TABLE named "terraform-lock" | Open — doc-only, low priority |
| DRL-2026-05-02-contact-response-dual-definition | 2026-05-02 | `contact.rs` defines local ChallengeResponse/ContactSubmitRequest/ContactResponse shadowing ADR-012 SSOT models | Open — fix: import api_openapi::models in contact.rs |
| DRL-2026-05-02-openapi-full-spec-public-endpoint | 2026-05-02 | `/api/openapi.json` now serves full spec unauthenticated (intentional); ADR-012 rules 3–5 superseded | Open — update ADR-012 to reflect intentional change |
| DRL-2026-05-02-askama-workspace-orphan | 2026-05-02 | `askama`/`askama_axum` still in workspace deps with no consumers; tsconfig strict claim points to wrong file | Open — remove orphaned deps; update ADR-019 claim |
| DRL-2026-05-03-coverage-floors | 2026-05-03 | 9/10 library crates below coverage floors; `get_crate_coverage` TOTAL line inflated by workspace deps | **RESOLVED 2026-05-03** — tests added to all 9 crates; `get_crate_coverage` rewritten for per-file aggregation; all floors pass |
| DRL-2026-05-03-rustsec-webpki-cves | 2026-05-03 | `cargo audit` failing: RUSTSEC-2026-0098/0099/0104 in `rustls-webpki 0.101.7` (via `aws-sdk-*` default features → `hyper-rustls 0.24` → `rustls 0.21`) | **RESOLVED 2026-05-03** — `default-features = false` on all `aws-sdk-*` workspace deps; only `rustls-webpki 0.103.13` remains |
| DRL-2026-05-04-sislam-outage | 2026-05-04 | sislam.com + dev.sislam.com 404ing — EFS SPA mount never applied; SPA bucket empty | **RESOLVED 2026-05-04** — CF→S3 direct serving; deploy-config SM secret; Lambda SPA code removed; both domains 200 |
| DRL-2026-05-04-adr009-ask-api-scope | 2026-05-04 | ADR-009 claims API GW is for `POST /api/contact` only; `/api/ask` also added; rate limit not enforced | Open — update ADR-009 text; enforce 2-req rate limit in prod |
| DRL-2026-05-04-adr005-askama-claim | 2026-05-04 | ADR-005 rule 2 references Askama, which was removed by ADR-019 | **RESOLVED 2026-05-04** — ADR-005 rule 2 updated with ADR-019 supersession |
| DRL-2026-05-04-adr015-feature-flag-not-implemented | 2026-05-04 | ADR-015 rule 3 claims feature-flag adapter selection; actual uses llm-proxy Lambda | **RESOLVED** — ADR-015 rule 3 updated to describe runtime provider selection via llm-proxy |
| DRL-2026-05-09-rag-challenges-corpus | 2026-05-09 | Challenges 7th corpus undocumented in plan system (W-RAG, W-CHL, ADR-016) | 7 entries; 6 RESOLVED, 1 PENDING (cache refresh) |

---

## Dependency Graph Summary

```
config-core  ←── config-toml, config-yaml, config-json, infra-types (optional), services/ui
api-core     ←── api-openapi, api-graphql, api-grpc, api-merger, services/ui
api-openapi  ←── api-merger, services/ui
api-graphql  ←── api-merger
api-grpc     ←── api-merger
llm-core     ←── llm-anthropic, services/ui (via W-RST), services/llm-proxy (agent loop, ADR-023)
rag-core     ←── rag-sqlite, services/ui (ask handler), xtask (rag commands)
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
| `zero-cost-microservices.md` | Zero-cost microservices patterns — API versioning, observability, resilience, module decomposition |
| `initial-setup.md` | Developer first-run guide (prerequisites, bootstrap, local dev loop) |
| `agentic-execution-steps.md` | Stepped implementation plan for Phases 9–11 (extended corpora → agentic core) |

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
├── services/ui/            # Portfolio site + deployed Lambda binary (with modular structure: modules/, middleware/, telemetry/)
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
| 8 | Quality pass | WIP (`just quality` passes: coverage floors ✅, audit ✅; per-crate READMEs + examples TODO) |
| 9 | Publish | TODO |
| 10 | Zero-cost microservices enhancements | WIP (~~W-VER~~ DONE, ~~W-OBS~~ DONE, ~~W-RES~~ DONE, W-MOD) |

**Overall: ~95% complete**
