use axum::{routing::get, Json, Router};

use crate::state::AppState;

const STACK_TOML: &str = include_str!("../../../../../stack.toml");

#[utoipa::path(
    get,
    path = "/api/stack",
    tag = "stack",
    responses(
        (status = 200, description = "Stack configuration as JSON", body = serde_json::Value)
    )
)]
pub async fn get_stack() -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    match toml::from_str::<serde_json::Value>(STACK_TOML) {
        Ok(value) => Ok(Json(value)),
        Err(e) => {
            tracing::error!("Failed to parse stack.toml: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse stack configuration: {}", e),
            ))
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_stack))
}
