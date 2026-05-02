# W-APIO: api-openapi
**Crate:** `crates/api-openapi/` | **Status:** DONE (SSOT refactor complete 2026-04-08)
**Coverage floor:** 80% | **Depends on:** W-API | **Depended on by:** W-APIM, W-UI

---

## W-APIO.1 Purpose

`api-openapi` is the **single source of truth (SSOT)** for all API data models and the
OpenAPI specification for the deploy-baba portfolio service. It provides:

- **`models/`** — every public/admin request/response struct with `ToSchema` + `ApiModel`.
- **`registry.rs`** — compile-time checked `ALL_MODELS` list (drives coverage tests).
- **`apidoc.rs`** — `PublicApiDoc`, `AdminApiDoc`, and `full_spec()` for router init.
- **`filter.rs`** — `public_view()` strips admin paths + unreferenced schemas.
- Original `OpenApiGenerator` / `OpenApiSchema` / `merge_openapi_specs` preserved for
  use by `api-merger` and examples.

`services/ui` imports all body types from this crate instead of defining them locally.

---

## W-APIO.2 Model Architecture

### `ApiModel` trait (compile-time enforcement)

```rust
pub trait ApiModel: Serialize + DeserializeOwned + 'static {
    fn schema_name() -> &'static str;
    fn example() -> Self;
}
```

Every struct used as a request/response body must implement `ApiModel`. This is enforced:
- **Compile time:** types in `ALL_MODELS` must implement `Serialize` → `cargo build` fails
  if you remove a registered model.
- **Test time (CI):** `schema_coverage` test fails if a model is not in `full_spec()`.
- **Source scan (CI):** `ui_coverage` test fails if a new `pub struct` appears in
  `services/ui/src/routes/` without being registered in `api_openapi::models`.

### Dual-spec endpoints

| Endpoint | Spec | Auth |
|----------|------|------|
| `GET /api/openapi.json` | Public filtered (no admin paths, no security schemes) | None |
| `GET /api/openapi-admin.json` | Full spec with admin paths + cookieAuth/bearerAuth | `require_auth` |
| `GET /docs` | Rapidoc → public spec | None |
| `GET /docs/admin` | Rapidoc → admin spec | None |

### Public filter

`api_openapi::filter::public_view(spec)`:
1. Drops any operation tagged `"admin"`.
2. Drops `PathItem`s with no remaining operations.
3. GC's `components.schemas` — removes entries not reachable from remaining operations.
4. Strips `cookieAuth` / `bearerAuth` security schemes.
5. Clears top-level `security` field.

---

## W-APIO.3 Module Layout

```
crates/api-openapi/src/
├── lib.rs           — OpenApiGenerator / OpenApiSchema / merge_openapi_specs (preserved)
├── models/
│   ├── mod.rs       — ApiModel trait + _assert_model() + pub use all
│   ├── common.rs    — ApiError
│   ├── health.rs    — HealthResponse
│   ├── crates.rs    — CrateInfo
│   ├── stack.rs     — (empty; GET /api/stack returns serde_json::Value)
│   ├── demo.rs      — ParseConfigRequest/Response, GenerateSpecRequest/Response, Field
│   ├── resume.rs    — Job, JobDetail, JobWithDetails, JobsQuery, Competency,
│   │                  EvidenceItem, CompetencyWithEvidence
│   ├── about.rs     — AboutSectionInput, AboutSectionResponse
│   ├── social.rs    — SocialLink, SocialLinkInput, SocialLinkResponse
│   ├── contact.rs   — ChallengeResponse, ContactSubmitRequest, ContactResponse
│   └── admin.rs     — JobInput, JobDetailInput, CompetencyInput, EvidenceInput, Evidence
├── registry.rs      — ALL_MODELS compile-time registry (fn() → serde_json::Value)
├── apidoc.rs        — PublicApiDoc, AdminApiDoc, SecurityAddon, full_spec()
└── filter.rs        — public_view(spec) → filtered OpenApi
```

---

## W-APIO.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-APIO.4.1 | Per-crate README.md | TODO | W-DX.3 dependency |
| W-APIO.4.2 | Models SSOT — all body types in models/ | **DONE 2026-04-08** | 29 models, ApiModel trait |
| W-APIO.4.3 | Public filter — strip admin paths + GC schemas | **DONE 2026-04-08** | filter.rs |
| W-APIO.4.4 | utoipa-axum router migration (eliminate paths list) | DEFERRED | utoipa-axum requires utoipa v5; workspace is v4 |
| W-APIO.4.5 | Coverage tests — three-tier taxonomy | **DONE 2026-04-08** | unit (inline #[cfg(test)]), integration (per-API), e2e (cross-cutting) |

---

## W-APIO.5 Test Strategy

Three-tier taxonomy (84 tests total, all green as of 2026-04-08):

### Unit tests — inline `#[cfg(test)]` in `src/`
- `src/registry.rs` — no duplicate names, all examples serialize, serde roundtrip for every model
- `src/filter.rs` — admin op detection, path filtering, security clearing

### Integration tests — `tests/integration.rs` + `tests/integration/`
Per-API modules: `health`, `jobs`, `competencies`, `contact`, `admin`, `about`, `social`, `demo`, `registry`.
Each verifies: schemas present in `full_spec()`, model examples have correct field values,
`ApiModel::schema_name()` matches registry, admin schemas absent from public spec via `AdminApiDoc`/`PublicApiDoc`.

### E2e tests — `tests/e2e.rs` + `tests/e2e/`
Cross-cutting: `public_spec`, `admin_spec`, `coverage`.
- `public_spec`: no cookieAuth/bearerAuth in filtered spec, no top-level security, no admin schemas
- `admin_spec`: full spec has both security schemes, all admin schemas, correct title/version
- `coverage`: source-scan services/ui routes for unregistered `pub struct` (cross-crate enforcement)

**Note on GC behaviour:** `public_view(full_spec())` removes ALL schemas when `full_spec()` has
no path operations (schema-only spec). The filter's schema GC is path-driven and is tested
at the unit level (`filter.rs`). Schema-survival assertions target `full_spec()` / `PublicApiDoc`
directly (not the filtered view) since `api-openapi` only stores schemas, not handler paths.

---

## W-APIO.6 Cross-References

- → W-API (ApiSpec trait)
- ← W-APIM (merged alongside graphql/grpc)
- ← W-UI (live demo endpoint + OpenAPI spec assembly)
- → `plans/cross-cutting/dependency-graph.md`
- → `plans/drift/DRL-2026-04-08-api-openapi-orphan.md` (orphan finding)
- → `plans/adr/ADR-012-openapi-ssot.md` (ADR for this refactor)
- → ADR-015 (LLM provider abstraction — LLM request/response models may extend api-openapi)
