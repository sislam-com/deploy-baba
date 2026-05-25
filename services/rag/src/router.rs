use rusqlite::Connection;
use service_protocol::{ServiceRequest, ServiceResponse};
use std::sync::{Arc, Mutex};
use tracing::error;

use crate::handlers;

pub async fn route(_db: &Arc<Mutex<Connection>>, req: ServiceRequest) -> ServiceResponse {
    let path = req.path.as_str();
    let method = req.method.as_str();

    match (method, path) {
        ("POST", "/api/ask") => handlers::ask::ask_handler(req.body).await,
        ("POST", "/api/v1/rag/ask") => handlers::ask::ask_handler(req.body).await,
        _ => {
            error!(method = %method, path = %path, "unknown route");
            ServiceResponse::error(404, "not found")
        }
    }
}
