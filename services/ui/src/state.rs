use std::sync::Arc;

use crate::auth::AuthConfig;
use crate::db::Db;

/// Top-level application state threaded through all routes.
///
/// `FromRef` implementations allow handlers to extract sub-states directly:
/// - `State(db): State<Arc<Db>>` — existing handlers, unchanged
/// - `State(auth): State<Arc<AuthConfig>>` — auth-aware handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub auth: Arc<AuthConfig>,
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
