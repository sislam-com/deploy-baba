# Zero-Cost Microservices Patterns

**Purpose:** Document the zero-cost approach to microservices architecture patterns for deploy-baba. All patterns use code-level solutions instead of infrastructure changes to maintain the zero-cost philosophy (ADR-005).

---

## Overview

Traditional microservices architectures rely on infrastructure solutions (API Gateway, service mesh, managed databases, observability platforms) that introduce recurring cost. For a portfolio project, this cost is unjustified. This document describes code-level alternatives that provide the same benefits without infrastructure complexity or cost.

---

## Pattern 1: API Versioning (W-VER)

**Traditional Approach:** API Gateway stage variables, version-specific DNS

**Zero-Cost Approach:** URL-based versioning with code-level routing

```rust
// /api/v1/jobs vs /api/v2/jobs
Router::new()
    .nest("/api/v1", routes::api::v1::router())
    .nest("/api/v2", routes::api::v2::router())
```

**Benefits:**
- Zero infrastructure cost
- Self-documenting URLs
- Deprecation headers communicate sunset timeline
- OpenAPI spec includes version metadata

**Trade-offs:**
- URL path changes require client updates
- Version extraction adds request overhead

**→ ADR-024, W-VER**

---

## Pattern 2: Observability (W-OBS)

**Traditional Approach:** CloudWatch Metrics, Datadog, New Relic

**Zero-Cost Approach:** SQLite-based metrics + structured logging

```sql
CREATE TABLE api_metrics (
    timestamp TEXT,
    endpoint TEXT,
    duration_ms INTEGER,
    status_code INTEGER
);
```

```rust
// Structured JSON logging
tracing::info!(request_id = %uuid, endpoint = "get_jobs", "Fetching jobs");
```

**Benefits:**
- Zero additional AWS cost
- Queryable time-series data via SQL
- Structured logs improve real-time debugging
- Metrics co-located with application data

**Trade-offs:**
- SQLite write overhead on every request
- Query performance degrades with large datasets
- No AWS console dashboards

**→ ADR-025, W-OBS**

---

## Pattern 3: Resilience Patterns (W-RES)

**Traditional Approach:** AWS WAF, API Gateway throttling, Redis rate limiting

**Zero-Cost Approach:** Tower middleware + in-memory state

```rust
// In-memory rate limiting
let limiter = RateLimiter::new(100, Duration::from_secs(60));

// Retry with exponential backoff
.layer(RetryLayer::new(RetryPolicy))

// Circuit breaker for external calls
if circuit_breaker.is_open() {
    return Err(AppError::CircuitBreakerOpen);
}
```

**Benefits:**
- Zero infrastructure cost
- Standard Rust middleware patterns
- Protects against cost runaway (LLM rate limiting)
- Handles transient failures gracefully

**Trade-offs:**
- In-memory state lost on Lambda restart
- Rate limiting per Lambda instance (not distributed)
- Manual transient error detection required

**→ ADR-026, W-RES**

---

## Pattern 4: Service Decomposition (W-MOD)

**Traditional Approach:** Separate Lambda functions per service, API Gateway routing

**Zero-Cost Approach:** Logical module separation within single Lambda

```rust
// services/ui/src/modules/
├── portfolio/    // Jobs, competencies, about
├── rag/          // RAG retrieval, /api/ask
├── admin/        // Dashboard CRUD
└── auth/         // Cognito validation

// Each module has:
// - router() function
// - independent tests
// - module-specific metrics
// - module-specific rate limits
```

**Benefits:**
- Clear boundaries without infrastructure cost
- Independent testing per module
- Future extraction path to separate Lambdas
- Reduced cognitive load (smaller modules)

**Trade-offs:**
- Initial refactoring effort
- Single Lambda still scales as a unit
- Some code duplication across modules

**→ ADR-027, W-MOD**

---

## Pattern 5: Service Discovery

**Traditional Approach:** Cloud Map, Consul, etcd

**Zero-Cost Approach:** Environment variables + Lambda-to-Lambda invocation

```rust
// Environment variable for service URL
let portfolio_api_url = std::env::var("PORTFOLIO_API_BASE_URL")?;

// Direct Lambda invocation via SDK
let client = aws_sdk_lambda::Client::new(&config);
client.invoke()
    .function_name("portfolio-service")
    .payload(payload)
    .send()
    .await?;
```

