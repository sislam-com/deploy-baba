//! Metrics query endpoint — `GET /api/v1/metrics`.
//!
//! Admin-gated endpoint for querying SQLite-collected request metrics.
//! Returns p50/p95/p99 latency percentiles, request counts, and error rates.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::db::Db;
use crate::state::AppState;
use crate::telemetry::{query_metrics, MetricsSummary};

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default = "default_hours")]
    pub hours: u32,
}

fn default_hours() -> u32 {
    24
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_metrics))
}

pub async fn get_metrics(
    State(db): State<Arc<Db>>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<Vec<MetricsSummary>>, (StatusCode, String)> {
    match query_metrics(&db, query.endpoint.as_deref(), query.hours) {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => {
            tracing::error!("metrics query failed: {e}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
