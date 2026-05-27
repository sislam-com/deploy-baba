# ADR-031: Lambda Microservices Architecture

**Date:** 2026-05-23
**Status:** Accepted
**Affected modules:** W-MOD, W-UI, W-RAG, W-AUTH, W-CTF, W-OBS, W-RES

## Context

ADR-027 established logical module boundaries within the monolithic `services/ui/` Lambda. Since then, three satellite Lambdas (`email`, `llm-proxy`, `mcp-gateway`) have proven that multi-service deployment works well for the portfolio project. The time has come to extract the logical modules into actual separate Lambda services while maintaining the zero-cost philosophy.

Current state:
- Single `services/ui/` Lambda handles all API routes (portfolio, admin, RAG, auth, contact, metrics, SPA serving)
- Three standalone Lambdas (`email`, `llm-proxy`, `mcp-gateway`) already exist
- `services/auth/` was extracted first as the proof-of-concept standalone service
- All services share a single SQLite database on EFS

Constraints:
- Must follow zero-cost philosophy: no new infrastructure unless justified
- Must maintain existing SPA → CloudFront → Lambda flow
- Must not increase cold-start latency for common requests
- Must preserve existing CI/CD pipeline patterns
- Must keep the shared SQLite on EFS as the single source of truth

## Decision

> We will transform `services/ui/` into a lightweight `api-gateway` Lambda that routes SPA requests to backend service Lambdas via AWS Lambda SDK invoke. Each backend service is a standalone binary in `services/` mounted to the shared EFS. Inter-service communication uses direct Lambda SDK invocation (synchronous, typed payloads) rather than HTTP Function URLs or async events.

### Service topology

```
CloudFront (OAC)
  ├── /assets/*       → S3 SPA bucket
  ├── /             → S3 index.html
  └── /api/v1/*     → api-gateway Lambda Function URL
        ├── /portfolio/*  → invoke portfolio Lambda
        ├── /rag/*        → invoke rag Lambda
        ├── /admin/*      → invoke admin Lambda
        ├── /auth/*       → invoke auth Lambda
        ├── /contact      → invoke contact Lambda
        ├── /metrics      → inline (reads shared SQLite)
        └── /health       → inline

Backend Lambdas (internal only, no Function URLs):
  services/portfolio/  → jobs, competencies, about, social-links, resume, challenges
  services/rag/        → /api/ask, retrieval, grounding
  services/admin/      → dashboard CRUD for all entities
  services/auth/       → Cognito JWKS validation, session (already extracted)
  services/contact/    → PoW validation + email Lambda delegation

Satellite Lambdas (existing):
  services/email/      → SES send (invoked by contact Lambda)
  services/llm-proxy/  → LLM provider abstraction (invoked by rag Lambda)
  services/mcp-gateway/ → Cognito-authenticated MCP server
```

### Inter-service protocol

**Lambda SDK invoke** with typed `ServiceRequest`/`ServiceResponse` payloads.

Why not Function URLs:
- Cold start: SDK invoke avoids HTTP handshake + DNS (lower latency)
- Security: no public endpoints for internal services
- Cost: no data transfer out to internet
- Consistency: matches existing email/llm-proxy invocation pattern

Protocol types (in `crates/service-protocol/`):

```rust
pub struct ServiceRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<String>,
    pub auth_context: Option<AuthContext>,
}

pub struct ServiceResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}
```

Backend Lambdas use `lambda_runtime` directly (not `lambda_http`) — they deserialize `ServiceRequest` from the event payload, route internally, and return `ServiceResponse`.

### Shared data / EFS

**Single EFS access point, single SQLite file, per-service write conventions.**

Rationale: zero-cost (no new EFS, no DynamoDB). The `admin` Lambda owns migrations. Each service reads from all tables but only writes to its domain tables.

| Service | Read | Write | Notes |
|---------|------|-------|-------|
| portfolio | all portfolio tables | `jobs`, `competencies`, `about_sections`, `social_links`, `resume_*`, `challenges` | reference data |
| rag | `rag_*` tables | `rag_queries` (query log) | reads portfolio tables for context |
| admin | all tables | all tables | owns migrations |
| auth | none | none | validates JWTs only |
| contact | none | `contact_submissions` | invokes email Lambda |
| api-gateway | `api_metrics` | `api_metrics` | metrics middleware |

### Incremental extraction strategy

Services will be extracted one at a time:
1. Phase 1: `service-protocol` crate (types + routing logic)
2. Phase 2: Extract `portfolio` service as proof-of-concept
3. Phase 3: Verify end-to-end (SPA → api-gateway → portfolio Lambda)
4. Phase 4: Extract remaining services (`rag`, `admin`, `contact`)
5. Phase 5: Full infra deployment with new Lambda resources

This minimizes blast radius and allows validation at each step.

## Consequences

**Positive:**
- Clear service boundaries enforced by compilation units
- Independent deployability per service
- Per-service scaling, memory, and timeout configuration
- Better fault isolation (one buggy service doesn't crash the whole API)
- Enables per-service observability and cost tracking
- Aligns with ADR-027's future extraction path

**Negative:**
- More Lambda functions to manage (higher operational complexity)
- Cold-start latency for routed requests (SDK invoke + Lambda cold start)
- Need to maintain service protocol compatibility across deploys
- Circuit breaker logic needed in api-gateway for backend failures

**Neutral:**
- Shared SQLite remains the single source of truth
- CI/CD pipeline extended to build multiple service zips
- Same VPC/EFS/IAM patterns reused for all new services

## Related

- ADR-027: Module-Based Service Decomposition (predecessor, logical separation)
- ADR-003: Lambda Function URL (api-gateway keeps Function URL)
- ADR-002: SQLite on EFS + S3 Backup (shared state continues)
- ADR-029: Dev/Prod Separation (naming: `deploy-baba-{env}-{service}`)
