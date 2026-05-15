//! Telemetry and observability module for deploy-baba
//!
//! Provides structured logging with tracing.
//! Follows zero-cost philosophy — no CloudWatch Metrics cost.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize telemetry with structured JSON logging
///
/// This should be called early in main() before any other logging setup.
/// Uses JSON format for CloudWatch Logs parsing and includes request IDs
/// for distributed tracing.
pub fn init_telemetry() {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    // Use JSON format in Lambda, human-readable format for local dev
    let is_lambda = std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok();

    let fmt_layer = if is_lambda {
        fmt::layer().json().boxed() // Structured JSON logs for CloudWatch
    } else {
        fmt::layer().without_time().boxed() // Human-readable for local dev
    };

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::new(rust_log))
        .init();
}
