# ADR-012: OpenAPI SSOT + Public/Admin Spec Split

**Date:** 2026-04-08
**Status:** Accepted
**Affected modules:** W-APIO, W-UI

## Context

`crates/api-openapi/` was designed as the single source of truth for all API data models
and the OpenAPI specification. In practice it was orphaned: `services/ui` defined all
request/response structs locally and assembled the spec via a hand-maintained `paths(...)`
list in `services/ui/src/openapi.rs`. The two layers had drifted — `HealthResponse` and
`CrateInfo` were missing `ToSchema` derives, admin-tagged paths were not filtered out of the
public spec, and there was no enforcement mechanism to prevent new structs from being added
to `services/ui/src/routes/` without registration.

The admin dashboard (`/dashboard`) was also live but its corresponding OpenAPI paths had no
security schemes, making the spec misleading to clients.

## Decision

> All API data models live exclusively in `crates/api-openapi/src/models/`. No
> request/response struct may be defined in `services/ui`. Two separate OpenAPI specs
> are served: a public-filtered spec and a full admin spec.

Specific rules:
1. Every public API struct implements `ApiModel` (compile-time: `schema_name()` + `example()`).
2. `ALL_MODELS` const in `registry.rs` lists every model; CI fails if count drifts from
   `full_spec().components.schemas`.
3. `public_view(spec)` strips admin-tagged operations, GCs unreferenced schemas, and removes
   `cookieAuth`/`bearerAuth` security schemes.
4. `GET /api/openapi.json` → filtered public spec (no auth required).
5. `GET /api/openapi-admin.json` → full spec behind `require_auth`.
6. A source-scan e2e test (`tests/e2e/coverage.rs`) fails if any serialisable `pub struct`
   in `services/ui/src/routes/` is not in `ALL_MODELS` or `ALLOWED_LOCAL`.

## Consequences

### Positive
- Single location for all model definitions — no drift between crate and service.
- Compile-time guarantee: removing a registered model breaks `cargo build`.
- CI guarantee: adding a route struct without registering it fails the coverage test.
- Public API clients never see admin paths or auth schemes.
- `HealthResponse` and `CrateInfo` fix: both now derive `ToSchema` (latent bug resolved).

### Negative / Trade-offs
- `utoipa-axum` router migration (eliminates hand-maintained `paths(...)`) is deferred:
  `utoipa-axum = "0.1"` depends on utoipa v5, workspace uses v4. Tracked as W-APIO.4.4.
- Schema GC in `public_view()` is path-driven. When called on a schema-only spec (no
  handler paths registered), GC removes all schemas — tests must target `full_spec()` or
  `PublicApiDoc::openapi()` directly, not `public_view(full_spec())`, for schema-presence
  assertions.

### Neutral
- `services/ui/src/openapi.rs` retains the hand-maintained `paths(...)` list until
  utoipa-axum migration lands (W-APIO.4.4 DEFERRED).
- `lib.rs` preserves `OpenApiGenerator`/`OpenApiSchema`/`merge_openapi_specs` for
  `api-merger` and examples.

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Keep structs in services/ui, use `#[derive(OpenApi)]` re-exports | Defeats SSOT; structs remain scattered |
| Single `ApiDoc` with client-side filtering | Leaks admin schema names to public clients |
| utoipa-axum `OpenApiRouter` (eliminates paths list) | Requires utoipa v5 upgrade; deferred (W-APIO.4.4) |
| `jsonschema` example validation tests | Added as dev-dep; deferred to a follow-up test pass |

## Cross-References

- → W-APIO (primary affected module)
- → W-UI (`services/ui` imports models, serves dual-spec)
- → DRL-2026-04-08-api-openapi-orphan (orphan finding that triggered this ADR)
- → ADR-008 (Cognito auth — `require_auth` gates admin spec endpoint)
