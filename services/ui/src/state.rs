use std::sync::Arc;

use crate::auth::AuthConfig;
use crate::db::Db;

/// Top-level application state threaded through all routes.
///
/// `FromRef` implementations allow handlers to extract sub-states directly:
/// - `State(db): State<Arc<Db>>` — existing handlers, unchanged
/// - `State(auth): State<Arc<AuthConfig>>` — auth-aware handlers
/// - `State(llm): State<Option<Arc<String>>>` — Anthropic API key (None in local dev without key)
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub auth: Arc<AuthConfig>,
    /// Anthropic API key loaded from Secrets Manager (or ANTHROPIC_API_KEY env var in dev).
    /// `None` when neither source is available — LLM-dependent routes return 503 in that case.
    pub anthropic_api_key: Option<Arc<String>>,
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
