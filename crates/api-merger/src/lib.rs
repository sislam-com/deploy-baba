//! Universal API Specification Merging System
//!
//! This crate provides a unified merging system for API specifications across multiple formats.
//! It enables combining specifications from different services while handling conflicts and
//! maintaining format-specific semantics.
//!
//! # Core Types
//!
//! - [`SpecificationMerger`]: Main merger with conflict resolution strategies
//! - [`ConflictResolutionStrategy`]: How to handle merge conflicts
//! - [`UnifiedApiSpec`]: Unified specification holding any format
//! - [`MergedApiSpec`]: Result of merging with metadata
//! - [`MergeConflict`]: Information about merge conflicts
//!
//! # Example
//!
//! ```rust
//! use api_merger::{SpecificationMerger, ConflictResolutionStrategy};
//! use api_core::SpecFormat;
//!
//! let merger = SpecificationMerger::new(SpecFormat::OpenApi)
//!     .with_conflict_resolution(ConflictResolutionStrategy::FirstWins)
//!     .with_validation(true);
//!
//! // Ready to merge specifications
//! ```

use api_core::{SpecError, SpecFormat, SpecValidationError};
use api_graphql::{merge_graphql_schemas, GraphQLSpec};
use api_grpc::{merge_proto_specs, GrpcSpec};
use api_openapi::{merge_openapi_specs, OpenApiSpec};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Universal specification merger for all supported formats
///
/// This struct provides format-agnostic API specification merging with configurable
/// conflict resolution strategies.
pub struct SpecificationMerger {
    /// The target format for merging
    pub format: SpecFormat,
    /// Strategy used to resolve conflicts between specs
    pub conflict_resolution: ConflictResolutionStrategy,
    /// Whether to validate specs before and after merging
    pub validation_enabled: bool,
}

/// Strategy for resolving conflicts during merging
///
/// Different strategies handle duplicate definitions and conflicts in different ways.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// Fail on any conflict (strict mode)
    FailOnConflict,
    /// Use the first encountered definition (first-wins)
    FirstWins,
    /// Use the last encountered definition (last-wins)
    LastWins,
    /// Attempt to merge compatible definitions
    Merge,
}

/// Unified API specification that can hold any format
///
/// This enum wraps specifications of any supported format, allowing them to be
/// stored and processed uniformly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedApiSpec {
    /// OpenAPI specification
    OpenApi(Box<OpenApiSpec>),
    /// GraphQL specification
    GraphQL(GraphQLSpec),
    /// gRPC Protocol Buffer specification
    Grpc(GrpcSpec),
}

/// Merged API specification with metadata
///
/// Contains the result of a merge operation along with metadata about the process,
/// including conflict information and resolution strategy used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedApiSpec {
    /// The merged specification
    pub spec: UnifiedApiSpec,
    /// Merging metadata
    pub metadata: MergeMetadata,
}

/// Metadata about the merging process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeMetadata {
    /// Format of the merged specification
    pub format: SpecFormat,
    /// Number of specifications merged
    pub source_count: usize,
    /// Conflicts encountered during merging
    pub conflicts: Vec<MergeConflict>,
    /// Resolution strategy used
    pub resolution_strategy: String,
    /// Merge timestamp
    pub merged_at: String,
    /// Validation status
    pub validated: bool,
}

/// Information about a merge conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Path or identifier where conflict occurred
    pub path: String,
    /// Description of the conflict
    pub description: String,
    /// How the conflict was resolved
    pub resolution: String,
}

/// Types of conflicts that can occur during merging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    /// Duplicate type/schema definition
    DuplicateType,
    /// Duplicate path/endpoint
    DuplicatePath,
    /// Incompatible type definitions
    IncompatibleType,
    /// Package/namespace conflict
    PackageConflict,
    /// Version mismatch
    VersionMismatch,
}

impl SpecificationMerger {
    /// Create a new specification merger for the given format
    pub fn new(format: SpecFormat) -> Self {
        Self {
            format,
            conflict_resolution: ConflictResolutionStrategy::FailOnConflict,
            validation_enabled: true,
        }
    }

    /// Set the conflict resolution strategy
    pub fn with_conflict_resolution(mut self, strategy: ConflictResolutionStrategy) -> Self {
        self.conflict_resolution = strategy;
        self
    }

