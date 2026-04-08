# DRL-2026-04-08: api-openapi Orphaned from services/ui

**Date:** 2026-04-08
**Status:** RESOLVED 2026-04-08 (W-APIO SSOT refactor complete)
**Affected modules:** W-APIO, W-UI

## Finding

During the W-APIO SSOT refactor, `crates/api-openapi/` was discovered to be orphaned from
`services/ui`. Specifically:

1. **`services/ui` defined all request/response structs locally** — `Job`, `JobDetail`,
   `Competency`, `ChallengeResponse`, `AboutSectionInput`, `SocialLink`, `SocialLinkInput`,
   etc. were defined in `services/ui/src/routes/api/*.rs` and `services/ui/src/db.rs`,
   not in the SSOT crate.

2. **`HealthResponse` and `CrateInfo` were missing `ToSchema`** — they were referenced as
   `body = HealthResponse` in `#[utoipa::path]` attributes but did not derive `ToSchema`,
   making the OpenAPI schema references silently invalid.

3. **Admin paths had no security schemes** — the 15+ paths under `/api/admin/` were
   documented in the spec without `cookieAuth`/`bearerAuth`, misleading API clients into
   thinking the endpoints were unauthenticated.

4. **No public/admin spec split** — a single spec was served at `/api/openapi.json`
   containing both public and admin paths. External clients could enumerate admin endpoints.

5. **No enforcement** — nothing prevented developers from adding new route structs to
   `services/ui/src/routes/` without registering them in `api-openapi`.

## Root Cause

The original plan described `api-openapi` as the spec generator, but the actual router
wiring in `services/ui/src/router.rs` used `ApiDoc::openapi()` (defined entirely within
`services/ui/src/openapi.rs`). The two layers were never connected. The crate's
`OpenApiGenerator` / `OpenApiSchema` traits were used only in examples and `api-merger`.

## Resolution

See ADR-012 for the full decision. Summary of changes:

- All 29 models moved to `crates/api-openapi/src/models/` with `ApiModel` trait.
- `ALL_MODELS` const registry in `registry.rs` (compile-time + test-time enforcement).
- `PublicApiDoc` + `AdminApiDoc` + `full_spec()` in `apidoc.rs`.
- `public_view()` filter in `filter.rs`.
- `services/ui/src/db.rs` `SocialLink` replaced with `pub use api_openapi::models::SocialLink`.
- All `services/ui/src/routes/api/*.rs` now import from `api_openapi::models::*`.
- Dual spec served: `/api/openapi.json` (public) + `/api/openapi-admin.json` (auth-gated).
- 84 tests across three tiers: unit (inline), integration (per-API), e2e (cross-cutting).

## Cross-References

- → ADR-012 (decision record for SSOT + public/admin split)
- → W-APIO.4.2–4.5 (work items completed)
