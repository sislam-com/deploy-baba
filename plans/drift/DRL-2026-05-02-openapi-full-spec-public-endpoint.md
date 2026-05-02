# DRL-2026-05-02-openapi-full-spec-public-endpoint

**ADR:** ADR-012 | **Detected:** 2026-05-02 | **Severity:** Intentional design change

## Context

ADR-012 rules 3–5 originally specified:
- (3) `public_view(spec)` strips admin-tagged operations and security schemes for the public endpoint
- (4) `GET /api/openapi.json` serves the filtered public spec (unauthenticated)
- (5) `GET /api/openapi-admin.json` serves the full spec behind `require_auth`

## Change Made (2026-05-02)

Session intentionally replaced this with a single unified spec served on both endpoints, both unauthenticated (`services/ui/src/router.rs`). The `/docs` page now shows all endpoints (public and admin) with security annotations (lock icons in RapiDoc). The `require_auth` middleware on the actual `/api/admin/*` routes is unchanged — only the spec visibility changed.

Motivation: developer/consumer experience. The public docs link now surfaces the full API surface so integrators can see what admin endpoints exist and understand the full contract, while the runtime auth enforcement remains intact.

## Status

**Accepted divergence** — ADR-012 should be updated to reflect this intentional change rather than tracking it as drift. Specifically, rules 3–5 should be revised to: "A single full spec is served at `/api/openapi.json` (unauthenticated). Admin paths carry `security` annotations. `/api/openapi-admin.json` is an alias. The `public_view()` filter is retired."

## Action Required

Update ADR-012 Status section to note this change. This DRL can then be closed.
