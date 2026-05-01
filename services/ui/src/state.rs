use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::AuthConfig;
use crate::db::Db;
use rag_sqlite::RagStore;

/// Top-level application state threaded through all routes.
///
/// `FromRef` implementations allow handlers to extract sub-states directly:
/// - `State(db): State<Arc<Db>>` — existing handlers, unchanged
/// - `State(auth): State<Arc<AuthConfig>>` — auth-aware handlers
/// - `State(llm): State<Option<Arc<String>>>` — Anthropic API key (None in local dev without key)
/// - `State(rag): State<Arc<RagStore>>` — RAG FTS retriever
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub auth: Arc<AuthConfig>,
    /// Anthropic API key loaded from Secrets Manager (or ANTHROPIC_API_KEY env var in dev).
    /// `None` when neither source is available — LLM-dependent routes return 503 in that case.
    pub anthropic_api_key: Option<Arc<String>>,
    /// RAG store (FTS5 retrieval backed by same SQLite file as `db`).
    pub rag: Arc<RagStore>,
    /// Absolute path to the SPA build root (contains index.html + assets/).
    /// Lambda: `/mnt/spa/active` (EFS symlink). Local: `web/dist` or SPA_ROOT env.
    pub spa_root: PathBuf,
    /// S3 bucket used by the sync-spa action.
    pub spa_bucket: String,
    /// S3 client for SPA sync.
    pub s3: aws_sdk_s3::Client,
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
