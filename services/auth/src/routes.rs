use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cognito;
use crate::error::AuthError;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/auth/signin", post(signin_handler))
        .route("/api/auth/forgot-password", post(forgot_password_handler))
        .route(
            "/api/auth/confirm-forgot-password",
            post(confirm_forgot_password_handler),
        )
        .route(
            "/api/auth/respond-to-challenge",
            post(respond_to_challenge_handler),
        )
        .route("/api/auth/signout", post(signout_handler))
        .route("/health", get(health_handler))
        .with_state(state)
}

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct SignInRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ForgotPasswordRequest {
    username: String,
}

#[derive(Deserialize)]
struct ConfirmForgotPasswordRequest {
    username: String,
    confirmation_code: String,
    new_password: String,
}

#[derive(Deserialize)]
struct RespondToChallengeRequest {
    challenge_name: String,
    session: String,
    #[serde(flatten)]
    responses: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
struct SignOutRequest {
    access_token: String,
}

#[derive(Serialize)]
struct SignInResponse {
    success: bool,
    #[serde(flatten)]
    result: Option<cognito::AuthResult>,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn signin_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignInRequest>,
) -> Result<impl IntoResponse, AuthError> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AuthError::MissingField(
            "username and password are required".to_string(),
        ));
    }

    let result =
        cognito::sign_in(&state.cognito, &state.config, &req.username, &req.password).await?;

    Ok((
        StatusCode::OK,
        Json(SignInResponse {
            success: true,
            result: Some(result),
        }),
    ))
}

async fn forgot_password_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    if req.username.is_empty() {
        return Err(AuthError::MissingField("username is required".to_string()));
    }

    cognito::forgot_password(&state.cognito, &state.config, &req.username).await?;

    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

async fn confirm_forgot_password_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfirmForgotPasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    if req.username.is_empty() || req.confirmation_code.is_empty() || req.new_password.is_empty() {
        return Err(AuthError::MissingField(
            "username, confirmation_code, and new_password are required".to_string(),
        ));
    }

    cognito::confirm_forgot_password(
        &state.cognito,
        &state.config,
        &req.username,
        &req.confirmation_code,
        &req.new_password,
    )
    .await?;

    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

async fn respond_to_challenge_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RespondToChallengeRequest>,
) -> Result<impl IntoResponse, AuthError> {
    if req.challenge_name.is_empty() || req.session.is_empty() {
        return Err(AuthError::MissingField(
            "challenge_name and session are required".to_string(),
        ));
    }

    // Remove challenge_name and session from the responses map before sending to Cognito
    let mut challenge_responses = req.responses.clone();
    challenge_responses.remove("challenge_name");
    challenge_responses.remove("session");

    let result = cognito::respond_to_challenge(
        &state.cognito,
        &state.config,
        &req.challenge_name,
        &req.session,
        challenge_responses,
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(SignInResponse {
            success: true,
            result: Some(result),
        }),
    ))
}

async fn signout_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignOutRequest>,
) -> Result<impl IntoResponse, AuthError> {
    if req.access_token.is_empty() {
        return Err(AuthError::MissingField(
            "access_token is required".to_string(),
        ));
    }

    cognito::global_sign_out(&state.cognito, &state.config, &req.access_token).await?;

    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

use axum::routing::get;

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}
