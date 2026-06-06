//! Library interface for services/ui
//!
//! Exposes the router builder and state types for integration testing.

pub mod auth;
pub mod db;
pub mod middleware;

pub mod openapi;
pub mod router;
pub mod routes;
pub mod state;
pub mod tailor;
pub mod telemetry;

// Re-export commonly used types
pub use router::build;
pub use state::AppState;