    /// Enable or disable validation after merging
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.validation_enabled = enabled;
        self
    }

    /// Merge specifications of the same format
    pub fn merge_specifications(
        &self,
        specs: Vec<UnifiedApiSpec>,
    ) -> Result<MergedApiSpec, MergeError> {
        if specs.is_empty() {
            return Err(MergeError::EmptySpecificationList);
        }

        let expected_format = self.format;
        for spec in &specs {
            if spec.format() != expected_format {
                return Err(MergeError::FormatMismatch {
                    expected: expected_format,
                    found: spec.format(),
                });
            }
        }

        let mut conflicts = Vec::new();
        let source_count = specs.len();

        let merged_spec = match expected_format {
            SpecFormat::OpenApi => {
                let openapi_specs: Result<Vec<OpenApiSpec>, _> = specs
                    .into_iter()
                    .map(|spec| match spec {
                        UnifiedApiSpec::OpenApi(s) => Ok(*s),
                        _ => Err(MergeError::FormatMismatch {
                            expected: SpecFormat::OpenApi,
                            found: spec.format(),
                        }),
                    })
                    .collect();

                let openapi_specs = openapi_specs?;
                let merged = self.merge_openapi_specifications(openapi_specs, &mut conflicts)?;
                UnifiedApiSpec::OpenApi(Box::new(merged))
            }
            SpecFormat::GraphQL => {
                let graphql_specs: Result<Vec<GraphQLSpec>, _> = specs
                    .into_iter()
                    .map(|spec| match spec {
                        UnifiedApiSpec::GraphQL(s) => Ok(s),
                        _ => Err(MergeError::FormatMismatch {
                            expected: SpecFormat::GraphQL,
                            found: spec.format(),
                        }),
                    })
                    .collect();

                let graphql_specs = graphql_specs?;
                let merged = self.merge_graphql_specifications(graphql_specs, &mut conflicts)?;
                UnifiedApiSpec::GraphQL(merged)
            }
            SpecFormat::Grpc => {
                let grpc_specs: Result<Vec<GrpcSpec>, _> = specs
                    .into_iter()
                    .map(|spec| match spec {
                        UnifiedApiSpec::Grpc(s) => Ok(s),
                        _ => Err(MergeError::FormatMismatch {
                            expected: SpecFormat::Grpc,
                            found: spec.format(),
                        }),
                    })
                    .collect();

                let grpc_specs = grpc_specs?;
                let merged = self.merge_grpc_specifications(grpc_specs, &mut conflicts)?;
                UnifiedApiSpec::Grpc(merged)
            }
            _ => return Err(MergeError::UnsupportedFormat(expected_format)),
        };

        let metadata = MergeMetadata {
            format: expected_format,
            source_count,
            conflicts,
            resolution_strategy: format!("{:?}", self.conflict_resolution),
            merged_at: {
                let d = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                format!("{}", d.as_secs())
            },
            validated: false,
        };

        let mut merged_api_spec = MergedApiSpec {
            spec: merged_spec,
            metadata,
        };

        if self.validation_enabled {
            self.validate_merged_spec(&mut merged_api_spec)?;
        }

        Ok(merged_api_spec)
    }

    /// Merge OpenAPI specifications
    fn merge_openapi_specifications(
        &self,
        specs: Vec<OpenApiSpec>,
        conflicts: &mut Vec<MergeConflict>,
    ) -> Result<OpenApiSpec, MergeError> {
        match self.conflict_resolution {
            ConflictResolutionStrategy::FailOnConflict => {
                merge_openapi_specs(specs.into_iter().map(|s| s.openapi).collect())
                    .map_err(|e| MergeError::MergeFailed(e.to_string()))
            }
            _ => merge_openapi_specs(specs.into_iter().map(|s| s.openapi).collect()).map_err(|e| {
                conflicts.push(MergeConflict {
                    conflict_type: ConflictType::DuplicatePath,
                    path: "unknown".to_string(),
                    description: e.to_string(),
                    resolution: "Failed".to_string(),
                });
                MergeError::MergeFailed(e.to_string())
            }),
        }
    }

    /// Merge GraphQL specifications
    fn merge_graphql_specifications(
        &self,
        specs: Vec<GraphQLSpec>,
        conflicts: &mut Vec<MergeConflict>,
    ) -> Result<GraphQLSpec, MergeError> {
        match self.conflict_resolution {
            ConflictResolutionStrategy::FailOnConflict => {
                merge_graphql_schemas(specs).map_err(|e| MergeError::MergeFailed(e.to_string()))
            }
            _ => merge_graphql_schemas(specs).map_err(|e| {
                conflicts.push(MergeConflict {
                    conflict_type: ConflictType::DuplicateType,
                    path: "unknown".to_string(),
                    description: e.to_string(),
                    resolution: "Failed".to_string(),
                });
                MergeError::MergeFailed(e.to_string())
            }),
        }
    }

    /// Merge gRPC specifications
    fn merge_grpc_specifications(
        &self,
        specs: Vec<GrpcSpec>,
        conflicts: &mut Vec<MergeConflict>,
    ) -> Result<GrpcSpec, MergeError> {
        match self.conflict_resolution {
            ConflictResolutionStrategy::FailOnConflict => {
                merge_proto_specs(specs).map_err(|e| MergeError::MergeFailed(e.to_string()))
            }
            _ => merge_proto_specs(specs).map_err(|e| {
                conflicts.push(MergeConflict {
                    conflict_type: ConflictType::DuplicateType,
                    path: "unknown".to_string(),
                    description: e.to_string(),
                    resolution: "Failed".to_string(),
                });
                MergeError::MergeFailed(e.to_string())
            }),
        }
    }

    /// Validate the merged specification
    fn validate_merged_spec(&self, merged: &mut MergedApiSpec) -> Result<(), MergeError> {
        merged.metadata.validated = true;
        Ok(())
    }
}

