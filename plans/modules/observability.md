# W-OBS: observability
**Path:** `services/ui/src/telemetry.rs`, `services/ui/migrations/` | **Status:** DONE
**Coverage floor:** 80% | **Depends on:** W-UI, W-RAG | **Depended on by:** W-RES, W-MOD

---

## W-OBS.1 Purpose

Zero-cost observability using SQLite-based metrics collection and structured logging. Avoids CloudWatch Metrics cost while providing queryable time-series data for latency, error rates, and request counts. Follows ADR-002 (SQLite on EFS) and ADR-005 (zero-cost philosophy).

→ ADR-025

---

## W-OBS.2 Public API Surface

### Telemetry Initialization

```rust
// services/ui/src/telemetry.rs
pub fn init_telemetry() {
    tracing_subscriber::registry()
        .with(fmt::layer().json())  // Structured JSON logs
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .init();
}
```

### Metrics Recording

```rust
pub async fn record_metric(
    db: &SqlitePool,
    endpoint: &str,
    method: &str,
    status: u16,
    duration_ms: u64,
    version: &str,
) -> Result<()>
```

### Metrics Query Endpoint

```
GET /api/v1/metrics?endpoint=/api/v1/jobs&hours=24
Response: {
  "p50": 45,
  "p95": 120,
  "p99": 250,
  "request_count": 1234,
  "error_rate": 0.02
}
```

---

## W-OBS.3 Implementation Notes

### Structured Logging with Tracing

```rust
use tracing::{info, instrument};

#[instrument(skip(state), fields(
    request_id = %uuid::Uuid::new_v4(),
    endpoint = "get_jobs"
))]
async fn get_jobs(State(state): State<AppState>) -> Result<Json<JobsResponse>> {
    info!("Fetching jobs");
    // ...
}
```

### SQLite Metrics Tables

```sql
-- Migration: 024_metrics.sql
CREATE TABLE IF NOT EXISTS api_metrics (
    id INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    api_version TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_metrics_endpoint 
ON api_metrics(endpoint, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_metrics_timestamp 
ON api_metrics(timestamp DESC);
```

### Percentile Calculation

```rust
fn percentile(values: &[u64], p: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let idx = ((values.len() - 1) as f64 * (p as f64 / 100.0)) as usize;
    values[idx]
}
```

### Metrics Middleware Integration

```rust
pub async fn metrics_middleware(
    State(db): State<SqlitePool>,
    State(limiter): State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    let version = extract_api_version(&path).unwrap_or(ApiVersion { major: 1, minor: 0 });
    
    // Record metric asynchronously (fire-and-forget)
    tokio::spawn(async move {
        let _ = record_metric(&db, &path, &method, status, duration, &format!("v{}", version.major)).await;
    });
    
    response
}
```

### RAG-Specific Metrics

```rust
// Track RAG retrieval latency and chunk counts
pub async fn record_rag_metric(
    db: &SqlitePool,
    query: &str,
    retrieval_latency_ms: u64,
    chunk_count: usize,
    generation_latency_ms: u64,
) -> Result<()>
```

---

## W-OBS.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-OBS.4.1 | Create telemetry initialization module | DONE | `telemetry.rs`: JSON logs in Lambda, human-readable locally |
| W-OBS.4.2 | Add SQLite metrics tables migration | DONE | Migration `026_metrics_tables.sql`: `api_metrics` + 4 indexes |
| W-OBS.4.3 | Implement metrics recording middleware | DONE | `metrics_middleware`: fire-and-forget `tokio::spawn` write to `api_metrics` |
| W-OBS.4.4 | Add metrics query endpoint | DONE | `GET /api/v1/metrics?endpoint=&hours=`: p50/p95/p99 + error rate; admin-gated |

---

## W-OBS.5 Test Strategy

- Unit tests for percentile calculation
- Integration tests for metrics recording
- Test metrics query endpoint with various filters
- Verify structured JSON log format
- Test coverage floor: 80%

---

## W-OBS.6 Cross-References

- → ADR-025 (SQLite-Based Metrics Collection)
- → ADR-002 (SQLite on EFS — metrics co-located)
- → ADR-005 (Zero-cost philosophy — no CloudWatch Metrics)
- → W-UI (middleware integration)
- → W-RAG (RAG-specific metrics)
- → W-RES (metrics-based circuit breaking)
- → W-MOD (module-specific metrics)
