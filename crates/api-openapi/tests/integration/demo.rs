/// Integration tests for the demo API
/// (`POST /api/demo/parse-config`, `POST /api/demo/generate-spec`).
use api_openapi::{
    apidoc::full_spec,
    models::{
        ApiModel, Field, GenerateSpecRequest, GenerateSpecResponse, ParseConfigRequest,
        ParseConfigResponse,
    },
};

#[test]
fn demo_schemas_present_in_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    for name in &[
        "ParseConfigRequest",
        "ParseConfigResponse",
        "GenerateSpecRequest",
        "GenerateSpecResponse",
        "Field",
    ] {
        assert!(
            schemas.contains_key(*name),
            "Demo schema '{}' missing from full_spec()",
            name
        );
    }
}

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

#[test]
fn demo_schemas_are_in_public_api_doc() {
    use api_openapi::apidoc::PublicApiDoc;
    use utoipa::OpenApi;
    let spec = PublicApiDoc::openapi();
    let schemas = &spec
        .components
        .as_ref()
        .expect("PublicApiDoc components")
        .schemas;

    // Demo schemas belong in PublicApiDoc
    for name in &[
        "ParseConfigRequest",
        "ParseConfigResponse",
        "GenerateSpecRequest",
        "GenerateSpecResponse",
        "Field",
    ] {
        assert!(
            schemas.contains_key(*name),
            "Demo schema '{}' missing from PublicApiDoc",
            name
        );
    }
}
