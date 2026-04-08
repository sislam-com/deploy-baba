use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::info;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ContactRequest {
    name: String,
    email: String,
    subject: String,
    message: String,
    /// Honeypot — must be empty. Bots fill all fields.
    #[serde(default)]
    website: String,
}

#[derive(Serialize)]
struct ContactResponse {
    success: bool,
    message: String,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

async fn handler(event: LambdaEvent<ContactRequest>) -> Result<ContactResponse, Error> {
    let req = event.payload;

    // Honeypot check — bots fill all fields
    if !req.website.is_empty() {
        return Ok(ContactResponse {
            success: true,
            message: "Message sent successfully".to_string(),
        });
    }

    // Input validation
    if req.name.is_empty() || req.name.len() > 100 {
        return Ok(ContactResponse {
            success: false,
            message: "Name must be 1–100 characters".to_string(),
        });
    }
    if req.email.is_empty() || req.email.len() > 254 || !req.email.contains('@') {
        return Ok(ContactResponse {
            success: false,
            message: "Valid email address required".to_string(),
        });
    }
    if req.subject.is_empty() || req.subject.len() > 200 {
        return Ok(ContactResponse {
            success: false,
            message: "Subject must be 1–200 characters".to_string(),
        });
    }
    if req.message.is_empty() || req.message.len() > 5000 {
        return Ok(ContactResponse {
            success: false,
            message: "Message must be 1–5000 characters".to_string(),
        });
    }

    let from_email = std::env::var("SES_FROM_EMAIL").unwrap_or_default();
    let to_email = std::env::var("CONTACT_TO_EMAIL").unwrap_or_default();

    if from_email.is_empty() || to_email.is_empty() {
        // Not configured — local dev mode
        info!("Email not configured (SES_FROM_EMAIL or CONTACT_TO_EMAIL unset); skipping send");
        return Ok(ContactResponse {
            success: true,
            message: "Message sent successfully".to_string(),
        });
    }

    // Compose and send email
    let email_subject = format!("[Contact Form] {}", req.subject);
    let email_body = format!(
        "Name: {}\nEmail: {}\nSubject: {}\n\n---\n\n{}",
        req.name, req.email, req.subject, req.message
    );

    let config = aws_config::load_from_env().await;
    let ses = aws_sdk_sesv2::Client::new(&config);

    let result = ses
        .send_email()
        .from_email_address(&from_email)
        .destination(Destination::builder().to_addresses(&to_email).build())
        .content(
            EmailContent::builder()
                .simple(
                    Message::builder()
                        .subject(Content::builder().data(&email_subject).build()?)
                        .body(
                            Body::builder()
                                .text(Content::builder().data(&email_body).build()?)
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .send()
        .await;

    match result {
        Ok(_) => {
            info!(from = %req.email, "contact form email sent");
            Ok(ContactResponse {
                success: true,
                message: "Message sent successfully".to_string(),
            })
        }
        Err(e) => {
            tracing::error!(error = ?e, "SES send_email failed");
            Ok(ContactResponse {
                success: false,
                message: "Failed to send message".to_string(),
            })
        }
    }
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .json()
        .init();

    run(service_fn(handler)).await
}
