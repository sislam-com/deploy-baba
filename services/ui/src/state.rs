use std::sync::Arc;

use crate::auth::AuthConfig;
use crate::db::Db;
use rag_sqlite::RagStore;

/// Top-level application state threaded through all routes.
///
/// `FromRef` implementations allow handlers to extract sub-states directly:
/// - `State(db): State<Arc<Db>>` — existing handlers, unchanged
/// - `State(auth): State<Arc<AuthConfig>>` — auth-aware handlers
/// - `State(rag): State<Arc<RagStore>>` — RAG FTS retriever
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub auth: Arc<AuthConfig>,
    /// RAG store (FTS5 retrieval backed by same SQLite file as `db`).
    pub rag: Arc<RagStore>,
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
