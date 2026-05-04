//! GraphQL Schema Generator
//!
//! This crate provides a GraphQL schema generator that implements the universal
//! API specification traits from `api-core`. It enables generation, validation,
//! and merging of GraphQL Schema Definition Language (SDL) specifications.
//!
//! # Core Types
//!
//! - [`GraphQLGenerator`]: The main generator implementing [`ApiSpecGenerator`]
//! - [`GraphQLSpec`]: The output specification wrapper with metadata
//! - [`GraphQLSchema`]: Trait for types providing schema information
//!
//! # Example
//!
//! ```rust
//! use api_graphql::{GraphQLGenerator, GraphQLSchema, GraphQLSchemaDefinition};
//! use api_core::ApiSpecGenerator;
//!
//! struct MySchema;
//!
//! impl GraphQLSchema for MySchema {
//!     fn schema_definition() -> GraphQLSchemaDefinition {
//!         GraphQLSchemaDefinition {
//!             sdl: r#"
//!             type Query {
//!                 users: [User!]!
//!             }
//!
//!             type User {
//!                 id: ID!
//!                 name: String!
//!             }
//!             "#.to_string(),
//!         }
//!     }
//! }
//!
//! let schema_def = MySchema::schema_definition();
//! let spec = GraphQLGenerator::<MySchema>::generate_spec(schema_def).unwrap();
//! ```

use api_core::{ApiSpecGenerator, SpecError, SpecValidationError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

/// GraphQL schema generator implementing universal API traits
///
/// This generator produces GraphQL Schema Definition Language (SDL) specifications
/// and provides validation and merging capabilities.
pub struct GraphQLGenerator<T> {
    _phantom: PhantomData<T>,
}

impl<T> GraphQLGenerator<T> {
    /// Create a new GraphQL generator for the given schema type
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for GraphQLGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// GraphQL schema definition
///
/// This struct holds the raw GraphQL SDL that will be processed by the generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLSchemaDefinition {
    /// The GraphQL Schema Definition Language (SDL) string
    pub sdl: String,
}

/// Trait for types that can provide GraphQL schema information
pub trait GraphQLSchema {
    /// Get the GraphQL schema definition for this API
    fn schema_definition() -> GraphQLSchemaDefinition;
}

/// GraphQL specification wrapper that includes validation metadata
///
/// This struct wraps the SDL string and provides metadata about the schema
/// including type counts and operation information.
#[derive(Clone, Serialize, Deserialize)]
pub struct GraphQLSpec {
    /// The GraphQL Schema Definition Language (SDL) string
    pub sdl: String,
    /// Generation metadata
    pub metadata: GraphQLMetadata,
}

impl std::fmt::Debug for GraphQLSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphQLSpec")
            .field("metadata", &self.metadata)
            .field("sdl_length", &self.sdl.len())
            .field("type_count", &self.sdl.matches("type ").count())
            .finish()
    }
}

/// Metadata about GraphQL schema generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLMetadata {
    /// Generator version and name
    pub generator: String,
    /// Generation timestamp
    pub generated_at: String,
    /// Specification validation status
    pub validated: bool,
    /// Number of types in the schema
    pub type_count: usize,
    /// Number of queries
    pub query_count: usize,
    /// Number of mutations
    pub mutation_count: usize,
    /// Number of subscriptions
    pub subscription_count: usize,
}

