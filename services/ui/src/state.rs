use std::sync::Arc;
use std::time::Duration;

use crate::auth::AuthConfig;
use crate::db::Db;
use crate::middleware::{CircuitBreaker, RateLimiter};

use rag_sqlite::RagStore;

/// Top-level application state threaded through all routes.
///
/// `FromRef` implementations allow handlers to extract sub-states directly:
/// - `State(db): State<Arc<Db>>` — existing handlers, unchanged
/// - `State(auth): State<Arc<AuthConfig>>` — auth-aware handlers
/// - `State(rag): State<Arc<RagStore>>` — RAG FTS retriever
/// - `State(limiter): State<Arc<RateLimiter>>` — rate-limited handlers
/// - `State(breaker): State<Arc<CircuitBreaker>>` — circuit-protected handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub auth: Arc<AuthConfig>,
    /// RAG store (FTS5 retrieval backed by same SQLite file as `db`).
    pub rag: Arc<RagStore>,
    /// In-memory rate limiter (sliding window per client IP + endpoint).
    pub rate_limiter: Arc<RateLimiter>,
    /// Circuit breaker for external LLM calls.
    pub llm_breaker: Arc<CircuitBreaker>,
}

impl AppState {
    /// Build a fresh AppState with default resilience configuration.
    pub fn with_defaults(db: Arc<Db>, auth: Arc<AuthConfig>, rag: Arc<RagStore>) -> Self {
        Self {
            db,
            auth,
            rag,
            rate_limiter: Arc::new(RateLimiter::new(
                100, // 100 requests per window
                Duration::from_secs(60),
            )),
            llm_breaker: Arc::new(CircuitBreaker::new(
                5, // open after 5 consecutive failures
                Duration::from_secs(60),
            )),
        }
    }
}

impl axum::extract::FromRef<AppState> for Arc<Db> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<AuthConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<RagStore> {
    fn from_ref(state: &AppState) -> Self {
        state.rag.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<RateLimiter> {
    fn from_ref(state: &AppState) -> Self {
        state.rate_limiter.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<CircuitBreaker> {
    fn from_ref(state: &AppState) -> Self {
        state.llm_breaker.clone()
    }
}
