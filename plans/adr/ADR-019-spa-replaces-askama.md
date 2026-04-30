# ADR-019: React/Vite SPA Replaces Server-Side Askama Templates

**Status:** Accepted
**Date:** 2026-04-30
**Affected modules:** W-WEB, W-UI, W-AUTH, W-ABT, W-SL, W-RSM, W-RAG, W-OTF, W-CI

---

## Context

The portfolio UI is currently rendered server-side using Askama (Rust template engine) with Tailwind CSS loaded via CDN script tag and small amounts of vanilla JS. There are 15 templates:

**Public (marketing):** `base.html`, `resume.html`, `about_me.html`, `about_repo.html`, `ask.html`, `contact.html`
**Dashboard (auth-gated CRUD):** `dashboard_home.html`, `dashboard_jobs_list.html`, `dashboard_job_detail.html`, `dashboard_competencies_list.html`, `dashboard_competency_detail.html`, `dashboard_about_list.html`, `dashboard_about_detail.html`, `dashboard_social_links_list.html`, `dashboard_social_link_detail.html`

Pain points with the current approach:
- No typed API contract between the browser and `services/ui` — frontend state is baked into Rust template structs.
- Tailwind CDN is a development-only tool not suitable for production (no purging, slow load).
- The `/ask` RAG chat UI requires increasingly complex client-side state; vanilla JS is becoming unwieldy.
- The dashboard CRUD forms cannot intercept API errors cleanly without a framework.
- Maintaining two mental models (Rust template struct + browser DOM) for every entity slows development.

Reference implementation: `~/njnewsroomproject/web/` (Vite 6 + React 18 + TypeScript strict, fully operational).

---

## Decision

**Replace all 15 Askama templates with a Vite 6 + React 18 + TypeScript strict SPA in `web/`.** The Rust Lambda (`services/ui`) becomes a pure **JSON API server + static asset host**. Askama and `askama_axum` are removed from `services/ui/Cargo.toml`.

### Stack

- Vite 6 — build tool with HMR, content-hash output, dev proxy to `:3000`.
- React 18 — UI framework.
- React Router 6 — client-side routing.
- TypeScript strict — `"strict": true` in `tsconfig.json`.
- Tailwind CSS 3 via PostCSS plugin (replaces CDN).
- `react-helmet-async` — `<title>` and meta per route.
- `openapi-fetch` + `openapi-typescript` — typed HTTP client auto-generated from `crates/api-openapi`'s `/api/openapi.json` (respects ADR-012 OpenAPI SSOT).
- Vitest + Testing Library — unit + smoke tests.

### Route mapping (Askama → React)

| Old Askama template | New React route | Data source |
|---|---|---|
| `resume.html` (/) | `routes/Home.tsx` | `GET /api/resume` (new endpoint) |
| `about_me.html` | `routes/AboutMe.tsx` | `GET /api/about/sections` (new endpoint) |
| `about_repo.html` | `routes/AboutRepo.tsx` | `GET /api/about/repo/snapshot` (new endpoint) |
| `ask.html` | `routes/Ask.tsx` | `POST /api/ask` (existing) |
| `contact.html` | `routes/Contact.tsx` | `POST /api/contact` + challenge flow (existing) |
| `dashboard_home.html` | `routes/dashboard/Home.tsx` | `GET /api/admin/*` aggregate counts (existing) |
| `dashboard_jobs_list.html` | `routes/dashboard/Jobs.tsx` | `GET /api/admin/jobs` (existing) |
| `dashboard_job_detail.html` | `routes/dashboard/JobDetail.tsx` | `GET /api/admin/jobs/:slug` (existing) |
| `dashboard_competencies_list.html` | `routes/dashboard/Competencies.tsx` | `GET /api/admin/competencies` (existing) |
| `dashboard_competency_detail.html` | `routes/dashboard/CompetencyDetail.tsx` | `GET /api/admin/competencies/:slug` |
| `dashboard_about_list.html` | `routes/dashboard/About.tsx` | `GET /api/admin/about` |
| `dashboard_about_detail.html` | `routes/dashboard/AboutDetail.tsx` | `GET /api/admin/about/:slug` |
| `dashboard_social_links_list.html` | `routes/dashboard/SocialLinks.tsx` | `GET /api/admin/social-links` |
| `dashboard_social_link_detail.html` | `routes/dashboard/SocialLinkDetail.tsx` | `GET /api/admin/social-links/:id` |

### New JSON endpoints required (Phase D.1)