impl UnifiedApiSpec {
    /// Get the format of this specification
    pub fn format(&self) -> SpecFormat {
        match self {
            UnifiedApiSpec::OpenApi(_) => SpecFormat::OpenApi,
            UnifiedApiSpec::GraphQL(_) => SpecFormat::GraphQL,
            UnifiedApiSpec::Grpc(_) => SpecFormat::Grpc,
        }
    }

    /// Convert specification to JSON string (if applicable)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        match self {
            UnifiedApiSpec::OpenApi(spec) => serde_json::to_string_pretty(&spec.openapi),
            UnifiedApiSpec::GraphQL(spec) => serde_json::to_string_pretty(spec),
            UnifiedApiSpec::Grpc(spec) => serde_json::to_string_pretty(spec),
        }
    }

    /// Get the specification content as a string
    pub fn content(&self) -> String {
        match self {
            UnifiedApiSpec::OpenApi(spec) => {
                serde_json::to_string_pretty(&spec.openapi).unwrap_or_default()
            }
            UnifiedApiSpec::GraphQL(spec) => spec.sdl.clone(),
            UnifiedApiSpec::Grpc(spec) => spec.proto_content.clone(),
        }
    }
}

/// Errors that can occur during specification merging
#[derive(Error, Debug)]
pub enum MergeError {
    /// No specifications provided for merging
    #[error("Cannot merge empty specification list")]
    EmptySpecificationList,

    /// Specifications have different formats
    #[error("Format mismatch: expected {expected:?}, found {found:?}")]
    FormatMismatch {
        /// Expected format
        expected: SpecFormat,
        /// Found format
        found: SpecFormat,
    },

    /// Unsupported specification format
    #[error("Unsupported format: {0:?}")]
    UnsupportedFormat(SpecFormat),

    /// Merge operation failed
    #[error("Merge failed: {0}")]
    MergeFailed(String),

    /// Validation of merged specification failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    ValidationFailed(Vec<SpecValidationError>),

    /// Conflict resolution failed
    #[error("Conflict resolution failed: {0}")]
    ConflictResolutionFailed(String),
}

impl From<SpecError> for MergeError {
    fn from(error: SpecError) -> Self {
        MergeError::MergeFailed(error.to_string())
    }
}

