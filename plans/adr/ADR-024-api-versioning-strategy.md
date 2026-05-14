# ADR-024: API Versioning Strategy

**Date:** 2026-05-14
**Status:** Proposed
**Affected modules:** W-VER, W-UI, W-APIO, W-OBS

## Context

The deploy-baba API currently has no versioning strategy. All endpoints live under `/api/` with no version identifiers. This makes breaking changes risky and unclear to API consumers. Traditional API versioning approaches using API Gateway versioning or header-based versioning introduce additional infrastructure cost and complexity.

Constraints:
- Must not introduce API Gateway (ADR-003: Lambda Function URL only)
- Must follow zero-cost philosophy (ADR-005: no additional infrastructure)
- Must maintain backward compatibility for existing clients
- Must provide clear deprecation timeline for sunset versions
- Must integrate with existing OpenAPI spec (ADR-012)

## Decision

> We will implement URL-based API versioning using code-level routing in the Lambda service. Version identifiers in the URL path (`/api/v1/...`, `/api/v2/...`) enable breaking changes while maintaining backward compatibility. Deprecation headers communicate sunset timelines without infrastructure changes.

Specific rules:
1. **URL-based versioning**: `/api/v1/jobs`, `/api/v2/jobs` — version in path, not headers
2. **Deprecation headers**: Sunset versions return `X-API-Deprecated: true`, `Sunset: date`, `X-API-Replacement: /api/v2/jobs`
3. **Code-level routing**: Version extraction via middleware, no API Gateway configuration
4. **OpenAPI metadata**: Version info in OpenAPI spec extensions (`x-api-deprecation`)
5. **Migration path**: Existing `/api/` endpoints redirect to `/api/v1/` initially
6. **Deprecation timeline**: Minimum 6 months notice before sunset

## Consequences

### Positive
- Zero infrastructure cost (pure code implementation)
- Clear version identifiers in URL (self-documenting)
- Backward compatibility through version coexistence
- Deprecation headers provide clear migration path
- OpenAPI spec includes version metadata
- Future extraction to separate Lambdas maintains version structure

### Negative / Trade-offs
- URL path changes required for breaking changes (client updates needed)
- Version extraction middleware adds request processing overhead
- Multiple route versions increase codebase size
- Deprecation enforcement is client-honor (not enforced by infrastructure)

### Neutral
- Initial migration adds `/api/v1/` nesting to all existing routes
- Version-specific rate limits can be configured per version
- Metrics can be analyzed by version (W-OBS integration)
- Future v2 endpoints can have different data structures

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Header-based versioning (`Accept: application/vnd.api.v1+json`) | Less discoverable, harder to debug in browser |
| API Gateway stage variables | Violates ADR-003 (no API Gateway), adds cost |
| Query parameter versioning (`?version=1`) | Non-RESTful, caching issues |
| Single version with backward-compatible extensions | Accumulates technical debt, unclear deprecation path |
| Date-based versioning (`/api/2024-05-14/jobs`) | Implies temporal stability that may not exist |

## Cross-References

- → W-VER (implementation module)
- → ADR-003 (Lambda Function URL — code-level routing)
- → ADR-005 (Zero-cost philosophy — no infra cost)
- → ADR-012 (OpenAPI SSOT — version metadata in spec)
- → W-UI (router structure updates)
- → W-APIO (OpenAPI spec modifications)
- → W-OBS (version-specific metrics)
- → W-RES (version-aware rate limiting)
