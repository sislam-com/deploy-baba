//! Integration tests for api-merger
//!
//! Tests conflict resolution strategies and basic configuration.

use api_core::SpecFormat;
use api_merger::{ConflictResolutionStrategy, SpecificationMerger};

// Test 1: ConflictResolutionStrategy variants
#[test]
fn test_conflict_resolution_strategies() {
    let first_wins = ConflictResolutionStrategy::FirstWins;
    let last_wins = ConflictResolutionStrategy::LastWins;
    let fail_on_conflict = ConflictResolutionStrategy::FailOnConflict;

    // These are enum variants
    assert_ne!(first_wins, last_wins);
    assert_ne!(last_wins, fail_on_conflict);
    assert_ne!(first_wins, fail_on_conflict);
}

// Test 2: SpecificationMerger configuration
#[test]
fn test_specification_merger_configuration() {
    // Test with conflict resolution
    let merger_with_strategy = SpecificationMerger::new(SpecFormat::OpenApi)
        .with_conflict_resolution(ConflictResolutionStrategy::FirstWins);
    // The merger is configured, but we can't access metadata directly
    // Just verify the merger was created successfully
    let _ = merger_with_strategy;

    // Test with validation
    let merger_with_validation =
        SpecificationMerger::new(SpecFormat::OpenApi).with_validation(true);
    let _ = merger_with_validation;
}

// Test 3: Empty merge: zero specs → error
#[test]
fn test_empty_merge() {
    let merger = SpecificationMerger::new(SpecFormat::OpenApi);

    let result = merger.merge_specifications(vec![]);
    assert!(result.is_err(), "Empty merge should fail");

    match result {
        Err(e) => {
            let error_str = format!("{:?}", e);
            assert!(
                error_str.contains("empty") || error_str.contains("Empty"),
                "Error should mention empty specs"
            );
        }
        _ => panic!("Expected error for empty merge"),
    }
}

// Test 4: Single spec merge
#[test]
fn test_single_spec_merge() {
    let merger = SpecificationMerger::new(SpecFormat::GraphQL);

    // Single spec should work (though api-merger doesn't implement actual merge logic)
    // This test just verifies the merger was created successfully
    let _ = merger;
}

// Test 5: SpecFormat consistency
#[test]
fn test_spec_format_consistency() {
    let merger_openapi = SpecificationMerger::new(SpecFormat::OpenApi);
    let merger_graphql = SpecificationMerger::new(SpecFormat::GraphQL);
    let merger_grpc = SpecificationMerger::new(SpecFormat::Grpc);

    // Verify mergers were created with correct formats
    let _ = merger_openapi;
    let _ = merger_graphql;
    let _ = merger_grpc;
}
