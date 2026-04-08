#[path = "integration/about.rs"]
mod about;
#[path = "integration/admin.rs"]
mod admin;
#[path = "integration/competencies.rs"]
mod competencies;
#[path = "integration/contact.rs"]
mod contact;
#[path = "integration/demo.rs"]
mod demo;
/// Per-API integration tests.
///
/// Each submodule tests a specific API surface area — verifying that the
/// correct schemas are present in `full_spec()` and that each model's
/// structure matches the expected API contract.
///
/// Running: `cargo test -p api-openapi --test integration`
#[path = "integration/health.rs"]
mod health;
#[path = "integration/jobs.rs"]
mod jobs;
#[path = "integration/registry.rs"]
mod registry;
#[path = "integration/social.rs"]
mod social;
