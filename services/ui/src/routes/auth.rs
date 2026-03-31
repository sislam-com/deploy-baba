use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::AuthConfig;
use crate::state::AppState;

/// GET /auth/login — redirect to Cognito hosted UI (or create dev session).
pub async fn login_handler(State(auth): State<Arc<AuthConfig>>) -> Response {
    if auth.dev_mode {
        // Skip Cognito; issue a fake cookie so the dashboard is reachable locally.
        let cookie = "auth_token=dev-bypass; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600";
        let mut headers = HeaderMap::new();
        headers.insert(header::SET_COOKIE, HeaderValue::from_static(cookie));
        headers.insert(header::LOCATION, HeaderValue::from_static("/dashboard"));
        return (StatusCode::FOUND, headers).into_response();
    }

    // Implicit grant: response_type=token with openid scope returns both access_token and
    // id_token in the fragment (#access_token=...&id_token=...) — no server exchange needed.
    let url = format!(
        "https://{}/oauth2/authorize?client_id={}&response_type=token\
         &scope=openid+email+profile&redirect_uri={}/auth/callback",
        auth.cognito_domain, auth.client_id, auth.app_domain
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&url).unwrap_or_else(|_| HeaderValue::from_static("/")),
    );
    (StatusCode::FOUND, headers).into_response()
}

/// GET /auth/callback — serve HTML page that extracts id_token from URL fragment via JS.
///
/// With implicit grant the token is in the fragment (#id_token=...) which is never sent to
/// the server.  This page runs client-side JS to extract the token and POST it to
/// /auth/set-session, which sets the HttpOnly cookie server-side.
pub async fn callback_handler() -> Response {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Signing in…</title>
  <style>
    body { font-family: system-ui, sans-serif; display: flex; align-items: center;
           justify-content: center; min-height: 100vh; margin: 0; background: #0f172a; color: #e2e8f0; }
    .msg { text-align: center; }
    .error { color: #f87171; }
  </style>
</head>
<body>
<div class="msg" id="msg">Signing in…</div>
<script>
(function() {
  var hash = window.location.hash.substring(1);
  var params = {};
  hash.split('&').forEach(function(part) {
    var eq = part.indexOf('=');
    if (eq > -1) params[part.substring(0, eq)] = decodeURIComponent(part.substring(eq + 1));
  });

  // Check for OAuth errors in query string or fragment
  var search = new URLSearchParams(window.location.search);
  var err = params['error'] || search.get('error');
  if (err) {
    document.getElementById('msg').className = 'msg error';
    document.getElementById('msg').textContent = 'Login error: ' + (params['error_description'] || err);
    return;
  }

  var idToken = params['id_token'];
  if (!idToken) {
    document.getElementById('msg').className = 'msg error';
    document.getElementById('msg').textContent = 'No token received. Please try again.';
    return;
  }

  window.location.href = '/auth/set-session?id_token=' + encodeURIComponent(idToken);
})();
</script>
</body>
</html>"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct SetSessionQuery {
    id_token: String,
}

/// GET /auth/set-session?id_token=... — validate id_token and set HttpOnly cookie.
///
/// Called by the /auth/callback JS via redirect after extracting the id_token from the URL
/// fragment. Using GET avoids SigV4 body hash mismatch with CloudFront OAC + Lambda Function URL.
pub async fn set_session_handler(
    State(auth): State<Arc<AuthConfig>>,
    Query(params): Query<SetSessionQuery>,
) -> Response {
    match auth.validate_token(&params.id_token).await {
        Ok(_) => {
            let secure = if auth.app_domain.starts_with("https") {
                "; Secure"
            } else {
                ""
            };
            let cookie = format!(
                "auth_token={}; Path=/; HttpOnly{}; SameSite=Lax; Max-Age=3600",
                params.id_token, secure
            );
            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie).unwrap_or_else(|_| HeaderValue::from_static("")),
            );
            headers.insert(header::LOCATION, HeaderValue::from_static("/dashboard"));
            (StatusCode::FOUND, headers).into_response()
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    }
}

/// GET /auth/logout — clear cookie and redirect to Cognito logout (or home in dev).
pub async fn logout_handler(State(auth): State<Arc<AuthConfig>>) -> Response {
    let clear = "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, HeaderValue::from_static(clear));

    let location = if auth.dev_mode {
        "/".to_string()
    } else {
        format!(
            "https://{}/logout?client_id={}&logout_uri={}",
            auth.cognito_domain, auth.client_id, auth.app_domain
        )
    };

    headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&location).unwrap_or_else(|_| HeaderValue::from_static("/")),
    );
    (StatusCode::FOUND, headers).into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .route("/set-session", get(set_session_handler))
        .route("/logout", get(logout_handler))
}
