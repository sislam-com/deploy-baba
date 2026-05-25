use rusqlite::Connection;
use service_protocol::{ServiceRequest, ServiceResponse};
use std::sync::{Arc, Mutex};
use tracing::error;

use crate::handlers;

pub async fn route(_db: &Arc<Mutex<Connection>>, req: ServiceRequest) -> ServiceResponse {
    let path = req.path.as_str();
    let method = req.method.as_str();

    match (method, path) {
        ("GET", "/api/v1/admin/jobs") => handlers::jobs::list_jobs().await,
        ("POST", "/api/v1/admin/jobs") => handlers::jobs::create_job(req.body).await,
        ("PUT", "/api/v1/admin/jobs/:slug") => {
            let slug = path.split('/').next_back().unwrap_or("");
            handlers::jobs::update_job(slug, req.body).await
        }
        ("DELETE", "/api/v1/admin/jobs/:slug") => {
            let slug = path.split('/').next_back().unwrap_or("");
            handlers::jobs::delete_job(slug).await
        }
        _ => {
            error!(method = %method, path = %path, "unknown route");
            ServiceResponse::error(404, "not found")
        }
    }
}
