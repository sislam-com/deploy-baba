# W-AGW: API Gateway

## Domain code: `W-AGW`

The `services/ui/` Lambda transformed into a lightweight routing layer. All business logic handlers move to backend service Lambdas; `services/ui/` only routes requests, applies global middleware, and handles inline endpoints.

---

## Status

| Aspect | Status |
|--------|--------|
| Routing middleware with Lambda SDK invoke | **TODO** |
| Inline route split (health, metrics, OpenAPI) | **TODO** |
| State refactoring (remove business state) | **TODO** |
| Entrypoint refactoring (init lambda_client) | **TODO** |
| Per-service circuit breakers | **TODO** |
| Correlation ID propagation | **TODO** |

---

## Location

- Code: `services/ui/src/` (after refactor)
- Plan: this file
- ADR: `plans/adr/ADR-031-lambda-microservices-architecture.md`

---

## What stays in api-gateway

1. **Health check** (`GET /health`) — inline, no backend call
2. **Metrics query** (`GET /api/v1/metrics`) — reads shared SQLite directly
3. **OpenAPI spec** (`/api/openapi.json`) — aggregate or static build-time merge
4. **SPA fallback** — for local dev mode
5. **Routing middleware** — path matching → `TargetService` → Lambda invoke
6. **Global middleware stack** — rate limiting, request validation, metrics recording, CORS, tracing

## What moves out

All business logic handlers → respective backend services:
- `routes/api/jobs.rs`, `competencies.rs`, `about.rs`, `social_links.rs`, `resume_data.rs`, `challenges.rs` → `services/portfolio/`
- `routes/api/ask.rs` + RAG retrieval → `services/rag/`
- `routes/api/admin.rs` → `services/admin/`
- `routes/auth.rs` → `services/auth/` (already extracted)
- `routes/contact.rs` → `services/contact/`

## Router structure (after refactor)

```rust
Router::new()
    // Inline routes (no backend call)
    .route("/health", get(health_handler))
    .route("/api/v1/metrics", get(metrics_handler))
    .route("/api/openapi.json", get(openapi_handler))
    .route("/docs", get(docs_handler))

    // Gateway routes (Lambda invoke)
    .route("/api/v1/portfolio/*path", any(gateway_handler))
    .route("/api/v1/rag/*path", any(gateway_handler))
    .route("/api/v1/admin/*path", any(gateway_handler))
    .route("/api/v1/auth/*path", any(gateway_handler))
    .route("/api/contact", post(gateway_handler))
    .route("/api/contact/challenge", get(gateway_handler))
    .route("/auth/*path", any(gateway_handler))

    // Global middleware
    .layer(rate_limit_middleware)
    .layer(validate_request_middleware)
    .layer(metrics_middleware)
    .with_state(state)
```

## State changes

Remove from `AppState`:
- `rag: Arc<RagStore>` → moves to `services/rag/`
- Auth-specific config → moves to `services/auth/`
- LLM keys → moves to `services/rag/`

Add to `AppState`:
- `lambda_client: aws_sdk_lambda::Client` — for SDK invoke

Keep in `AppState`:
- `db: Arc<Db>` — for metrics reads
- `rate_limiter: Arc<RateLimiter>` — global rate limiting
- `llm_breaker: Arc<CircuitBreaker>` → rename to `service_breakers: ServiceCircuitBreakers`

---

## Remaining work

- [ ] Implement `gateway.rs` routing middleware with Lambda SDK invoke (W-AGW.4.1)
- [ ] Refactor `router.rs` to inline vs gateway split (W-AGW.4.2)
- [ ] Update `state.rs` — remove business state, add lambda_client (W-AGW.4.3)
- [ ] Update `main.rs` — init lambda_client, remove business init (W-AGW.4.4)
- [ ] Add per-service circuit breakers (W-AGW.4.5)
- [ ] Add `x-request-id` correlation propagation (W-AGW.4.6)
- [ ] Update OpenAPI spec generation (aggregate or static merge) (W-AGW.4.7)
- [ ] Gateway integration tests with mock Lambda client (W-AGW.4.8)
