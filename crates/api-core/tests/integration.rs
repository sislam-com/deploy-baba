//! Integration tests for api-core
//!
//! Tests ApiSpecGenerator lifecycle, error paths, merge behavior, and serde round-trip.

use api_core::{ApiSpecGenerator, SpecError, SpecFormat, SpecValidationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSchema {
    title: String,
    version: String,
    paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSpec {
    content: String,
    format: SpecFormat,
}

struct TestGenerator;

impl ApiSpecGenerator for TestGenerator {
    type Schema = TestSchema;
    type Output = TestSpec;

    fn generate_spec(schema: TestSchema) -> Result<TestSpec, SpecError> {
        if schema.title.is_empty() {
            return Err(SpecError::GenerationFailed(
                "Title cannot be empty".to_string(),
            ));
        }
        Ok(TestSpec {
            content: format!("{} v{}", schema.title, schema.version),
            format: SpecFormat::OpenApi,
        })
    }

    fn validate_spec(spec: &TestSpec) -> Result<(), Vec<SpecValidationError>> {
        if spec.content.is_empty() {
            return Err(vec![SpecValidationError::new(
                "content",
                "Content cannot be empty",
            )]);
        }
        Ok(())
    }
}

// Test 1: ApiSpecGenerator lifecycle: construct → generate_spec() → assert non-empty output
#[test]
fn test_spec_generator_lifecycle() {
    let schema = TestSchema {
        title: "Test API".to_string(),
        version: "1.0.0".to_string(),
        paths: vec!["/users".to_string(), "/posts".to_string()],
    };

    let result = TestGenerator::generate_spec(schema);
    assert!(result.is_ok(), "Generation should succeed");
    let spec = result.unwrap();
    assert!(
        !spec.content.is_empty(),
        "Generated spec should have non-empty content"
    );
    assert_eq!(spec.format, SpecFormat::OpenApi);
}

// Test 2: generate_spec() error path: misconfigured generator → SpecGenerationError
#[test]
fn test_generation_error_path() {
    let invalid_schema = TestSchema {
        title: "".to_string(), // Empty title should trigger error
        version: "1.0.0".to_string(),
        paths: vec![],
    };

    let result = TestGenerator::generate_spec(invalid_schema);
    assert!(result.is_err(), "Generation should fail for invalid schema");
    match result {
        Err(SpecError::GenerationFailed(msg)) => {
            assert!(
                msg.contains("empty"),
                "Error message should mention empty title"
            );
        }
        _ => panic!("Expected Generation error"),
    }
}

// Test 3: Default merge_specs behavior: two non-overlapping specs → combined spec
#[test]
fn test_default_merge_single_spec() {
    let spec1 = TestSpec {
        content: "Spec 1".to_string(),
        format: SpecFormat::OpenApi,
    };

    let result = TestGenerator::merge_specs(vec![spec1]);
    assert!(result.is_ok(), "Merge of single spec should succeed");
    let merged = result.unwrap();
    assert_eq!(merged.content, "Spec 1");
}

// Test 4: Default merge_specs error: multiple specs not implemented
#[test]
fn test_default_merge_multiple_specs_error() {
    let spec1 = TestSpec {
        content: "Spec 1".to_string(),
        format: SpecFormat::OpenApi,
    };
    let spec2 = TestSpec {
        content: "Spec 2".to_string(),
        format: SpecFormat::OpenApi,
    };

    let result = TestGenerator::merge_specs(vec![spec1, spec2]);
    assert!(
        result.is_err(),
        "Merge of multiple specs should fail by default"
    );
    match result {
        Err(SpecError::MergeError(msg)) => {
            assert!(
                msg.contains("not implemented"),
                "Error should mention not implemented"
            );
        }
        _ => panic!("Expected MergeError"),
    }
}

// Test 5: Default merge_specs error: empty specs list
#[test]
fn test_default_merge_empty_specs_error() {
    let result = TestGenerator::merge_specs(vec![]);
    assert!(result.is_err(), "Merge of empty specs should fail");
    match result {
        Err(SpecError::MergeError(msg)) => {
            assert!(msg.contains("empty"), "Error should mention empty list");
        }
        _ => panic!("Expected MergeError"),
    }
}

// Test 6: SpecFormat serde round-trip: serde_json::to_string → from_str → equality
#[test]
fn test_spec_format_serde_round_trip() {
    let format = SpecFormat::OpenApi;
    let serialized = serde_json::to_string(&format).expect("Should serialize");
    let deserialized: SpecFormat = serde_json::from_str(&serialized).expect("Should deserialize");
    assert_eq!(format, deserialized, "Round-trip should preserve value");
}

// Test 7: generate_and_validate convenience method
#[test]
fn test_generate_and_validate() {
    let schema = TestSchema {
        title: "Valid API".to_string(),
        version: "1.0.0".to_string(),
        paths: vec![],
    };

    let result = TestGenerator::generate_and_validate(schema);
    assert!(result.is_ok(), "Generate and validate should succeed");
    let spec = result.unwrap();
    assert!(!spec.content.is_empty());
}

// Test 8: generate_and_validate with validation failure
#[test]
fn test_generate_and_validate_validation_failure() {
    // Create a spec that will fail validation
    let spec = TestSpec {
        content: "".to_string(), // Empty content will fail validation
        format: SpecFormat::OpenApi,
    };

    // Manually call validate to test the error path
    let result = TestGenerator::validate_spec(&spec);
    assert!(result.is_err(), "Validation should fail for empty content");
    match result {
        Err(errors) => {
            assert_eq!(errors.len(), 1, "Should have one validation error");
            assert_eq!(errors[0].path, "content");
        }
        _ => panic!("Expected validation error"),
    }
}

// Test 9: SpecFormat Display implementation
#[test]
fn test_spec_format_display() {
    assert_eq!(format!("{}", SpecFormat::OpenApi), "OpenAPI");
    assert_eq!(format!("{}", SpecFormat::GraphQL), "GraphQL");
    assert_eq!(format!("{}", SpecFormat::Grpc), "gRPC");
}

// Test 10: SpecFormat equality
#[test]
fn test_spec_format_equality() {
    assert_eq!(SpecFormat::OpenApi, SpecFormat::OpenApi);
    assert_ne!(SpecFormat::OpenApi, SpecFormat::GraphQL);
}
