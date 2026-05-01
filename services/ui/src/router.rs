use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

use crate::openapi::ApiDoc;
use crate::routes;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    let full_spec = ApiDoc::openapi();
    let public_spec = api_openapi::filter::public_view(&full_spec);

    let public_spec_clone = public_spec.clone();
    let full_spec_clone = full_spec.clone();

    let admin_routes = routes::api::admin::router().route_layer(
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::require_auth),
    );

    // SPA root — serves index.html as fallback for client-side routing
    let spa_root = state.spa_root.clone();
    let spa_assets_dir = spa_root.join("assets");
    let index_html = spa_root.join("index.html");

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
        // ── API routes ───────────────────────────────────────────────────────
        .nest("/api", routes::api::router())
        .nest("/api/admin", admin_routes)
        // ── Auth routes (server-side Cognito redirects) ─────────────────────
        .nest("/auth", routes::auth::router())
        // ── OpenAPI specs ────────────────────────────────────────────────────
        .route(
            "/api/openapi.json",
            get(move || async move { axum::Json(public_spec_clone) }),
        )
        .route(
            "/api/openapi-admin.json",
            get(move || async move { axum::Json(full_spec_clone) }).route_layer(
                axum::middleware::from_fn_with_state(
                    state.clone(),
                    crate::middleware::require_auth,
                ),
            ),
        )
        // ── API Docs (RapiDoc) ───────────────────────────────────────────────
        .route("/docs", get(docs_handler))
        .route("/docs/admin", get(docs_admin_handler))
        // ── SPA hashed assets — long-lived cache (filenames contain content hash) ─
        .nest_service("/assets", ServeDir::new(&spa_assets_dir).precompressed_br())
        // ── SPA fallback — serve index.html for any unmatched path ───────────
        .fallback_service(ServeDir::new(&spa_root).fallback(ServeFile::new(&index_html)))
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

async fn docs_admin_handler() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>deploy-baba Admin API Docs</title>
<script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
</head><body>
<rapi-doc spec-url="/api/openapi-admin.json" theme="dark" render-style="read"
  show-header="false" allow-try="true"></rapi-doc>
</body></html>"#,
    )
}
