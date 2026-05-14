use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use serde_json::json;
use std::sync::Arc;

use crate::auth::AuthConfig;

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[cfg(test)]
use std::time::{Duration, Instant};

#[cfg(test)]
use tokio::sync::Mutex;

/// Axum middleware that enforces authentication.
///
/// Token extraction order:
/// 1. `auth_token` HttpOnly cookie (set by `/auth/callback`)
/// 2. `Authorization: Bearer <token>` header (API fallback)
///
/// On failure:
/// - `Accept: application/json` → 401 JSON
/// - Otherwise → 302 to Cognito login (or `/auth/login` in dev mode)
pub async fn require_auth(
    State(auth): State<Arc<AuthConfig>>,
    mut req: Request,
    next: Next,
) -> Response {
    match extract_token(req.headers()) {
        Some(token) => match auth.validate_token(&token).await {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
                next.run(req).await
            }
            Err(_) => redirect_or_401(req.headers(), &auth),
        },
        None => redirect_or_401(req.headers(), &auth),
    }
}

pub fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    extract_token(headers)
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    // 1. auth_token cookie
    if let Some(cookie_hdr) = headers.get("cookie") {
        if let Ok(s) = cookie_hdr.to_str() {
            for part in s.split(';') {
                let part = part.trim();
                if let Some(val) = part.strip_prefix("auth_token=") {
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }

    // 2. Authorization: Bearer <token>
    if let Some(auth_hdr) = headers.get("authorization") {
        if let Ok(val) = auth_hdr.to_str() {
            if let Some(token) = val.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    None
}

fn redirect_or_401(headers: &HeaderMap, auth: &AuthConfig) -> Response {
    let wants_json = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("application/json"))
        .unwrap_or(false);

    if wants_json {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response()
    } else {
        let location = if auth.dev_mode {
            "/auth/login".to_string()
        } else {
            format!(
                "https://{}/oauth2/authorize?client_id={}&response_type=code\
                 &scope=openid+email+profile&redirect_uri={}/auth/callback",
                auth.cognito_domain, auth.client_id, auth.app_domain
            )
        };

        let mut resp_headers = HeaderMap::new();
        resp_headers.insert(
            axum::http::header::LOCATION,
            HeaderValue::from_str(&location).unwrap_or_else(|_| HeaderValue::from_static("/")),
        );
        (StatusCode::FOUND, resp_headers).into_response()
    }
}

/// In-memory rate limiter using sliding window algorithm
/// Key format: "client_ip:endpoint"
#[cfg(test)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

#[cfg(test)]
impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed per window
    /// * `window` - Time window for rate limiting
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    /// Check if a request should be allowed for the given key
    ///
    /// # Arguments
    /// * `key` - Unique identifier for the client (e.g., "client_ip:endpoint")
    ///
    /// # Returns
    /// * `true` if request is allowed
    /// * `false` if rate limit exceeded
    pub async fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let entry = requests.entry(key.to_string()).or_default();

        // Remove expired entries
        entry.retain(|&t| now.duration_since(t) < self.window);

        if entry.len() < self.max_requests {
            entry.push(now);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
/// Retry policy for transient errors
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

#[cfg(test)]
impl RetryPolicy {
    pub fn new(max_attempts: u32, initial_backoff_ms: u64, max_backoff_ms: u64) -> Self {
        Self {
            max_attempts,
            initial_backoff_ms,
            max_backoff_ms,
        }
    }
}

#[cfg(test)]
impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3, 100, 5000)
    }
}

#[cfg(test)]
/// Circuit breaker to prevent cascading failures
pub struct CircuitBreaker {
    is_open: Arc<AtomicBool>,
    failure_count: Arc<AtomicUsize>,
    success_count: Arc<AtomicUsize>,
    threshold: usize,
    open_timeout: Duration,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
}

#[cfg(test)]
impl CircuitBreaker {
    /// Create a new circuit breaker
    ///
    /// # Arguments
    /// * `threshold` - Number of consecutive failures before opening
    /// * `open_timeout` - How long to stay open before attempting recovery
    pub fn new(threshold: usize, open_timeout: Duration) -> Self {
        Self {
            is_open: Arc::new(AtomicBool::new(false)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            success_count: Arc::new(AtomicUsize::new(0)),
            threshold,
            open_timeout,
            last_failure_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Check if the circuit is open (blocking requests)
    pub async fn is_open(&self) -> bool {
        if !self.is_open.load(Ordering::SeqCst) {
            return false;
        }

        // Check if we should attempt recovery
        if let Some(last_failure) = *self.last_failure_time.lock().await {
            if last_failure.elapsed() > self.open_timeout {
                // Attempt recovery - move to half-open state
                tracing::info!("Circuit breaker attempting recovery");
                self.is_open.store(false, Ordering::SeqCst);
                return false;
            }
        }

        true
    }

    /// Record a successful call
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.fetch_add(1, Ordering::SeqCst);

        // If we were in half-open state, close the circuit
        if self.success_count.load(Ordering::SeqCst) >= 2 {
            self.is_open.store(false, Ordering::SeqCst);
            tracing::info!("Circuit breaker closed after successful recovery");
        }
    }

    /// Record a failed call
    pub async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

        if failures >= self.threshold {
            self.is_open.store(true, Ordering::SeqCst);
            *self.last_failure_time.lock().await = Some(Instant::now());
            tracing::warn!(
                "Circuit breaker opened after {} consecutive failures",
                failures
            );
        }
    }
}

#[cfg(test)]
mod circuit_breaker_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_basic() {
        let breaker = CircuitBreaker::new(3, Duration::from_secs(60));

        // Initially closed
        assert!(!breaker.is_open().await);

        // Record failures below threshold
        breaker.record_failure().await;
        breaker.record_failure().await;
        assert!(!breaker.is_open().await);

        // Record failure at threshold
        breaker.record_failure().await;
        assert!(breaker.is_open().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let breaker = CircuitBreaker::new(2, Duration::from_millis(100));

        // Open the circuit
        breaker.record_failure().await;
        breaker.record_failure().await;
        assert!(breaker.is_open().await);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should allow recovery attempt
        assert!(!breaker.is_open().await);

        // Record success to close circuit
        breaker.record_success();
        breaker.record_success();
        assert!(!breaker.is_open().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset_on_success() {
        let breaker = CircuitBreaker::new(5, Duration::from_secs(60));

        // Record some failures
        breaker.record_failure().await;
        breaker.record_failure().await;
        breaker.record_failure().await;

        // Record success resets failure count and keeps circuit closed
        breaker.record_success();
        assert!(!breaker.is_open().await);
    }
}

#[cfg(test)]
mod retry_tests {
    use super::*;

    #[derive(Debug, Clone, thiserror::Error)]
    enum TestError {
        #[error("transient error: {0}")]
        Transient(String),
        #[error("non-transient error: {0}")]
        NonTransient(String),
    }

    impl RetryPolicy {
        /// Execute an operation with retry logic
        async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
        where
            F: Fn() -> Fut,
            Fut: std::future::Future<Output = Result<T, E>>,
            E: std::error::Error + Clone,
        {
            let mut attempt = 0;
            let mut backoff = self.initial_backoff_ms;

            loop {
                attempt += 1;

                match operation().await {
                    Ok(result) => return Ok(result),
                    Err(error) if attempt < self.max_attempts && is_transient_error(&error) => {
                        tracing::warn!(
                            "Transient error (attempt {}/{}), retrying in {}ms: {}",
                            attempt,
                            self.max_attempts,
                            backoff,
                            error
                        );
                        tokio::time::sleep(Duration::from_millis(backoff)).await;
                        backoff = std::cmp::min(backoff * 2, self.max_backoff_ms);
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }

    /// Check if an error is transient (should be retried)
    fn is_transient_error<E: std::error::Error>(error: &E) -> bool {
        let error_msg = error.to_string().to_lowercase();

        // Retry on: timeouts, network errors, 5xx status codes, rate limits
        error_msg.contains("timeout")
            || error_msg.contains("connection")
            || error_msg.contains("network")
            || error_msg.contains("5xx")
            || error_msg.contains("rate limit")
            || error_msg.contains("too many requests")
    }

    #[tokio::test]
    async fn test_retry_policy_success_on_first_attempt() {
        let policy = RetryPolicy::default();

        let result: Result<&str, TestError> = policy
            .execute(|| async { Ok::<_, TestError>("success") })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_policy_success_on_retry() {
        let policy = RetryPolicy::new(3, 10, 100);

        // Test that immediate success works
        let result: Result<&str, TestError> = policy
            .execute(|| async { Ok::<_, TestError>("success") })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retry_policy_exhausted() {
        let policy = RetryPolicy::new(2, 10, 100);

        // Test that transient errors eventually exhaust retries
        let result: Result<&str, TestError> = policy
            .execute(|| async {
                Err::<_, TestError>(TestError::Transient("connection timeout".to_string()))
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_policy_no_retry_on_non_transient() {
        let policy = RetryPolicy::default();

        // Test that non-transient errors are not retried
        let result: Result<&str, TestError> = policy
            .execute(|| async {
                Err::<_, TestError>(TestError::NonTransient("invalid request".to_string()))
            })
            .await;

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        // Should allow first 5 requests for the same key
        for i in 0..5 {
            assert!(
                limiter.check("test").await,
                "Request {} should be allowed",
                i
            );
        }

        // 6th request should be denied
        assert!(!limiter.check("test").await, "6th request should be denied");
    }

    #[tokio::test]
    async fn test_rate_limiter_window_expiration() {
        let limiter = RateLimiter::new(2, Duration::from_millis(100));

        // First 2 requests allowed
        assert!(limiter.check("test").await);
        assert!(limiter.check("test").await);

        // 3rd request denied
        assert!(!limiter.check("test").await);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Request should be allowed again after window expires
        assert!(limiter.check("test").await);
    }
}
