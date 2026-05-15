# W-RES: resilience
**Path:** `services/ui/src/middleware/` | **Status:** TODO
**Coverage floor:** 80% | **Depends on:** W-UI, W-OBS | **Depended on by:** W-MOD

---

## W-RES.1 Purpose

Code-level resilience patterns including in-memory rate limiting, retry with exponential backoff, circuit breaker for external LLM calls, and request validation. Zero infrastructure cost — all patterns implemented in Rust code using Tower middleware and standard library.

→ ADR-026

---

## W-RES.2 Public API Surface

### Rate Limiter

```rust
// services/ui/src/middleware/rate_limit.rs
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self
    pub async fn check(&self, key: &str) -> bool
}
```

### Retry Policy

```rust
// services/ui/src/middleware/retry.rs
pub struct RetryPolicy;

impl<E> Policy<(), Request, E> for RetryPolicy
where E: std::error::Error + Clone
```

### Circuit Breaker

```rust
// services/ui/src/middleware/circuit_breaker.rs
pub struct CircuitBreaker {
    is_open: Arc<AtomicBool>,
    failure_count: Arc<AtomicUsize>,
    threshold: usize,
}

impl CircuitBreaker {
    pub fn new(threshold: usize) -> Self
    pub fn record_failure(&self)
    pub fn record_success(&self)
    pub fn is_open(&self) -> bool
}
```

### Request Validation

```rust
// services/ui/src/middleware/validation.rs
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct ContactRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 1000))]
    pub message: String,
}
```

---

## W-RES.3 Implementation Notes

### In-Memory Rate Limiting

```rust
// Key format: "client_ip:endpoint"
let key = format!("{}:{}",
    req.headers().get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown"),
    req.uri().path()
);

if limiter.check(&key).await {
    Ok(next.run(req).await)
} else {
    Err(StatusCode::TOO_MANY_REQUESTS)
}
```

**Configuration**:
- Default: 100 requests per minute per endpoint
- Admin endpoints: 50 requests per minute
- `/api/ask`: 10 requests per minute (LLM cost control)

### Retry with Exponential Backoff

```rust
use tower::retry::RetryLayer;

// Apply to router
.layer(RetryLayer::new(RetryPolicy))

// Transient error detection
fn is_transient<E: std::error::Error>(error: &E) -> bool {
    // Retry on: timeouts, 5xx errors, network errors
    // Don't retry: 4xx errors, validation errors
}
```

**Configuration**:
- Max retries: 3
- Initial backoff: 100ms
- Backoff multiplier: 2.0
- Max backoff: 5s

### Circuit Breaker for LLM Calls

```rust
if circuit_breaker.is_open() {
    return Err(AppError::CircuitBreakerOpen);
}

match llm_proxy.call(&prompt).await {
    Ok(response) => {
        circuit_breaker.record_success();
        Ok(response)
    }
    Err(e) if is_transient(&e) => {
        circuit_breaker.record_failure();
        Err(e)
    }
    Err(e) => Err(e),
}
```

**Configuration**:
- Failure threshold: 5 consecutive failures
- Open timeout: 60 seconds (before attempting recovery)
- Half-open requests: 1 (test request before closing)

### Request Validation Middleware

```rust
pub async fn validate_request_middleware<T: Validate>(
    Json(body): Json<T>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Err(errors) = body.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(next.run(req).await)
}
```

### Middleware Stack Order

```rust
Router::new()
    .layer(metrics_middleware)           // W-OBS
    .layer(rate_limit_middleware)       // W-RES
    .layer(retry_middleware)            // W-RES
    .layer(validation_middleware)       // W-RES
    .nest("/api/v1", routes::api::v1::router())
```

---

## W-RES.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-RES.4.1 | Implement in-memory rate limiter | TODO | Per-endpoint limits, sliding window |
| W-RES.4.2 | Add retry with exponential backoff | TODO | Tower RetryLayer, transient error detection |
| W-RES.4.3 | Implement circuit breaker for LLM calls | TODO | Atomic state, failure threshold, recovery logic |
| W-RES.4.4 | Add request validation middleware | TODO | validator crate, error responses |

---

## W-RES.5 Test Strategy

- Unit tests for rate limiter (window expiration, limit enforcement)
- Unit tests for circuit breaker state transitions
- Integration tests for retry logic with mock failures
- Validation tests with malformed requests
- Test coverage floor: 80%

---

## W-RES.6 Cross-References

- → ADR-026 (Code-Level Resilience Patterns)
- → ADR-005 (Zero-cost philosophy — no external dependencies)
- → W-UI (middleware stack integration)
- → W-OBS (metrics for circuit breaker decisions)
- → W-LLM (circuit breaker for LLM proxy calls)
- → W-MOD (module-specific rate limits)
