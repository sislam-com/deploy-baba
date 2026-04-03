use askama::Template;
use aws_sdk_lambda::primitives::Blob;
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use crate::db::{load_social_links, Db, SocialLink};

type HmacSha256 = Hmac<Sha256>;

// ─── POW_SECRET ───────────────────────────────────────────────────────────────
//
// Loaded once per cold start via `init_pow_secret()`. Falls back to a local
// dev default when `POW_SECRET_ARN` is not set (i.e. running locally).

static POW_SECRET: OnceLock<[u8; 32]> = OnceLock::new();

/// Fetch the PoW HMAC key from Secrets Manager (or fall back to dev default).
/// Call once at Lambda cold start before the router is built.
pub async fn init_pow_secret() {
    if POW_SECRET.get().is_some() {
        return; // already initialised (shouldn't happen at cold start, but be safe)
    }

    let secret = match std::env::var("POW_SECRET_ARN") {
        Ok(arn) => {
            let config = aws_config::load_from_env().await;
            let client = aws_sdk_secretsmanager::Client::new(&config);
            match client.get_secret_value().secret_id(&arn).send().await {
                Ok(resp) => resp
                    .secret_string()
                    .unwrap_or("dev-secret-change-me")
                    .to_string(),
                Err(e) => {
                    tracing::error!("Failed to fetch POW_SECRET from Secrets Manager: {}", e);
                    "dev-secret-change-me".to_string()
                }
            }
        }
        Err(_) => {
            tracing::warn!("POW_SECRET_ARN not set — using dev default");
            "dev-secret-change-me".to_string()
        }
    };

    let mut key = [0u8; 32];
    key.copy_from_slice(&Sha256::digest(secret.as_bytes()));
    POW_SECRET.set(key).ok();
}

fn pow_secret() -> &'static [u8; 32] {
    POW_SECRET.get().expect("init_pow_secret() must be called before serving requests")
}

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

// ─── Nonce tracker (replay protection) ───────────────────────────────────────

struct NonceTracker {
    used: Mutex<HashMap<String, Instant>>,
}

static USED_NONCES: OnceLock<NonceTracker> = OnceLock::new();

fn nonce_tracker() -> &'static NonceTracker {
    USED_NONCES.get_or_init(|| NonceTracker {
        used: Mutex::new(HashMap::new()),
    })
}

impl NonceTracker {
    /// Returns true and marks the nonce as used; false if already used.
    fn check_and_mark(&self, nonce: &str) -> bool {
        let mut map = self.used.lock().unwrap();
        let now = Instant::now();
        // Evict entries older than challenge TTL (5 min)
        map.retain(|_, t| now.duration_since(*t) < Duration::from_secs(300));
        if map.contains_key(nonce) {
            return false;
        }
        map.insert(nonce.to_string(), now);
        true
    }
}

// ─── PoW helpers ──────────────────────────────────────────────────────────────

const POW_DIFFICULTY: u32 = 18;
const CHALLENGE_TTL_SECS: i64 = 300;

fn compute_hmac(secret: &[u8], nonce: &str, timestamp: i64, difficulty: u32) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(nonce.as_bytes());
    mac.update(b":");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b":");
    mac.update(difficulty.to_string().as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn has_leading_zero_bits(hash: &[u8], bits: u32) -> bool {
    let full_bytes = (bits / 8) as usize;
    let remaining_bits = bits % 8;
    for i in 0..full_bytes {
        if i >= hash.len() || hash[i] != 0 {
            return false;
        }
    }
    if remaining_bits > 0 {
        let mask = 0xFF_u8 << (8 - remaining_bits);
        if full_bytes >= hash.len() || (hash[full_bytes] & mask) != 0 {
            return false;
        }
    }
    true
}

fn verify_pow(nonce: &str, timestamp: i64, solution: u64, signature: &str) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    if (now - timestamp).abs() > CHALLENGE_TTL_SECS {
        return Err("Challenge expired".to_string());
    }

    let expected = compute_hmac(pow_secret(), nonce, timestamp, POW_DIFFICULTY);
    if expected != signature {
        return Err("Invalid challenge signature".to_string());
    }

    if !nonce_tracker().check_and_mark(nonce) {
        return Err("Challenge already used".to_string());
    }

    let input = format!("{}:{}", nonce, solution);
    let hash = Sha256::digest(input.as_bytes());
    if !has_leading_zero_bits(&hash, POW_DIFFICULTY) {
        return Err("Invalid proof of work".to_string());
    }

    Ok(())
}

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate {
    social_links: Vec<SocialLink>,
}

#[derive(Serialize)]
struct ChallengeResponse {
    nonce: String,
    timestamp: i64,
    difficulty: u32,
    signature: String,
}

#[derive(Deserialize, Serialize)]
pub struct ContactSubmitRequest {
    name: String,
    email: String,
    subject: String,
    message: String,
    #[serde(default)]
    website: String,
    pow_nonce: String,
    pow_timestamp: i64,
    pow_solution: u64,
    pow_signature: String,
}

// Strip PoW fields before forwarding to email Lambda
#[derive(Serialize)]
struct EmailPayload<'a> {
    name: &'a str,
    email: &'a str,
    subject: &'a str,
    message: &'a str,
    website: &'a str,
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

pub async fn challenge_issue() -> impl IntoResponse {
    use rand::RngCore;
    let mut nonce_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = hex::encode(nonce_bytes);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let signature = compute_hmac(pow_secret(), &nonce, timestamp, POW_DIFFICULTY);

    Json(ChallengeResponse {
        nonce,
        timestamp,
        difficulty: POW_DIFFICULTY,
        signature,
    })
}

pub async fn contact_submit(
    headers: HeaderMap,
    Json(req): Json<ContactSubmitRequest>,
) -> impl IntoResponse {
    // Honeypot — silently succeed for bots
    if !req.website.is_empty() {
        return (
            StatusCode::OK,
            Json(ContactResponse {
                success: true,
                message: "Message sent!".to_string(),
            }),
        )
            .into_response();
    }

    // Proof-of-work verification
    if let Err(msg) = verify_pow(
        &req.pow_nonce,
        req.pow_timestamp,
        req.pow_solution,
        &req.pow_signature,
    ) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ContactResponse {
                success: false,
                message: msg,
            }),
        )
            .into_response();
    }

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

    // Invoke email Lambda (PoW fields stripped from payload)
    let fn_name =
        std::env::var("EMAIL_LAMBDA_NAME").unwrap_or_else(|_| "deploy-baba-email".to_string());

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_lambda::Client::new(&config);

    let email_payload = EmailPayload {
        name: &req.name,
        email: &req.email,
        subject: &req.subject,
        message: &req.message,
        website: &req.website,
    };

    let payload = match serde_json::to_vec(&email_payload) {
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
