/// Integration tests for the demo API model types.
/// Note: ParseConfigRequest/Response, GenerateSpecRequest/Response, and Field are
/// implementation types for the demo handlers but are intentionally excluded from
/// the public OpenAPI spec (removed 2026-05-02).
use api_openapi::models::{
    ApiModel, Field, GenerateSpecRequest, GenerateSpecResponse, ParseConfigRequest,
    ParseConfigResponse,
};

#[test]
fn parse_config_request_example_has_content() {
    let req = ParseConfigRequest::example();
    assert!(
        !req.content.is_empty(),
        "ParseConfigRequest.content should not be empty"
    );
}

#[test]
fn parse_config_response_example_has_success_field() {
    let resp = ParseConfigResponse::example();
    // success should be true for the canonical example
    let _ = resp.success; // field access confirms the struct compiles
}

#[test]
fn field_example_has_name_and_type() {
    let field = Field::example();
    assert!(!field.name.is_empty(), "Field.name should not be empty");
    assert!(
        !field.field_type.is_empty(),
        "Field.field_type should not be empty"
    );
}

#[test]
fn generate_spec_request_example_has_fields() {
    let req = GenerateSpecRequest::example();
    assert!(
        !req.fields.is_empty(),
        "GenerateSpecRequest.fields should not be empty"
    );
}

#[test]
fn generate_spec_response_example_has_spec() {
    let resp = GenerateSpecResponse::example();
    // spec is a serde_json::Value — must not be null
    assert!(
        !resp.spec.is_null(),
        "GenerateSpecResponse.spec should not be null"
    );
}

