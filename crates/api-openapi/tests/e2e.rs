#[path = "e2e/admin_spec.rs"]
mod admin_spec;
#[path = "e2e/coverage.rs"]
mod coverage;
/// End-to-end tests for cross-cutting API behaviour in relation to the UI.
///
/// Verifies that the public and admin spec split behaves correctly end to end:
/// - The public spec (`GET /api/openapi.json`) contains no admin paths/schemas/schemes.
/// - The admin spec (`GET /api/openapi-admin.json`) is the full spec with all admin info.
/// - No unregistered API structs exist in `services/ui/src/routes/`.
///
/// Running: `cargo test -p api-openapi --test e2e`
#[path = "e2e/public_spec.rs"]
mod public_spec;
