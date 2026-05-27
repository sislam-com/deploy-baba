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

  fetch('/auth/set-session?id_token=' + encodeURIComponent(idToken))
    .then(function(r) {
      if (r.ok) { window.location.href = '/dashboard'; }
      else {
        document.getElementById('msg').className = 'msg error';
        document.getElementById('msg').textContent = 'Session failed. Please try again.';
      }
    })
    .catch(function() {
      document.getElementById('msg').className = 'msg error';
      document.getElementById('msg').textContent = 'Network error. Please try again.';
    });
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
/// Returns 200 with Set-Cookie header. The SPA navigates to /dashboard via React Router
/// after a successful response. Using 200 (not 302) ensures the browser stores the cookie
/// when called from fetch().
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
            (StatusCode::OK, headers, "ok").into_response()
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    }
}

/// GET /auth/logout — clear cookie and redirect to home.
pub async fn logout_handler() -> Response {
    let clear = "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, HeaderValue::from_static(clear));
    headers.insert(header::LOCATION, HeaderValue::from_static("/"));
    (StatusCode::FOUND, headers).into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/callback", get(callback_handler))
        .route("/set-session", get(set_session_handler))
        .route("/logout", get(logout_handler))
}
