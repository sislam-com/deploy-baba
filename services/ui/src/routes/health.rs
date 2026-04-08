use axum::Json;

pub use api_openapi::models::HealthResponse;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: VERSION.into(),
    })
}
