# W-VER: api-versioning
**Path:** `services/ui/src/middleware/`, `services/ui/src/router.rs` | **Status:** TODO
**Coverage floor:** 80% | **Depends on:** W-UI, W-APIO | **Depended on by:** W-OBS, W-RES

---

## W-VER.1 Purpose

URL-based API versioning strategy using Lambda Function URL routing (no API Gateway). Enables breaking changes while maintaining backward compatibility through versioned endpoints. Follows zero-cost principles by using code-level routing instead of infrastructure solutions.

→ ADR-024

---

## W-VER.2 Public API Surface

### Middleware Functions

```rust
// services/ui/src/middleware/version.rs
pub struct ApiVersion {
    pub major: u8,
    pub minor: u8,
}

pub async fn extract_api_version(
    uri: &Uri,
) -> Result<ApiVersion, ApiError>
```

### Router Structure

```rust
// services/ui/src/router.rs
Router::new()
    .nest("/api/v1", routes::api::v1::router())
    .nest("/api/v2", routes::api::v2::router())  // Future versions
    .fallback(api_version_handler)
```

### Response Headers

```rust
// Deprecation headers for sunset versions
X-API-Deprecated: true
Sunset: 2027-01-01
X-API-Replacement: /api/v2/jobs
```

---

## W-VER.3 Implementation Notes

### Version Extraction Logic

Parse version from URL path `/api/v1/...` → `ApiVersion { major: 1, minor: 0 }`

```rust
let version = path
    .split('/')
    .nth(2)  // /api/v1/...
    .and_then(|v| v.strip_prefix('v'))
    .ok_or(ApiError::InvalidVersion)?;
```

### Deprecation Middleware

```rust
pub async fn deprecation_middleware(
    version: ApiVersion,
    req: Request,
    next: Next,
) -> Response {
    if version.major < 2 {
        let mut resp = next.run(req).await;
        resp.headers_mut().insert("X-API-Deprecated", "true".parse().unwrap());
        resp.headers_mut().insert("Sunset", "2027-01-01".parse().unwrap());
        resp
    } else {
        next.run(req).await
    }
}
```

### OpenAPI Version Metadata

```rust
// services/ui/src/openapi.rs
struct ApiVersionModifier;
impl Modify for ApiVersionModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.version = "1.0.0";
        openapi
            .info
            .extensions
            .insert("x-api-deprecation".to_string(), serde_json::json!({
                "v1": "2027-01-01",
                "v2": null
            }));
    }
}
```

### Migration Path

- **Phase 1**: Add version extraction middleware (no routing changes)
- **Phase 2**: Migrate existing routes to `/api/v1/` with redirects from `/api/`
- **Phase 3**: Add `/api/v2/` for breaking changes
- **Phase 4**: Sunset `/api/v1/` after deprecation period

---

## W-VER.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-VER.4.1 | Create version extraction middleware | TODO | Parse `ApiVersion` from URL path, error handling |
| W-VER.4.2 | Add deprecation headers middleware | TODO | Sunset headers, replacement endpoint headers |
| W-VER.4.3 | Update router structure for version nesting | TODO | `/api/v1/` nesting, fallback handler |
| W-VER.4.4 | Add OpenAPI version metadata | TODO | ApiVersionModifier, deprecation schedule in spec |

---

## W-VER.5 Test Strategy

- Unit tests for version extraction logic (valid/invalid versions)
- Integration tests for deprecation headers on sunset versions
- OpenAPI spec validation includes version metadata
- Test coverage floor: 80%

---

## W-VER.6 Cross-References

- → ADR-024 (API Versioning Strategy)
- → ADR-003 (Lambda Function URL — no API Gateway)
- → W-UI (router structure)
- → W-APIO (OpenAPI spec generation)
- → W-OBS (version-specific metrics)
- → W-RES (version-aware rate limiting)