impl<T> ApiSpecGenerator for GraphQLGenerator<T>
where
    T: GraphQLSchema,
{
    type Schema = GraphQLSchemaDefinition;
    type Output = GraphQLSpec;

    fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError> {
        let sdl = schema.sdl;

        // Create metadata from the schema
        let type_count = sdl.matches("type ").count();
        let query_count = count_operations(&sdl, "Query");
        let mutation_count = count_operations(&sdl, "Mutation");
        let subscription_count = count_operations(&sdl, "Subscription");

        let metadata = GraphQLMetadata {
            generator: format!("api-graphql v{}", env!("CARGO_PKG_VERSION")),
            generated_at: {
                let d = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                format!("{}", d.as_secs())
            },
            validated: false,
            type_count,
            query_count,
            mutation_count,
            subscription_count,
        };

        let spec = GraphQLSpec { sdl, metadata };

        Ok(spec)
    }

    fn validate_spec(spec: &Self::Output) -> Result<(), Vec<SpecValidationError>> {
        let mut errors = Vec::new();

        // Validate SDL is not empty
        if spec.sdl.trim().is_empty() {
            errors.push(SpecValidationError::new(
                "sdl",
                "GraphQL SDL cannot be empty",
            ));
        }

        // Validate Query type exists (required by GraphQL spec)
        if !spec.sdl.contains("type Query") {
            errors.push(SpecValidationError::new(
                "schema",
                "GraphQL schema must contain a Query type",
            ));
        }

        // Check for basic SDL syntax validity
        if !validate_basic_sdl_syntax(&spec.sdl) {
            errors.push(SpecValidationError::new(
                "sdl",
                "GraphQL SDL contains syntax errors",
            ));
        }

        // Validate type definitions
        if let Err(validation_errors) = validate_graphql_types(&spec.sdl) {
            errors.extend(validation_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn merge_specs(specs: Vec<Self::Output>) -> Result<Self::Output, SpecError> {
        if specs.is_empty() {
            return Err(SpecError::MergeError(
                "Cannot merge empty specification list".to_string(),
            ));
        }

        if specs.len() == 1 {
            return Ok(specs.into_iter().next().unwrap());
        }

        merge_graphql_schemas(specs)
    }
}

/// Helper function to count operations in a specific type
fn count_operations(sdl: &str, type_name: &str) -> usize {
    if let Some(start) = sdl.find(&format!("type {} {{", type_name)) {
        let remaining = &sdl[start..];
        if let Some(end) = remaining.find('}') {
            let type_body = &remaining[..end];
            return type_body
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty()
                        && !trimmed.starts_with("type")
                        && !trimmed.starts_with("}")
                        && trimmed.contains(':')
                })
                .count();
        }
    }
    0
}

/// Basic SDL syntax validation
fn validate_basic_sdl_syntax(sdl: &str) -> bool {
    let mut brace_count = 0;
    for char in sdl.chars() {
        match char {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            _ => {}
        }
        if brace_count < 0 {
            return false;
        }
    }
    brace_count == 0
}

/// Validate GraphQL type definitions
fn validate_graphql_types(sdl: &str) -> Result<(), Vec<SpecValidationError>> {
    let mut errors = Vec::new();

    // Check for duplicate type definitions
    let mut type_names = std::collections::HashSet::new();

    for line in sdl.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("type ")
            || trimmed.starts_with("input ")
            || trimmed.starts_with("enum ")
        {
            if let Some(type_definition) = extract_type_name(trimmed) {
                if !type_names.insert(type_definition.clone()) {
                    errors.push(SpecValidationError::new(
                        format!("types.{}", type_definition),
                        format!("Duplicate type definition: {}", type_definition),
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Extract type name from a type definition line
fn extract_type_name(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// GraphQL-specific specification errors
#[derive(Error, Debug)]
pub enum GraphQLSpecError {
    /// GraphQL schema generation error
    #[error("GraphQL schema error: {0}")]
    Schema(String),

    /// GraphQL validation failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    Validation(Vec<SpecValidationError>),

    /// SDL parsing error
    #[error("SDL parsing error: {0}")]
    SdlParsing(String),
}

impl From<GraphQLSpecError> for SpecError {
    fn from(error: GraphQLSpecError) -> Self {
        match error {
            GraphQLSpecError::Schema(msg) | GraphQLSpecError::SdlParsing(msg) => {
                SpecError::GenerationFailed(msg)
            }
            GraphQLSpecError::Validation(errors) => {
                let error_msg = format_validation_errors(&errors);
                SpecError::InvalidSchema(error_msg)
            }
        }
    }
}

impl From<SpecError> for GraphQLSpecError {
    fn from(error: SpecError) -> Self {
        match error {
            SpecError::InvalidSchema(msg) | SpecError::GenerationFailed(msg) => {
                GraphQLSpecError::Schema(msg)
            }
            _ => GraphQLSpecError::Schema(error.to_string()),
        }
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

/// Convenience function to generate GraphQL SDL from a schema
pub fn generate_graphql_sdl<T: GraphQLSchema>() -> Result<String, GraphQLSpecError> {
    let schema = T::schema_definition();
    let spec = GraphQLGenerator::<T>::generate_spec(schema)?;

    GraphQLGenerator::<T>::validate_spec(&spec).map_err(GraphQLSpecError::Validation)?;

    Ok(spec.sdl)
}

/// Convenience function to merge multiple GraphQL schemas
pub fn merge_graphql_schemas(specs: Vec<GraphQLSpec>) -> Result<GraphQLSpec, SpecError> {
    if specs.is_empty() {
        return Err(SpecError::MergeError(
            "Cannot merge empty specification list".to_string(),
        ));
    }

    if specs.len() == 1 {
        return Ok(specs.into_iter().next().unwrap());
    }

    // For GraphQL, merging is more complex than simple concatenation
    let mut merged_sdl = String::new();
    let mut seen_types = std::collections::HashSet::new();

    // Collect all unique type definitions
    for spec in &specs {
        for line in spec.sdl.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("type ")
                || trimmed.starts_with("input ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("interface ")
            {
                if let Some(type_name) = extract_type_name(trimmed) {
                    if type_name == "Query"
                        || type_name == "Mutation"
                        || type_name == "Subscription"
                    {
                        continue;
                    }

                    if seen_types.contains(&type_name) {
                        return Err(SpecError::MergeError(format!(
                            "Duplicate type found during merge: {}",
                            type_name
                        )));
                    }
                    seen_types.insert(type_name);
                }
            }

            merged_sdl.push_str(line);
            merged_sdl.push('\n');
        }
    }

    // Create merged metadata
    let total_types = seen_types.len();
    let total_queries = specs.iter().map(|s| s.metadata.query_count).sum();
    let total_mutations = specs.iter().map(|s| s.metadata.mutation_count).sum();
    let total_subscriptions = specs.iter().map(|s| s.metadata.subscription_count).sum();

    let metadata = GraphQLMetadata {
        generator: format!("api-graphql v{} (merged)", env!("CARGO_PKG_VERSION")),
        generated_at: {
            let d = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}", d.as_secs())
        },
        validated: false,
        type_count: total_types,
        query_count: total_queries,
        mutation_count: total_mutations,
        subscription_count: total_subscriptions,
    };

    Ok(GraphQLSpec {
        sdl: merged_sdl,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct QueryRoot;

    impl GraphQLSchema for QueryRoot {
        fn schema_definition() -> GraphQLSchemaDefinition {
            GraphQLSchemaDefinition {
                sdl: r#"
type Query {
    users: [User!]!
    user(id: ID!): User
}

type User {
    id: ID!
    name: String!
    email: String!
}
"#
                .to_string(),
            }
        }
    }

    #[test]
    fn test_graphql_generation() {
        let schema = QueryRoot::schema_definition();
        let spec = GraphQLGenerator::<QueryRoot>::generate_spec(schema).unwrap();

        assert!(!spec.sdl.is_empty());
        assert!(spec.sdl.contains("type Query"));
        assert!(spec.sdl.contains("type User"));
        assert!(spec.metadata.type_count > 0);
    }

    #[test]
    fn test_graphql_validation() {
        let schema = QueryRoot::schema_definition();
        let spec = GraphQLGenerator::<QueryRoot>::generate_spec(schema).unwrap();

        let validation_result = GraphQLGenerator::<QueryRoot>::validate_spec(&spec);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_graphql_validation_failure() {
        let invalid_spec = GraphQLSpec {
            sdl: "invalid graphql syntax { }".to_string(),
            metadata: GraphQLMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: false,
                type_count: 0,
                query_count: 0,
                mutation_count: 0,
                subscription_count: 0,
            },
        };

        let validation_result = GraphQLGenerator::<QueryRoot>::validate_spec(&invalid_spec);
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_generate_graphql_sdl() {
        let sdl = generate_graphql_sdl::<QueryRoot>().unwrap();

        assert!(sdl.contains("type Query"));
        assert!(sdl.contains("type User"));
        assert!(sdl.contains("users"));
        assert!(sdl.contains("user"));
    }

    #[test]
    fn test_operation_counting() {
        let sdl = r#"
type Query {
    users: [User!]!
    user(id: ID!): User
}

type User {
    id: ID!
    name: String!
}
"#;

        let query_count = count_operations(sdl, "Query");
        assert_eq!(query_count, 2);
    }

    #[test]
    fn test_basic_sdl_validation() {
        assert!(validate_basic_sdl_syntax("type Query { hello: String }"));
        assert!(!validate_basic_sdl_syntax("type Query { hello: String"));
        assert!(!validate_basic_sdl_syntax("type Query } hello: String {"));
    }

    #[test]
    fn test_type_name_extraction() {
        assert_eq!(extract_type_name("type User {"), Some("User".to_string()));
        assert_eq!(
            extract_type_name("input CreateUserInput {"),
            Some("CreateUserInput".to_string())
        );
        assert_eq!(
            extract_type_name("enum Status {"),
            Some("Status".to_string())
        );
        assert_eq!(extract_type_name("invalid"), None);
    }

    #[test]
    fn test_graphql_schema_merging() {
        let spec1 = GraphQLSpec {
            sdl: "type User { id: ID! name: String! }".to_string(),
            metadata: GraphQLMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                type_count: 1,
                query_count: 0,
                mutation_count: 0,
                subscription_count: 0,
            },
        };

        let spec2 = GraphQLSpec {
            sdl: "type Post { id: ID! title: String! }".to_string(),
            metadata: GraphQLMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                type_count: 1,
                query_count: 0,
                mutation_count: 0,
                subscription_count: 0,
            },
        };

        let merged = merge_graphql_schemas(vec![spec1, spec2]).unwrap();
        assert!(merged.sdl.contains("type User"));
        assert!(merged.sdl.contains("type Post"));
        assert_eq!(merged.metadata.type_count, 2);
    }

    #[test]
    fn test_graphql_merge_conflict() {
        let spec1 = GraphQLSpec {
            sdl: "type User { id: ID! }".to_string(),
            metadata: GraphQLMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                type_count: 1,
                query_count: 0,
                mutation_count: 0,
                subscription_count: 0,
            },
        };

        let spec2 = GraphQLSpec {
            sdl: "type User { name: String! }".to_string(),
            metadata: GraphQLMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                type_count: 1,
                query_count: 0,
                mutation_count: 0,
                subscription_count: 0,
            },
        };

        let result = merge_graphql_schemas(vec![spec1, spec2]);
        assert!(result.is_err());

        if let Err(SpecError::MergeError(msg)) = result {
            assert!(msg.contains("Duplicate type"));
        }
    }

    #[test]
    fn test_graphql_generator_default() {
        let _gen = GraphQLGenerator::<QueryRoot>::default();
    }

    #[test]
    fn test_merge_specs_empty_returns_error() {
        let result = GraphQLGenerator::<QueryRoot>::merge_specs(vec![]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpecError::MergeError(_)));
    }

    #[test]
    fn test_merge_specs_single_returns_ok() {
        let schema = QueryRoot::schema_definition();
        let spec = GraphQLGenerator::<QueryRoot>::generate_spec(schema).unwrap();
        let sdl = spec.sdl.clone();
        let result = GraphQLGenerator::<QueryRoot>::merge_specs(vec![spec]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sdl, sdl);
    }

    #[test]
    fn test_mutation_subscription_counting() {
        let sdl = r#"type Mutation {
    createUser(name: String!): User!
    deleteUser(id: ID!): Boolean!
}
type Subscription {
    userAdded: User!
}"#;
        assert_eq!(count_operations(sdl, "Mutation"), 2);
        assert_eq!(count_operations(sdl, "Subscription"), 1);
        assert_eq!(count_operations(sdl, "Query"), 0);
    }

    #[test]
    fn test_graphql_spec_debug_format() {
        let schema = QueryRoot::schema_definition();
        let spec = GraphQLGenerator::<QueryRoot>::generate_spec(schema).unwrap();
        let debug_str = format!("{:?}", spec);
        assert!(debug_str.contains("GraphQLSpec"));
        assert!(debug_str.contains("sdl_length"));
    }

    #[test]
    fn test_graphql_error_conversions() {
        let err = GraphQLSpecError::Schema("schema error".to_string());
        let spec_err: SpecError = err.into();
        assert!(matches!(spec_err, SpecError::GenerationFailed(_)));

        let err = GraphQLSpecError::SdlParsing("parse error".to_string());
        let spec_err: SpecError = err.into();
        assert!(matches!(spec_err, SpecError::GenerationFailed(_)));

        let err = GraphQLSpecError::Validation(vec![SpecValidationError::new("f", "msg")]);
        let spec_err: SpecError = err.into();
        assert!(matches!(spec_err, SpecError::InvalidSchema(_)));

        let spec_err = SpecError::InvalidSchema("invalid".to_string());
        let gql_err: GraphQLSpecError = spec_err.into();
        assert!(matches!(gql_err, GraphQLSpecError::Schema(_)));

        let spec_err = SpecError::MergeError("merge failed".to_string());
        let gql_err: GraphQLSpecError = spec_err.into();
        assert!(matches!(gql_err, GraphQLSpecError::Schema(_)));
    }

    #[test]
    fn test_validate_graphql_types_duplicate_input() {
        let sdl = r#"type Query { hello: String }
input CreateUserInput { name: String! }
input CreateUserInput { email: String! }"#;
        let result = validate_graphql_types(sdl);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("CreateUserInput")));
    }

    #[test]
    fn test_merge_with_interface_type() {
        let spec1 = GraphQLSpec {
            sdl: "interface Node { id: ID! }\ntype User implements Node { id: ID! }".to_string(),
            metadata: GraphQLMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                type_count: 2,
                query_count: 0,
                mutation_count: 0,
                subscription_count: 0,
            },
        };
        let spec2 = GraphQLSpec {
            sdl: "type Post { id: ID! }".to_string(),
            metadata: GraphQLMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                type_count: 1,
                query_count: 0,
                mutation_count: 0,
                subscription_count: 0,
            },
        };
        let result = merge_graphql_schemas(vec![spec1, spec2]);
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert!(merged.sdl.contains("interface Node"));
    }
}
