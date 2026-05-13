//! Integration tests for api-openapi
//!
//! Tests metadata correctness and basic spec structure.

use api_openapi::OpenApiMetadata;

// Test 1: OpenApiMetadata field validation
#[test]
fn test_openapi_metadata_fields() {
    let metadata = OpenApiMetadata {
        generator: "test-generator".to_string(),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        validated: true,
        path_count: 10,
        schema_count: 25,
    };

    assert_eq!(metadata.generator, "test-generator");
    assert_eq!(metadata.generated_at, "2026-01-01T00:00:00Z");
    assert!(metadata.validated);
    assert_eq!(metadata.path_count, 10);
    assert_eq!(metadata.schema_count, 25);
}

// Test 2: OpenApi basic structure
#[test]
fn test_openapi_basic_structure() {
    use utoipa::openapi::{Info, OpenApi};

    let info = Info::new("Test API", "1.0.0");
    let openapi = OpenApi::new(info, utoipa::openapi::Paths::new());

    assert_eq!(openapi.info.title, "Test API");
    assert_eq!(openapi.info.version, "1.0.0");
}

// Test 3: OpenApi serialization/deserialization
#[test]
fn test_openapi_serde_round_trip() {
    use utoipa::openapi::{Info, OpenApi};

    let info = Info::new("Test API", "2.0.0");
    let original = OpenApi::new(info, utoipa::openapi::Paths::new());

    let json_str = serde_json::to_string(&original).expect("Should serialize");
    let deserialized: OpenApi = serde_json::from_str(&json_str).expect("Should deserialize");

    assert_eq!(deserialized.info.title, original.info.title);
    assert_eq!(deserialized.info.version, original.info.version);
}

// Test 4: OpenApiMetadata serde round-trip
#[test]
fn test_metadata_serde_round_trip() {
    let original = OpenApiMetadata {
        generator: "integration-test".to_string(),
        generated_at: "2026-05-09T12:00:00Z".to_string(),
        validated: true,
        path_count: 0,
        schema_count: 5,
    };

    let json_str = serde_json::to_string(&original).expect("Should serialize");
    let deserialized: OpenApiMetadata =
        serde_json::from_str(&json_str).expect("Should deserialize");

    assert_eq!(deserialized.generator, original.generator);
    assert_eq!(deserialized.generated_at, original.generated_at);
    assert_eq!(deserialized.validated, original.validated);
    assert_eq!(deserialized.path_count, original.path_count);
    assert_eq!(deserialized.schema_count, original.schema_count);
}
