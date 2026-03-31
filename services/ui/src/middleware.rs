use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::auth::AuthConfig;

/// Axum middleware that enforces authentication.
///
/// Token extraction order:
/// 1. `auth_token` HttpOnly cookie (set by `/auth/callback`)
/// 2. `Authorization: Bearer <token>` header (API fallback)
///
/// On failure:
/// - `Accept: application/json` → 401 JSON
/// - Otherwise → 302 to Cognito login (or `/auth/login` in dev mode)
pub async fn require_auth(
    State(auth): State<Arc<AuthConfig>>,
    mut req: Request,
    next: Next,
) -> Response {
    match extract_token(req.headers()) {
        Some(token) => match auth.validate_token(&token).await {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
                next.run(req).await
            }
            Err(_) => redirect_or_401(req.headers(), &auth),
        },
        None => redirect_or_401(req.headers(), &auth),
    }
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    // 1. auth_token cookie
    if let Some(cookie_hdr) = headers.get("cookie") {
        if let Ok(s) = cookie_hdr.to_str() {
            for part in s.split(';') {
                let part = part.trim();
                if let Some(val) = part.strip_prefix("auth_token=") {
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }

    // 2. Authorization: Bearer <token>
    if let Some(auth_hdr) = headers.get("authorization") {
        if let Ok(val) = auth_hdr.to_str() {
            if let Some(token) = val.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    None
}

fn redirect_or_401(headers: &HeaderMap, auth: &AuthConfig) -> Response {
    let wants_json = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("application/json"))
        .unwrap_or(false);

    if wants_json {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response()
    } else {
        let location = if auth.dev_mode {
            "/auth/login".to_string()
        } else {
            format!(
                "https://{}/oauth2/authorize?client_id={}&response_type=code\
                 &scope=openid+email+profile&redirect_uri={}/auth/callback",
                auth.cognito_domain, auth.client_id, auth.app_domain
            )
        };

        let mut resp_headers = HeaderMap::new();
        resp_headers.insert(
            axum::http::header::LOCATION,
            HeaderValue::from_str(&location).unwrap_or_else(|_| HeaderValue::from_static("/")),
        );
        (StatusCode::FOUND, resp_headers).into_response()
    }
}