**Benefits:**
- Zero managed service cost
- Simple and explicit
- Works with existing Lambda VPC endpoints

**Trade-offs:**
- Manual service URL management
- No automatic service registration
- No health checking infrastructure

---

## Pattern 6: Inter-Service Communication

**Traditional Approach:** Service mesh (Istio, AWS App Mesh), event buses

**Zero-Cost Approach:** Direct HTTP calls + EventBridge for async

```rust
// Synchronous HTTP call
let response = reqwest::get(&service_url).await?;

// Async EventBridge
aws_sdk_eventbridge::Client::new(&config)
    .put_events()
    .entries(entry)
    .send()
    .await?;
```

**Benefits:**
- Zero service mesh cost
- Explicit communication patterns
- EventBridge for decoupled async flows

**Trade-offs:**
- No automatic retries (implement in code)
- No distributed tracing (use X-Ray if needed)
- Manual circuit breaking required

---

## Pattern 7: Configuration Management

**Traditional Approach:** Parameter Store, AppConfig, external config servers

**Zero-Cost Approach:** Stack-local config + Secrets Manager for secrets

```toml
# stack.toml (local-only, not committed)
[project]
name = "deploy-baba"
mode = "production"

[observability]
log_level = "info"
metrics_enabled = true
```

```rust
// Secrets Manager for sensitive values
let secret = secrets_manager
    .get_secret_value(&secret_arn)
    .await?;
```

**Benefits:**
- Zero config management cost
- Local config for development
- Secrets Manager for production secrets
- Type-safe config parsing

**Trade-offs:**
- Manual config deployment (justfile commands)
- No dynamic config updates without redeploy
- No config versioning infrastructure

---

## Pattern 8: Deployment Automation

**Traditional Approach:** CodeDeploy, Spinnaker, ArgoCD

**Zero-Cost Approach:** GitHub Actions + xtask commands

```yaml
# .github/workflows/deploy-dev.yml
- name: Build Lambda
  run: just lambda-build

- name: Deploy Lambda
  run: just lambda-deploy ${{ secrets.AWS_PROFILE }}

- name: Verify Deployment
  run: just infra-verify dev.sislam.com
```

**Benefits:**
- Zero deployment tooling cost
- Native GitHub Actions integration
- xtask provides type-safe deployment commands
- Automated rollback on health check failure

**Trade-offs:**
- Manual pipeline configuration
- No canary deployments (implement in code if needed)
- No blue-green infrastructure (use Lambda aliases)

---

## Implementation Roadmap

### Phase 1: Foundation (P0)
- [x] ~~W-VER.4.1~~: Version extraction middleware — **DONE**
- [x] ~~W-VER.4.2~~: Deprecation headers middleware — **DONE**
- [x] ~~W-VER.4.3~~: Router structure for version nesting — **DONE**
- [x] ~~W-VER.4.4~~: OpenAPI version metadata — **DONE**
- [ ] W-OBS.4.1: Telemetry initialization
- [ ] W-OBS.4.2: SQLite metrics tables

### Phase 2: Resilience (P1)
- [ ] W-RES.4.1: In-memory rate limiting
- [ ] W-RES.4.2: Retry with exponential backoff
- [ ] W-RES.4.3: Circuit breaker for LLM calls

### Phase 3: Module Decomposition (P2)
- [ ] W-MOD.4.1: Module trait definitions
- [ ] W-MOD.4.2: Extract portfolio module
- [ ] W-MOD.4.3: Extract RAG module

### Phase 4: Advanced Features (P3)
- [ ] W-OBS.4.4: Metrics query endpoint
- [ ] W-MOD.4.6: Module-specific metrics

---

## Cross-References

- → ADR-005 (Zero-cost philosophy)
- → ADR-002 (SQLite on EFS)
- → ADR-003 (Lambda Function URL)
- → W-VER (API versioning)
- → W-OBS (Observability)
- → W-RES (Resilience patterns)
- → W-MOD (Module decomposition)
- → W-UI (Service integration)
- → W-RAG (RAG module)
- → W-AUTH (Auth module)
