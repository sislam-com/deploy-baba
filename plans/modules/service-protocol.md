# W-SVP: Service Protocol

## Domain code: `W-SVP`

Typed inter-service request/response protocol for Lambda microservices. Defines the contract between the api-gateway routing Lambda and all backend service Lambdas.

---

## Status

| Aspect | Status |
|--------|--------|
| Protocol types (ServiceRequest, ServiceResponse, AuthContext) | **DONE** |
| TargetService enum with path mapping | **DONE** |
| Lambda name generation | **DONE** |
| Unit tests for round-trip serialization | **DONE** |
| Axum request conversion (gated behind `axum` feature) | **DONE** |
| `ServiceRouter` trait for backend handlers | **DONE** |

---

## Location

- Crate: `crates/service-protocol/`
- ADR: `plans/adr/ADR-031-lambda-microservices-architecture.md`

---

## Design

### Types

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

pub struct AuthContext {
    pub sub: String,
    pub email: String,
    pub groups: Vec<String>,
}
```

### Routing

`TargetService::from_path()` maps URL paths to backend services:

| Path prefix | TargetService |
|-------------|---------------|
| `/api/v1/portfolio/*` | `Portfolio` |
| `/api/v1/rag/*`, `/api/ask` | `Rag` |
| `/api/v1/admin/*` | `Admin` |
| `/api/v1/auth/*`, `/auth/*` | `Auth` |
| `/api/contact/*` | `Contact` |
| `/api/v1/agent/*` | `Agent` |
| `/api/v1/metrics` | `Metrics` (inline) |
| `/health` | `Health` (inline) |

### Lambda naming

`TargetService::lambda_name(project, env)` produces `deploy-baba-{env}-{service}`:
- portfolio: `deploy-baba-prod-portfolio`
- rag: `deploy-baba-prod-rag`
- admin: `deploy-baba-prod-admin`
- auth: `deploy-baba-prod-auth`
- contact: `deploy-baba-prod-contact`
- agent: `deploy-baba-prod-agent`

### Why not gRPC / Protobuf

- Simpler: serde_json is already a workspace dependency
- No code generation step
- Human-readable payloads for debugging in CloudWatch
- No additional dependencies

### Why not HTTP Function URLs for internal routing

- SDK invoke avoids HTTP handshake + DNS (lower latency)
- No public endpoints for internal services (security)
- No data transfer out charges
- Consistent with existing email/llm-proxy invocation pattern

---

## Remaining work

- [ ] Add `x-request-id` correlation ID to `ServiceRequest` headers (W-SVP.4.1)
- [ ] Add `ServiceRequest::from_lambda_event()` for backend deserialization (W-SVP.4.2)
- [ ] Add `ServiceResponse::into_lambda_response()` for backend serialization (W-SVP.4.3)
- [ ] Property-based tests for arbitrary `ServiceRequest` → serde → equality (W-SVP.4.4)
