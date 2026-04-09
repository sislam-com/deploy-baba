/// Integration tests for the health API (`GET /health`).
use api_openapi::{apidoc::full_spec, models::ApiModel, models::HealthResponse};

#[test]
fn health_response_schema_present_in_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;
    assert!(
        schemas.contains_key("HealthResponse"),
        "HealthResponse schema missing from full_spec()"
    );
}

#[test]
fn health_response_example_has_required_fields() {
    let example = HealthResponse::example();
    assert!(
        !example.status.is_empty(),
        "HealthResponse.status should not be empty"
    );
    assert!(
        !example.version.is_empty(),
        "HealthResponse.version should not be empty"
    );
}

#[test]
fn health_response_example_status_is_ok() {
    let example = HealthResponse::example();
    assert_eq!(
        example.status, "ok",
        "HealthResponse canonical example should have status='ok'"
    );
}

#[test]
fn health_response_schema_name_matches_registry() {
    assert_eq!(HealthResponse::schema_name(), "HealthResponse");
}
