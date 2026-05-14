use axum::{
    response::Html,
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
        // ── Resume file downloads ────────────────────────────────────────────
        .nest_service("/resume", ServeDir::new("target/resume"))
        // ── API versioning (ADR-024) ─────────────────────────────────────────
        // Phase 1: Version extraction middleware added, routes unchanged
        // Future: Migrate to /api/v1/ structure with backward-compatible redirects
        // ── API routes (current unversioned, will migrate to /api/v1/ in Phase 2) ─
        .nest("/api", routes::api::router())
        .nest("/api/admin", admin_routes)
        // ── Auth routes (server-side Cognito redirects) ─────────────────────
        .nest("/auth", routes::auth::router())
        // ── OpenAPI spec ─────────────────────────────────────────────────────
        // Full combined spec — served unauthenticated so /docs shows all routes.
        // /api/openapi-admin.json kept as an alias for backward compatibility.
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