These public endpoints do not yet exist and must be added before the corresponding React routes ship:
- `GET /api/resume` — combined `{ bio, jobs, competencies, social_links }`.
- `GET /api/about/sections` — list of `AboutSection` rows (used by /about/me and /about/repo).
- `GET /api/about/repo/snapshot` — repo capability/stack data (currently inlined in `about_repo.html`).
- `GET /api/social-links` — list of `SocialLink` rows for nav rendering.
- `GET /api/auth/me` — `{ authenticated: bool, email: Option<String> }` from Cognito session cookie; used by SPA auth gate.

Each new endpoint must follow ADR-012 (struct defined in `crates/api-openapi/src/models/`, registered in `ALL_MODELS`).

### Axum router change (Phase D.4)

The Axum router simplifies to: JSON API routes + auth redirect handlers + `ServeDir` catch-all.

```rust
// Catch-all SPA fallback: hashed assets get long-cache, all other paths → index.html
.fallback_service(
    ServeDir::new(&state.spa_root)
        .fallback(ServeFile::new(state.spa_root.join("index.html")))
)
```

`SPA_ROOT` env var: `web/dist` locally, `/mnt/spa/active` on Lambda.

### SPA delivery (Lambda + EFS, Phase D.4)

Reuses the SPA-from-EFS pattern from njnewsroomproject ADR-003:
1. CI builds `web/dist/` and syncs to `s3://deploy-baba-{env}-spa-{acct}/${SHA}/`.
2. CI invokes Lambda with `{"action":"sync-spa","sha":"${SHA}"}`.
3. Lambda sync handler copies S3 → `/mnt/spa/${SHA}/` and atomically swaps the `active` symlink.

### Auth flows stay server-side

`/auth/login`, `/auth/callback`, `/auth/logout`, `/auth/set-session` remain Rust handlers. They perform Cognito OAuth redirects and set the HttpOnly cookie; they do not render HTML.

### SEO trade-off

The initial implementation is pure CSR (no server-side rendering). Marketing routes (`/`, `/about/me`, `/about/repo`, `/resume`, `/contact`) will not be pre-rendered on day one.

**Mitigation (P3, W-WEB.5):** Build-time prerender via `vite-plugin-prerender-spa` or React Router v7 framework-mode prerender. CI step before Vite build: invoke Lambda `{action:"export-content"}` to fetch live SQLite content into `web/src/data/snapshot.ts`. Vite prerenders marketing routes using the snapshot. Each deploy regenerates the snapshot, so content stays current at deploy frequency. Crawlers see fully-rendered HTML; users get hydration + live refetch from `/api/*`.

---

## Consequences

**Positive:**
- Single stack (React + TypeScript) for all UI development. No more Rust template struct / DOM duality.
- Typed API client from `crates/api-openapi` — breaking API changes caught at `pnpm run typecheck`.
- Tailwind PostCSS build — purged CSS, no CDN dependency.
- The `/ask` chat UI gains `react-markdown`, proper loading states, and citation components.
- Dashboard CRUD gains error handling, optimistic updates, and typed form validation.
- Lambda binary shrinks: Askama + template compilation removed; `aws-sdk-s3` added for sync handler.

**Negative:**
- Large diff: ~50 new files in `web/`, Askama templates deleted, 15 route handlers simplified.
- Pure CSR on day one means marketing pages are not pre-rendered for crawlers. Mitigation: W-WEB.5 (P3).
- Adds `pnpm` and Node 20 as dev dependencies. Mitigation: devcontainer + `scripts/dev-doctor.sh`.
- Existing `/auth/callback` and Cognito cookie set-session must be carefully preserved through the router refactor.

---

## Migration strategy

Each sub-phase (D.1–D.5) merges to `main` independently, deploys to dev via `deploy-dev.yml`, soaks, then promotes to prod via `just release-promote`. The highest-risk cutover (D.4: flip the Axum router catch-all) runs in dev for ≥24h before promotion.

Askama templates are not deleted until D.5, after all React routes are verified in dev.

---

## Cross-References
- → ADR-001 (justfile-only interface — `just web*` recipes added)
- → ADR-003 (Lambda Function URL — no API Gateway for SPA)
- → ADR-004 (dual-mode entry point — SPA_ROOT env var)
- → ADR-005 (zero-cost philosophy — SPA build in S3 + EFS, no CDN)
- → ADR-007 (OpenTofu — `infra/s3-spa.tf` added, `infra/lambda.tf` updated)
- → ADR-012 (OpenAPI SSOT — openapi-fetch types generated from api-openapi spec)
- → ADR-013 (Dashboard dark theme — ported from Tailwind CDN classes to PostCSS config)
- → ADR-020 (CI OIDC — deploy-dev/deploy-prod workflows build + sync SPA)
- → `plans/modules/web.md` (W-WEB work items)
- → `plans/cross-cutting/initial-setup.md` (pnpm prerequisite)
