use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Auth service error type.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("Cognito error: {0}")]
    Cognito(String),
    #[error("authentication failed: {0}")]
    AuthFailed(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match &self {
            AuthError::MissingField(_) => StatusCode::BAD_REQUEST,
            AuthError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AuthError::Cognito(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::AuthFailed(_) => StatusCode::UNAUTHORIZED,
        };

        let body = Json(json!({"error": self.to_string()}));
        (status, body).into_response()
    }
}
