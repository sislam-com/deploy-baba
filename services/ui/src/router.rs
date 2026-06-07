use axum::{
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

use crate::openapi::ApiDoc;
use crate::routes;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    // Single unified spec — public and admin paths together.
    // Admin paths carry `security` annotations so RapiDoc renders lock icons;
    // the actual require_auth middleware on /api/admin/* is unchanged.
    let spec = ApiDoc::openapi();
    let spec_clone = spec.clone();

    let admin_routes = routes::api::admin::router().route_layer(
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::require_auth),
    );

    let metrics_routes = routes::api::metrics::router().route_layer(
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::require_auth),
    );

    // SPA assets (/, /about, /assets/*) are served directly from S3 via CloudFront OAC.
    // Lambda handles only /api/*, /auth/*, /health, /docs, /resume/* (ADR-003 + cdn.tf behaviors).

    Router::new()
        // ── Health ───────────────────────────────────────────────────────────
        .route("/health", get(routes::health::get_health))
        // ── Contact API (not under /api to preserve ADR-009 path) ───────────
        .route(
            "/api/contact/challenge",
            get(routes::contact::challenge_issue),
        )
        .route("/api/contact", post(routes::contact::contact_submit))
        .route("/api/ask", post(routes::api::ask::ask))
        // ── Resume file downloads ────────────────────────────────────────────
        .nest_service("/resume", ServeDir::new("target/resume"))
        // ── API Versioning (ADR-024) ─────────────────────────────────────────
        // Phase 2: Routes migrated to /api/v1/ with backward-compatible redirects
        // ── Versioned API routes (v1) ─────────────────────────────────────────
        .nest("/api/v1", routes::api::router())
        .nest("/api/v1/admin", admin_routes)
        .nest("/api/v1/metrics", metrics_routes)
        // Apply deprecation middleware to versioned routes
        // Currently a no-op (v1 is current); will add headers when v1 is deprecated
        .layer(axum::middleware::from_fn(
            crate::middleware::deprecation_middleware,
        ))
        // ── Backward-compatible redirects ─────────────────────────────────────
        // Redirect /api/* → /api/v1/* for existing clients
        // Preserves /api/contact, /api/openapi.json, /api/openapi-admin.json
        .route("/api/*path", get(api_redirect_handler))
        // ── Auth routes (server-side Cognito redirects) ─────────────────────
        .nest("/auth", routes::auth::router())
        // ── OpenAPI spec ─────────────────────────────────────────────────────
        // Full combined spec — served unauthenticated so /docs shows all routes.
        .route(
            "/api/openapi.json",
            get(move || async move { axum::Json(spec_clone) }),
        )
        .route(
            "/api/openapi-admin.json",
            get(move || async move { axum::Json(spec.clone()) }),
        )
        // ── API Docs (RapiDoc) ───────────────────────────────────────────────
        .route("/docs", get(docs_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::validate_request_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::telemetry::metrics_middleware,
        ))
        .with_state(state)
}

async fn docs_handler() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>deploy-baba API Docs</title>
<script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
</head><body>
<rapi-doc spec-url="/api/openapi.json" theme="dark" render-style="read"
  show-header="false" allow-try="true"></rapi-doc>
</body></html>"#,
    )
}

/// Redirect handler for backward-compatible API versioning.
///
/// Redirects unversioned `/api/*` requests to `/api/v1/*`.
/// Preserves special paths: /api/contact, /api/openapi.json, /api/openapi-admin.json
///
/// Example:
/// - `/api/jobs` → `/api/v1/jobs`
/// - `/api/about` → `/api/v1/about`
async fn api_redirect_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<Redirect, StatusCode> {
    // Preserve special paths that should not be redirected
    if path.starts_with("v1/") || path == "v1" {
        return Err(StatusCode::NOT_FOUND);
    }

    let preserved_paths = [
        "contact",
        "contact/challenge",
        "openapi.json",
        "openapi-admin.json",
    ];

    if preserved_paths.contains(&path.as_str()) {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Redirect::temporary(&format!("/api/v1/{}", path)))
}
