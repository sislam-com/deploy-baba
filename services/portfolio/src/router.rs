use rusqlite::Connection;
use service_protocol::{ServiceRequest, ServiceResponse};
use std::sync::{Arc, Mutex};
use tracing::error;

use crate::handlers;

pub async fn route(db: &Arc<Mutex<Connection>>, req: ServiceRequest) -> ServiceResponse {
    let path = req.path.as_str();
    let method = req.method.as_str();

    match (method, path) {
        ("GET", "/api/v1/portfolio/jobs") => handlers::jobs::list_jobs(db).await,
        ("GET", "/api/v1/portfolio/jobs/:slug") => {
            // Extract slug from path
            let slug = path.split('/').next_back().unwrap_or("");
            handlers::jobs::get_job(db, slug).await
        }
        ("GET", "/api/v1/portfolio/competencies") => {
            handlers::competencies::list_competencies(db).await
        }
        ("GET", "/api/v1/portfolio/about") => handlers::about::list_about(db).await,
        ("GET", "/api/v1/portfolio/social-links") => {
            handlers::social_links::list_social_links(db).await
        }
        ("GET", "/api/v1/portfolio/resume") => handlers::resume::get_resume(db).await,
        ("GET", "/api/v1/portfolio/challenges") => handlers::challenges::list_challenges(db).await,
        ("GET", "/api/v1/portfolio/challenges/:slug") => {
            let slug = path.split('/').next_back().unwrap_or("");
            handlers::challenges::get_challenge(db, slug).await
        }
        _ => {
            error!(method = %method, path = %path, "unknown route");
            ServiceResponse::error(404, "not found")
        }
    }
}
