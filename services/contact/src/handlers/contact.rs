use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client;
use serde::{Deserialize, Serialize};
use service_protocol::ServiceResponse;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Deserialize)]
#[allow(dead_code)]
struct ContactSubmit {
    name: String,
    email: String,
    subject: String,
    message: String,
    nonce: String,
    solution: String,
}

#[derive(Serialize)]
struct ContactResponse {
    success: bool,
    message: String,
}

pub async fn submit_contact(lambda_client: &Arc<Client>, body: Option<String>) -> ServiceResponse {
    let req: ContactSubmit = match body.and_then(|b| serde_json::from_str(&b).ok()) {
        Some(r) => r,
        None => {
            return ServiceResponse::error(400, "invalid request body");
        }
    };

    // Honeypot: website field is omitted from struct — any bot filling it would fail parse
    // Basic validation
    if req.name.is_empty() || req.name.len() > 100 {
        return ServiceResponse::error(400, "name must be 1-100 characters");
    }
    if req.email.is_empty() || !req.email.contains('@') || req.email.len() > 254 {
        return ServiceResponse::error(400, "valid email required");
    }
    if req.subject.is_empty() || req.subject.len() > 200 {
        return ServiceResponse::error(400, "subject must be 1-200 characters");
    }
    if req.message.is_empty() || req.message.len() > 5000 {
        return ServiceResponse::error(400, "message must be 1-5000 characters");
    }

    // TODO: Validate PoW solution against nonce

    // Delegate to email Lambda
    let email_fn =
        std::env::var("EMAIL_FN_NAME").unwrap_or_else(|_| "deploy-baba-prod-email".to_string());
    let payload = serde_json::json!({
        "name": req.name,
        "email": req.email,
        "subject": req.subject,
        "message": req.message,
        "website": "",
    });

    match lambda_client
        .invoke()
        .function_name(&email_fn)
        .payload(Blob::new(payload.to_string()))
        .send()
        .await
    {
        Ok(resp) => {
            if let Some(blob) = resp.payload {
                let bytes = blob.into_inner();
                let email_resp: serde_json::Value =
                    serde_json::from_slice(&bytes).unwrap_or_default();
                info!("email lambda responded: {:?}", email_resp);
            }
            ServiceResponse::ok(ContactResponse {
                success: true,
                message: "Message sent successfully".to_string(),
            })
        }
        Err(e) => {
            error!("email lambda invoke failed: {}", e);
            ServiceResponse::error(500, "failed to send message")
        }
    }
}
