use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use crate::middleware::extract_token_from_headers;
use crate::state::AppState;

pub use api_openapi::models::AuthMe;

#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Auth status: whether the caller is authenticated and their email", body = AuthMe)
    )
)]
pub async fn auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthMe>, (StatusCode, String)> {
    match extract_token_from_headers(&headers) {
        Some(token) => match state.auth.validate_token(&token).await {
            Ok(claims) => Ok(Json(AuthMe {
                authenticated: true,
                email: Some(claims.email),
            })),
            Err(_) => Ok(Json(AuthMe {
                authenticated: false,
                email: None,
            })),
        },
        None => Ok(Json(AuthMe {
            authenticated: false,
            email: None,
        })),
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(auth_me))
}
