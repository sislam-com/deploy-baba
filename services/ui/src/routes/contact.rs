use askama::Template;
use aws_sdk_lambda::primitives::Blob;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use crate::db::{load_social_links, Db, SocialLink};

// ─── Rate limiter ─────────────────────────────────────────────────────────────

struct RateLimiter {
    entries: Mutex<HashMap<String, (u32, Instant)>>,
}

static CONTACT_RATE_LIMITER: OnceLock<RateLimiter> = OnceLock::new();

fn rate_limiter() -> &'static RateLimiter {
    CONTACT_RATE_LIMITER.get_or_init(|| RateLimiter {
        entries: Mutex::new(HashMap::new()),
    })
}

impl RateLimiter {
    fn check_and_increment(&self, ip: &str) -> bool {
        let mut map = self.entries.lock().unwrap();
        let now = Instant::now();
        let window = Duration::from_secs(3600);
        let max = 3u32;
        let entry = map.entry(ip.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) >= window {
            *entry = (1, now);
            true
        } else if entry.0 < max {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate {
    social_links: Vec<SocialLink>,
}

#[derive(Deserialize, Serialize)]
pub struct ContactSubmitRequest {
    name: String,
    email: String,
    subject: String,
    message: String,
    #[serde(default)]
    website: String,
}

#[derive(Deserialize, Serialize)]
struct ContactResponse {
    success: bool,
    message: String,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

pub async fn contact_page(State(db): State<Arc<Db>>) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();
    let social_links = load_social_links(&conn);
    ContactTemplate { social_links }
}

pub async fn contact_submit(
    headers: HeaderMap,
    Query(req): Query<ContactSubmitRequest>,
) -> impl IntoResponse {
    // Rate limit by client IP
    let source_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim())
        .unwrap_or("unknown");

    if !rate_limiter().check_and_increment(source_ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ContactResponse {
                success: false,
                message: "Too many requests. Please try again later.".to_string(),
            }),
        )
            .into_response();
    }

    // Invoke email Lambda directly via SDK (no public HTTP endpoint needed)
    let fn_name =
        std::env::var("EMAIL_LAMBDA_NAME").unwrap_or_else(|_| "deploy-baba-email".to_string());

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_lambda::Client::new(&config);

    let payload = match serde_json::to_vec(&req) {
        Ok(b) => Blob::new(b),
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ContactResponse {
                    success: false,
                    message: "Invalid request".to_string(),
                }),
            )
                .into_response();
        }
    };

    match client
        .invoke()
        .function_name(&fn_name)
        .payload(payload)
        .send()
        .await
    {
        Ok(resp) => {
            if let Some(blob) = resp.payload() {
                if let Ok(r) = serde_json::from_slice::<ContactResponse>(blob.as_ref()) {
                    let status = if r.success {
                        StatusCode::OK
                    } else {
                        StatusCode::INTERNAL_SERVER_ERROR
                    };
                    return (status, Json(r)).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ContactResponse {
                    success: false,
                    message: "Failed to send message".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("email lambda invocation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ContactResponse {
                    success: false,
                    message: "Failed to send message".to_string(),
                }),
            )
                .into_response()
        }
    }
}
