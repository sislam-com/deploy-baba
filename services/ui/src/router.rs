use axum::{response::Html, routing::get, Router};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

use crate::openapi::ApiDoc;
use crate::routes;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    let api_routes = routes::api::router();

    // Admin routes — protected by require_auth middleware
    let admin_routes = routes::api::admin::router().route_layer(
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::require_auth),
    );

    // Dashboard routes — all protected by require_auth middleware
    let dashboard_route = Router::new()
        .route("/dashboard", get(routes::dashboard::dashboard_home))
        .route(
            "/dashboard/jobs",
            get(routes::dashboard::dashboard_jobs_list),
        )
        .route(
            "/dashboard/jobs/new",
            get(routes::dashboard::dashboard_job_new),
        )
        .route(
            "/dashboard/jobs/:slug",
            get(routes::dashboard::dashboard_job_detail),
        )
        .route(
            "/dashboard/competencies",
            get(routes::dashboard::dashboard_competencies_list),
        )
        .route(
            "/dashboard/competencies/:slug",
            get(routes::dashboard::dashboard_competency_detail),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::require_auth,
        ));

    let openapi = ApiDoc::openapi();

    Router::new()
        .route("/", get(routes::resume::handler))
        .nest_service("/resume", ServeDir::new("target/resume"))
        .route("/health", get(routes::health::get_health))
        .nest("/api", api_routes)
        .nest("/api/admin", admin_routes)
        .merge(dashboard_route)
        .nest("/auth", routes::auth::router())
        .route("/docs", get(docs_handler))
        .route(
            "/api/openapi.json",
            get(move || async move { axum::Json(openapi) }),
        )
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
