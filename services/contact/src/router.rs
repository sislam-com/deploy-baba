use aws_sdk_lambda::Client;
use service_protocol::{ServiceRequest, ServiceResponse};
use std::sync::Arc;
use tracing::error;

use crate::handlers;

pub async fn route(lambda_client: &Arc<Client>, req: ServiceRequest) -> ServiceResponse {
    let path = req.path.as_str();
    let method = req.method.as_str();

    match (method, path) {
        ("GET", "/api/contact/challenge") => handlers::challenge::issue_challenge().await,
        ("POST", "/api/contact") => {
            handlers::contact::submit_contact(lambda_client, req.body).await
        }
        _ => {
            error!(method = %method, path = %path, "unknown route");
            ServiceResponse::error(404, "not found")
        }
    }
}
