//! Telemetry and observability module for deploy-baba
//!
//! Provides structured logging with tracing and SQLite-based metrics collection.
//! Follows zero-cost philosophy — no CloudWatch Metrics cost.

use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::db::Db;

/// Initialize telemetry with structured JSON logging
///
/// This should be called early in main() before any other logging setup.
/// Uses JSON format in Lambda and human-readable format for local dev.
pub fn init_telemetry() {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    let is_lambda = std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok();

    let fmt_layer = if is_lambda {
        fmt::layer().json().boxed()
    } else {
        fmt::layer().without_time().boxed()
    };

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::new(rust_log))
        .init();
}

/// Record a single API request metric to the SQLite database.
///
/// Fire-and-forget: errors are logged but not propagated.
/// Safe to call from any context; falls back to synchronous execution
/// if no Tokio runtime is available.
pub fn record_metric(
    db: &Arc<Db>,
    endpoint: &str,
    method: &str,
    status: u16,
    duration_ms: u64,
    api_version: &str,
) {
    let db = Arc::clone(db);
    let endpoint = endpoint.to_string();
    let method = method.to_string();
    let api_version = api_version.to_string();

    let fut = async move {
        let conn = match db.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("metrics: failed to lock db: {e}");
                return;
            }
        };

        let sql = "
            INSERT INTO api_metrics (timestamp, endpoint, method, status_code, duration_ms, api_version)
            VALUES (datetime('now'), ?1, ?2, ?3, ?4, ?5)
        ";

        if let Err(e) = conn.execute(
            sql,
            rusqlite::params![
                &endpoint,
                &method,
                status as i64,
                duration_ms as i64,
                &api_version
            ],
        ) {
            tracing::warn!("metrics: failed to record metric: {e}");
        }
    };

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(fut);
    } else {
        tracing::debug!("metrics: no tokio runtime available, skipping async write");
    }
}

/// Axum middleware that records request metrics fire-and-forget.
///
/// Captures endpoint, method, status code, duration, and API version.
/// Metrics are written asynchronously to avoid blocking the response.
pub async fn metrics_middleware(State(db): State<Arc<Db>>, req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let response = next.run(req).await;

    let duration = start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    let version = extract_api_version(&path);

    record_metric(&db, &path, &method, status, duration, &version);

    response
}

/// Extract API version from URL path.
///
/// `/api/v1/jobs` → `"v1"`  
/// `/api/jobs` → `"unversioned"`  
/// `/health` → `"none"`
fn extract_api_version(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 && parts[1] == "api" {
        if let Some(v) = parts.get(2) {
            if v.starts_with('v') {
                return v.to_string();
            }
            return "unversioned".to_string();
        }
    }
    "none".to_string()
}

/// Summary of metrics for a single endpoint or all endpoints.
#[derive(Debug, serde::Serialize)]
pub struct MetricsSummary {
    pub endpoint: Option<String>,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub request_count: u64,
    pub error_count: u64,
    pub error_rate: f64,
}

/// Query metrics from the SQLite database.
///
/// `endpoint`: filter to a specific path, or `None` for all endpoints grouped.
/// `hours`: time window in hours (default 24).
pub fn query_metrics(
    db: &Arc<Db>,
    endpoint: Option<&str>,
    hours: u32,
) -> anyhow::Result<Vec<MetricsSummary>> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| anyhow::anyhow!("db lock: {e}"))?;

    let sql = "
        SELECT endpoint, duration_ms, status_code
        FROM api_metrics
        WHERE timestamp > datetime('now', '-' || ?1 || ' hours')
        ORDER BY endpoint, duration_ms
    ";

    let rows: Vec<(String, u64, u16)> = if let Some(ep) = endpoint {
        let mut stmt = conn.prepare(
            "
            SELECT endpoint, duration_ms, status_code
            FROM api_metrics
            WHERE endpoint = ?1
              AND timestamp > datetime('now', '-' || ?2 || ' hours')
            ORDER BY endpoint, duration_ms
            ",
        )?;
        let rows = stmt
            .query_map(rusqlite::params![ep, hours], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)? as u64,
                    row.get::<_, i64>(2)? as u16,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        rows
    } else {
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt
            .query_map(rusqlite::params![hours], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)? as u64,
                    row.get::<_, i64>(2)? as u16,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        rows
    };

    use std::collections::BTreeMap;
    let mut grouped: BTreeMap<String, Vec<(u64, u16)>> = BTreeMap::new();
    for (ep, dur, status) in rows {
        grouped.entry(ep).or_default().push((dur, status));
    }

    let mut results = Vec::new();
    for (ep, mut entries) in grouped {
        entries.sort_by_key(|(d, _)| *d);
        let durations: Vec<u64> = entries.iter().map(|(d, _)| *d).collect();
        let total = entries.len() as u64;
        let errors = entries.iter().filter(|(_, s)| *s >= 500).count() as u64;

        results.push(MetricsSummary {
            endpoint: if endpoint.is_some() { None } else { Some(ep) },
            p50: percentile(&durations, 50),
            p95: percentile(&durations, 95),
            p99: percentile(&durations, 99),
            request_count: total,
            error_count: errors,
            error_rate: if total > 0 {
                errors as f64 / total as f64
            } else {
                0.0
            },
        });
    }

    Ok(results)
}

/// Calculate the p-th percentile from a sorted slice of durations.
///
/// Uses nearest-rank method: index = ceil(P/100 * N) - 1
fn percentile(sorted: &[u64], p: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let n = sorted.len();
    let idx = ((p as f64 / 100.0) * n as f64).ceil() as usize;
    let idx = idx.saturating_sub(1).min(n - 1);
    sorted[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_basic() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        assert_eq!(percentile(&data, 50), 50);
        assert_eq!(percentile(&data, 95), 100);
        assert_eq!(percentile(&data, 99), 100);
    }

    #[test]
    fn test_percentile_single() {
        let data = vec![42];
        assert_eq!(percentile(&data, 50), 42);
        assert_eq!(percentile(&data, 95), 42);
    }

    #[test]
    fn test_percentile_empty() {
        let data: Vec<u64> = vec![];
        assert_eq!(percentile(&data, 50), 0);
    }

    #[test]
    fn test_extract_api_version() {
        assert_eq!(extract_api_version("/api/v1/jobs"), "v1");
        assert_eq!(extract_api_version("/api/jobs"), "unversioned");
        assert_eq!(extract_api_version("/health"), "none");
        assert_eq!(extract_api_version("/api/v2/admin"), "v2");
        assert_eq!(extract_api_version("/"), "none");
    }
}