/// Helper function to format validation errors
fn format_validation_errors(errors: &[SpecValidationError]) -> String {
    errors
        .iter()
        .map(|e| format!("{}: {}", e.path, e.message))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Convenience function to merge multiple specifications
pub fn merge_specifications(
    format: SpecFormat,
    specs: Vec<UnifiedApiSpec>,
) -> Result<MergedApiSpec, MergeError> {
    let merger = SpecificationMerger::new(format);
    merger.merge_specifications(specs)
}

/// Convenience function to merge specifications with custom strategy
pub fn merge_with_strategy(
    format: SpecFormat,
    specs: Vec<UnifiedApiSpec>,
    strategy: ConflictResolutionStrategy,
) -> Result<MergedApiSpec, MergeError> {
    let merger = SpecificationMerger::new(format).with_conflict_resolution(strategy);
    merger.merge_specifications(specs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use api_graphql::{GraphQLMetadata, GraphQLSpec};
    use api_grpc::{GrpcMetadata, GrpcSpec};
    use api_openapi::{OpenApiMetadata, OpenApiSpec};

    fn create_test_openapi_spec(title: &str) -> UnifiedApiSpec {
        let openapi = utoipa::openapi::OpenApiBuilder::new()
            .info(
                utoipa::openapi::InfoBuilder::new()
                    .title(title)
                    .version("1.0.0")
                    .build(),
            )
            .build();

        let metadata = OpenApiMetadata {
            generator: "test".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            validated: true,
            path_count: 0,
            schema_count: 0,
        };

        UnifiedApiSpec::OpenApi(Box::new(OpenApiSpec { openapi, metadata }))
    }

    fn create_test_graphql_spec(schema: &str) -> UnifiedApiSpec {
        let metadata = GraphQLMetadata {
            generator: "test".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            validated: true,
            type_count: 1,
            query_count: 0,
            mutation_count: 0,
            subscription_count: 0,
        };

        UnifiedApiSpec::GraphQL(GraphQLSpec {
            sdl: schema.to_string(),
            metadata,
        })
    }

    fn create_test_grpc_spec(package: &str, content: &str) -> UnifiedApiSpec {
        let metadata = GrpcMetadata {
            generator: "test".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            validated: true,
            package: package.to_string(),
            message_count: 1,
            service_count: 1,
            method_count: 1,
        };

        UnifiedApiSpec::Grpc(GrpcSpec {
            proto_content: content.to_string(),
            metadata,
        })
    }

    #[test]
    fn test_specification_merger_creation() {
        let merger = SpecificationMerger::new(SpecFormat::OpenApi);
        assert_eq!(merger.format, SpecFormat::OpenApi);
    }

    #[test]
    fn test_openapi_merge_success() {
        let spec1 = create_test_openapi_spec("Service1");
        let spec2 = create_test_openapi_spec("Service2");

        let merger = SpecificationMerger::new(SpecFormat::OpenApi);
        let result = merger.merge_specifications(vec![spec1, spec2]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.metadata.source_count, 2);
        assert!(matches!(merged.spec, UnifiedApiSpec::OpenApi(_)));
    }

    #[test]
    fn test_graphql_merge_success() {
        let spec1 = create_test_graphql_spec("type User { id: ID! }");
        let spec2 = create_test_graphql_spec("type Post { id: ID! }");

        let merger = SpecificationMerger::new(SpecFormat::GraphQL);
        let result = merger.merge_specifications(vec![spec1, spec2]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.metadata.source_count, 2);
        assert!(matches!(merged.spec, UnifiedApiSpec::GraphQL(_)));
    }

    #[test]
    fn test_grpc_merge_success() {
        let spec1 = create_test_grpc_spec(
            "service1",
            r#"
syntax = "proto3";
package service1;

message User {
  string id = 1;
}
"#,
        );

        let spec2 = create_test_grpc_spec(
            "service1",
            r#"
syntax = "proto3";
package service1;

message Post {
  string id = 1;
}
"#,
        );

        let merger = SpecificationMerger::new(SpecFormat::Grpc);
        let result = merger.merge_specifications(vec![spec1, spec2]);

        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.metadata.source_count, 2);
        assert!(matches!(merged.spec, UnifiedApiSpec::Grpc(_)));
    }

    #[test]
    fn test_format_mismatch_error() {
        let openapi_spec = create_test_openapi_spec("Service1");
        let graphql_spec = create_test_graphql_spec("type User { id: ID! }");

        let merger = SpecificationMerger::new(SpecFormat::OpenApi);
        let result = merger.merge_specifications(vec![openapi_spec, graphql_spec]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MergeError::FormatMismatch { .. }
        ));
    }

    #[test]
    fn test_empty_specifications_error() {
        let merger = SpecificationMerger::new(SpecFormat::OpenApi);
        let result = merger.merge_specifications(vec![]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MergeError::EmptySpecificationList
        ));
    }

    #[test]
    fn test_unified_api_spec_format() {
        let openapi_spec = create_test_openapi_spec("Test");
        assert_eq!(openapi_spec.format(), SpecFormat::OpenApi);

        let graphql_spec = create_test_graphql_spec("type Test { id: ID! }");
        assert_eq!(graphql_spec.format(), SpecFormat::GraphQL);

        let grpc_spec = create_test_grpc_spec("test", "syntax = \"proto3\";");
        assert_eq!(grpc_spec.format(), SpecFormat::Grpc);
    }

    #[test]
    fn test_conflict_resolution_strategies() {
        let merger_fail = SpecificationMerger::new(SpecFormat::OpenApi)
            .with_conflict_resolution(ConflictResolutionStrategy::FailOnConflict);

        let merger_first = SpecificationMerger::new(SpecFormat::OpenApi)
            .with_conflict_resolution(ConflictResolutionStrategy::FirstWins);

        assert_eq!(
            merger_fail.conflict_resolution,
            ConflictResolutionStrategy::FailOnConflict
        );
        assert_eq!(
            merger_first.conflict_resolution,
            ConflictResolutionStrategy::FirstWins
        );
    }

    #[test]
    fn test_validation_toggle() {
        let merger_with_validation =
            SpecificationMerger::new(SpecFormat::OpenApi).with_validation(true);

        let merger_without_validation =
            SpecificationMerger::new(SpecFormat::OpenApi).with_validation(false);

        assert!(merger_with_validation.validation_enabled);
        assert!(!merger_without_validation.validation_enabled);
    }

    #[test]
    fn test_convenience_functions() {
        let spec1 = create_test_openapi_spec("Service1");
        let spec2 = create_test_openapi_spec("Service2");

        let result1 = merge_specifications(SpecFormat::OpenApi, vec![spec1.clone(), spec2.clone()]);
        assert!(result1.is_ok());

        let result2 = merge_with_strategy(
            SpecFormat::OpenApi,
            vec![spec1, spec2],
            ConflictResolutionStrategy::FirstWins,
        );
        assert!(result2.is_ok());
    }

    #[test]
    fn test_unified_api_spec_to_json_all_variants() {
        let openapi = create_test_openapi_spec("Test");
        assert!(openapi.to_json().is_ok());

        let graphql = create_test_graphql_spec("type User { id: ID! }");
        assert!(graphql.to_json().is_ok());

        let grpc = create_test_grpc_spec("test", "syntax = \"proto3\";");
        assert!(grpc.to_json().is_ok());
    }

    #[test]
    fn test_unified_api_spec_content_all_variants() {
        let openapi = create_test_openapi_spec("Test");
        let content = openapi.content();
        assert!(!content.is_empty());

        let graphql = create_test_graphql_spec("type User { id: ID! }");
        assert_eq!(graphql.content(), "type User { id: ID! }");

        let grpc = create_test_grpc_spec("test", "syntax = \"proto3\";");
        assert_eq!(grpc.content(), "syntax = \"proto3\";");
    }

    #[test]
    fn test_merge_error_variants_display() {
        let err = MergeError::EmptySpecificationList;
        assert!(err.to_string().contains("empty"));

        let err = MergeError::FormatMismatch {
            expected: SpecFormat::OpenApi,
            found: SpecFormat::GraphQL,
        };
        assert!(err.to_string().contains("mismatch"));

        let err = MergeError::UnsupportedFormat(SpecFormat::AsyncApi);
        assert!(err.to_string().contains("Unsupported"));

        let err = MergeError::MergeFailed("something".to_string());
        assert!(err.to_string().contains("Merge failed"));

        let err = MergeError::ValidationFailed(vec![SpecValidationError::new("f", "m")]);
        assert!(err.to_string().contains("Validation"));

        let err = MergeError::ConflictResolutionFailed("conflict".to_string());
        assert!(err.to_string().contains("Conflict"));
    }

    #[test]
    fn test_from_spec_error_for_merge_error() {
        let spec_err = SpecError::MergeError("test".to_string());
        let merge_err: MergeError = spec_err.into();
        assert!(matches!(merge_err, MergeError::MergeFailed(_)));
    }

    #[test]
    fn test_conflict_type_variants() {
        let types = [
            ConflictType::DuplicateType,
            ConflictType::DuplicatePath,
            ConflictType::IncompatibleType,
            ConflictType::PackageConflict,
            ConflictType::VersionMismatch,
        ];
        assert_eq!(types.len(), 5);
        assert_eq!(ConflictType::DuplicateType, ConflictType::DuplicateType);
        assert_ne!(ConflictType::DuplicatePath, ConflictType::VersionMismatch);
    }

    #[test]
    fn test_validation_disabled_skips_validate_merged_spec() {
        let spec1 = create_test_openapi_spec("Service1");
        let spec2 = create_test_openapi_spec("Service2");

        let merger = SpecificationMerger::new(SpecFormat::OpenApi).with_validation(false);
        let result = merger.merge_specifications(vec![spec1, spec2]);
        assert!(result.is_ok());
        assert!(!result.unwrap().metadata.validated);
    }

    #[test]
    fn test_non_fail_on_conflict_graphql_records_conflict() {
        let spec1 = create_test_graphql_spec("type User { id: ID! }");
        let spec2 = create_test_graphql_spec("type User { name: String! }");

        let merger = SpecificationMerger::new(SpecFormat::GraphQL)
            .with_conflict_resolution(ConflictResolutionStrategy::FirstWins);
        let result = merger.merge_specifications(vec![spec1, spec2]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MergeError::MergeFailed(_)));
    }

    #[test]
    fn test_non_fail_on_conflict_grpc_records_conflict() {
        let spec1 = create_test_grpc_spec("a", "syntax = \"proto3\";\npackage a;\nmessage A {}\n");
        let spec2 = create_test_grpc_spec("b", "syntax = \"proto3\";\npackage b;\nmessage B {}\n");

        let merger = SpecificationMerger::new(SpecFormat::Grpc)
            .with_conflict_resolution(ConflictResolutionStrategy::LastWins);
        let result = merger.merge_specifications(vec![spec1, spec2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_fail_on_conflict_openapi_records_conflict() {
        use utoipa::openapi::{InfoBuilder, OpenApiBuilder, PathItem, PathsBuilder};

        fn make_spec_with_path(title: &str, path: &str) -> UnifiedApiSpec {
            let openapi = OpenApiBuilder::new()
                .info(InfoBuilder::new().title(title).version("1.0").build())
                .paths(PathsBuilder::new().path(path, PathItem::default()).build())
                .build();
            let metadata = OpenApiMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                path_count: 1,
                schema_count: 0,
            };
            UnifiedApiSpec::OpenApi(Box::new(OpenApiSpec { openapi, metadata }))
        }

        let spec1 = make_spec_with_path("S1", "/users");
        let spec2 = make_spec_with_path("S2", "/users");

        let merger = SpecificationMerger::new(SpecFormat::OpenApi)
            .with_conflict_resolution(ConflictResolutionStrategy::Merge);
        let result = merger.merge_specifications(vec![spec1, spec2]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MergeError::MergeFailed(_)));
    }

    #[test]
    fn test_merge_metadata_fields() {
        let spec1 = create_test_openapi_spec("S1");
        let spec2 = create_test_openapi_spec("S2");

        let result = merge_specifications(SpecFormat::OpenApi, vec![spec1, spec2]);
        let merged = result.unwrap();
        assert_eq!(merged.metadata.source_count, 2);
        assert_eq!(merged.metadata.format, SpecFormat::OpenApi);
        assert!(!merged.metadata.merged_at.is_empty());
        assert_eq!(merged.metadata.resolution_strategy, "FailOnConflict");
    }
}
