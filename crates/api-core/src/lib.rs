//! Universal API Specification Generation Traits
//!
//! This crate provides universal traits for generating API specifications in any format.
//! It enables zero-cost abstractions where different generators can be swapped without
//! changing downstream code.
//!
//! # Core Concepts
//!
//! The main abstraction is the [`ApiSpecGenerator`] trait, which defines a standardized
//! interface for converting API schemas into specifications, validating them, and merging
//! multiple specifications together.
//!
//! # Supported Formats
//!
//! - **OpenAPI**: RESTful HTTP API specifications
//! - **GraphQL**: Query language schema definitions
//! - **gRPC**: Protocol buffer service definitions
//! - **AsyncAPI**: Event-driven asynchronous API specifications
//! - **JSON Schema**: Generic JSON schema specifications
//!
//! # Example
//!
//! ```rust
//! use api_core::{ApiSpecGenerator, SpecError, SpecFormat, SpecValidationError};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct ApiSchema {
//!     title: String,
//!     version: String,
//!     paths: Vec<String>,
//! }
//!
//! struct OpenApiGenerator;
//!
//! impl ApiSpecGenerator for OpenApiGenerator {
//!     type Schema = ApiSchema;
//!     type Output = String;
//!
//!     fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError> {
//!         // Implementation would generate OpenAPI JSON
//!         Ok(format!("{{\"title\":\"{}\",\"version\":\"{}\"}}", schema.title, schema.version))
//!     }
//!
//!     fn validate_spec(spec: &Self::Output) -> Result<(), Vec<SpecValidationError>> {
//!         // Implementation would validate OpenAPI spec
//!         Ok(())
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Universal trait for generating API specifications
///
/// Implementors of this trait provide standardized generation, validation, and merging
/// operations for API specifications. The trait uses associated types to enable
/// format-specific behavior without runtime overhead.
///
/// # Type Parameters
///
/// - `Schema`: The input specification schema (implementation-specific)
/// - `Output`: The output specification format (implementation-specific)
#[must_use]
pub trait ApiSpecGenerator {
    /// The input schema type for this generator
    type Schema;
    /// The output specification type for this generator
    type Output;

    /// Generate an API specification from a schema
    ///
    /// # Errors
    ///
    /// Returns `SpecError` if the schema is invalid or generation fails.
    fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError>;

    /// Validate a generated specification
    ///
    /// # Errors
    ///
    /// Returns a vector of validation errors if the specification is invalid.
    /// An empty vector indicates successful validation.
    fn validate_spec(spec: &Self::Output) -> Result<(), Vec<SpecValidationError>>;

    /// Merge multiple specifications into one
    ///
    /// Default implementation only accepts single specifications. Implementations
    /// should override this for format-specific merge logic.
    ///
    /// # Errors
    ///
    /// Returns `SpecError::MergeError` if the merge cannot be performed.
    fn merge_specs(specs: Vec<Self::Output>) -> Result<Self::Output, SpecError> {
        if specs.is_empty() {
            return Err(SpecError::MergeError(
                "Cannot merge empty specification list".to_string(),
            ));
        }
        if specs.len() == 1 {
            return Ok(specs.into_iter().next().unwrap());
        }
        Err(SpecError::MergeError(
            "Merging not implemented for this generator".to_string(),
        ))
    }

    /// Generate and validate a specification in one operation
    ///
    /// This is a convenience method that combines generation and validation.
    ///
    /// # Errors
    ///
    /// Returns `SpecGenerationError` if either generation or validation fails.
    fn generate_and_validate(schema: Self::Schema) -> Result<Self::Output, SpecGenerationError> {
        let spec = Self::generate_spec(schema).map_err(SpecGenerationError::Generation)?;
        Self::validate_spec(&spec).map_err(SpecGenerationError::Validation)?;
        Ok(spec)
    }
}

/// Trait for specifications that can be converted to different formats
///
/// Implementors provide format conversion capabilities, enabling specifications
/// to be transformed between compatible formats.
pub trait SpecFormatConverter<T> {
    /// Convert specification to the target format
    ///
    /// # Errors
    ///
    /// Returns `SpecError` if the conversion is not supported or fails.
    fn convert_to_format(&self, format: SpecFormat) -> Result<T, SpecError>;
}

/// Trait for specifications that support versioning
///
/// Implementors provide version management capabilities, including compatibility
/// checking and version upgrades.
pub trait SpecVersioning {
    /// Get the specification version
    fn version(&self) -> &str;

    /// Check if this specification is compatible with another version
    fn is_compatible_with(&self, other_version: &str) -> bool;

    /// Upgrade specification to a newer version
    ///
    /// # Errors
    ///
    /// Returns `SpecError::VersionError` if the upgrade cannot be performed.
    fn upgrade_to(&mut self, target_version: &str) -> Result<(), SpecError>;
}

/// Supported API specification formats
///
/// This enumeration covers the major API specification standards used in
/// modern API development.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SpecFormat {
    /// OpenAPI 3.0+ specification format
    OpenApi,
    /// GraphQL schema definition language
    GraphQL,
    /// Protocol Buffers (gRPC) service definition
    Grpc,
    /// AsyncAPI specification for event-driven APIs
    AsyncApi,
    /// JSON Schema validation schema
    JsonSchema,
}

impl fmt::Display for SpecFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecFormat::OpenApi => write!(f, "OpenAPI"),
            SpecFormat::GraphQL => write!(f, "GraphQL"),
            SpecFormat::Grpc => write!(f, "gRPC"),
            SpecFormat::AsyncApi => write!(f, "AsyncAPI"),
            SpecFormat::JsonSchema => write!(f, "JSON Schema"),
        }
    }
}

/// Specification validation error with context information
///
/// Validation errors include the path to the invalid element, a human-readable
/// message, and an optional error code for programmatic handling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecValidationError {
    /// The path or field that failed validation (e.g., "paths./users.post.responses")
    pub path: String,
    /// Human-readable error message describing the validation failure
    pub message: String,
    /// Optional error code for programmatic error handling (e.g., "MISSING_FIELD")
    pub code: Option<String>,
}

impl SpecValidationError {
    /// Create a new validation error
    ///
    /// # Example
    ///
    /// ```rust
    /// use api_core::SpecValidationError;
    ///
    /// let error = SpecValidationError::new("info.title", "Title is required");
    /// assert_eq!(error.path, "info.title");
    /// assert_eq!(error.message, "Title is required");
    /// assert_eq!(error.code, None);
    /// ```
    pub fn new<P: Into<String>, M: Into<String>>(path: P, message: M) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            code: None,
        }
    }

    /// Create a new validation error with an error code
    ///
    /// # Example
    ///
    /// ```rust
    /// use api_core::SpecValidationError;
    ///
    /// let error = SpecValidationError::with_code(
    ///     "components.schemas.User",
    ///     "Type mismatch",
    ///     "TYPE_MISMATCH"
    /// );
    /// assert_eq!(error.code, Some("TYPE_MISMATCH".to_string()));
    /// ```
    pub fn with_code<P: Into<String>, M: Into<String>, C: Into<String>>(
        path: P,
        message: M,
        code: C,
    ) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            code: Some(code.into()),
        }
    }
}

impl fmt::Display for SpecValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.code {
            Some(code) => write!(
                f,
                "Validation error at '{}' [{}]: {}",
                self.path, code, self.message
            ),
            None => write!(f, "Validation error at '{}': {}", self.path, self.message),
        }
    }
}

/// Combined error type for specification generation and validation
///
/// This error type distinguishes between generation-phase errors and
/// validation-phase errors, allowing for targeted error handling.
#[derive(Error, Debug)]
pub enum SpecGenerationError {
    /// Error occurred during the generation phase
    #[error("Generation error: {0}")]
    Generation(SpecError),

    /// Error occurred during the validation phase
    #[error("Validation errors: {}", format_validation_errors(.0))]
    Validation(Vec<SpecValidationError>),
}

/// Core specification-related errors
///
/// This error type covers all specification-related failures including
/// invalid schemas, generation failures, merge errors, and serialization issues.
#[derive(Error, Debug)]
pub enum SpecError {
    /// The provided schema is invalid or incomplete
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    /// Generation of the specification failed
    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    /// Merge operation failed
    #[error("Merge error: {0}")]
    MergeError(String),

    /// The requested format is not supported
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Version-related error
    #[error("Version error: {0}")]
    VersionError(String),

    /// Serialization or deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Helper function to format multiple validation errors
fn format_validation_errors(errors: &[SpecValidationError]) -> String {
    errors
        .iter()
        .map(|e| format!("{}", e))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Metadata for API specifications
///
/// This structure provides standardized metadata that should be present
/// in all API specifications, including title, version, contact, and license information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    /// Specification title
    pub title: String,
    /// Specification version (semantic versioning recommended)
    pub version: String,
    /// Optional description of the API
    pub description: Option<String>,
    /// Contact information for the API owner/maintainer
    pub contact: Option<ContactInfo>,
    /// License information for the API
    pub license: Option<LicenseInfo>,
    /// List of server/deployment URLs
    pub servers: Vec<ServerInfo>,
}

/// Contact information for API documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// Contact person or organization name
    pub name: Option<String>,
    /// Contact email address
    pub email: Option<String>,
    /// Contact URL (e.g., support website)
    pub url: Option<String>,
}

/// License information for the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    /// License name (e.g., "MIT", "Apache-2.0")
    pub name: String,
    /// URL to the license document
    pub url: Option<String>,
}

/// Server information for API deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server URL or base path
    pub url: String,
    /// Optional description of the server
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestSchema {
        title: String,
        version: String,
    }

    struct MockGenerator;

    impl ApiSpecGenerator for MockGenerator {
        type Schema = TestSchema;
        type Output = String;

        fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError> {
            if schema.title.is_empty() {
                return Err(SpecError::InvalidSchema(
                    "Title cannot be empty".to_string(),
                ));
            }
            Ok(format!(
                "{{\"title\":\"{}\",\"version\":\"{}\"}}",
                schema.title, schema.version
            ))
        }

        fn validate_spec(spec: &Self::Output) -> Result<(), Vec<SpecValidationError>> {
            if !spec.contains("title") {
                return Err(vec![SpecValidationError::new(
                    "root",
                    "Missing title field",
                )]);
            }
            Ok(())
        }
    }

    #[test]
    fn test_spec_generation_success() {
        let schema = TestSchema {
            title: "Test API".to_string(),
            version: "1.0.0".to_string(),
        };
        let result = MockGenerator::generate_and_validate(schema);
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(spec.contains("Test API"));
    }

    #[test]
    fn test_spec_generation_invalid_schema() {
        let schema = TestSchema {
            title: "".to_string(),
            version: "1.0.0".to_string(),
        };
        let result = MockGenerator::generate_and_validate(schema);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpecGenerationError::Generation(SpecError::InvalidSchema(_)) => {}
            _ => panic!("Expected invalid schema error"),
        }
    }

    #[test]
    fn test_validation_error_display() {
        let error = SpecValidationError::with_code(
            "paths./users",
            "Missing required field",
            "MISSING_FIELD",
        );
        assert!(error.to_string().contains("paths./users"));
        assert!(error.to_string().contains("MISSING_FIELD"));
    }

    #[test]
    fn test_spec_format_display() {
        assert_eq!(SpecFormat::OpenApi.to_string(), "OpenAPI");
        assert_eq!(SpecFormat::GraphQL.to_string(), "GraphQL");
        assert_eq!(SpecFormat::Grpc.to_string(), "gRPC");
        assert_eq!(SpecFormat::AsyncApi.to_string(), "AsyncAPI");
        assert_eq!(SpecFormat::JsonSchema.to_string(), "JSON Schema");
    }

    #[test]
    fn test_spec_format_equality() {
        assert_eq!(SpecFormat::OpenApi, SpecFormat::OpenApi);
        assert_ne!(SpecFormat::OpenApi, SpecFormat::GraphQL);
    }

    #[test]
    fn test_validation_error_creation() {
        let error = SpecValidationError::new("field", "error message");
        assert_eq!(error.path, "field");
        assert_eq!(error.message, "error message");
        assert_eq!(error.code, None);
    }

    #[test]
    fn test_validation_error_with_code() {
        let error = SpecValidationError::with_code("field", "error", "CODE");
        assert_eq!(error.code, Some("CODE".to_string()));
    }

    #[test]
    fn test_merge_empty_specs() {
        let result: Result<String, SpecError> = MockGenerator::merge_specs(vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpecError::MergeError(msg) => assert!(msg.contains("empty")),
            _ => panic!("Expected MergeError"),
        }
    }

    #[test]
    fn test_merge_single_spec() {
        let spec = "test".to_string();
        let result = MockGenerator::merge_specs(vec![spec.clone()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), spec);
    }

    #[test]
    fn test_merge_multiple_specs_not_implemented() {
        let result = MockGenerator::merge_specs(vec!["a".to_string(), "b".to_string()]);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpecError::MergeError(msg) => assert!(msg.contains("not implemented")),
            _ => panic!("Expected MergeError"),
        }
    }

    #[test]
    fn test_spec_error_variants_display() {
        assert!(SpecError::InvalidSchema("x".to_string())
            .to_string()
            .contains("Invalid schema"));
        assert!(SpecError::GenerationFailed("x".to_string())
            .to_string()
            .contains("Generation failed"));
        assert!(SpecError::MergeError("x".to_string())
            .to_string()
            .contains("Merge error"));
        assert!(SpecError::UnsupportedFormat("x".to_string())
            .to_string()
            .contains("Unsupported format"));
        assert!(SpecError::VersionError("x".to_string())
            .to_string()
            .contains("Version error"));
    }

    #[test]
    fn test_validation_error_display_no_code() {
        let error = SpecValidationError::new("paths./users", "Missing field");
        let display = error.to_string();
        assert!(display.contains("paths./users"));
        assert!(display.contains("Missing field"));
        assert!(!display.contains('['));
    }

    #[test]
    fn test_spec_generation_error_validation_display() {
        let errors = vec![SpecValidationError::with_code("f", "m", "CODE")];
        let err = SpecGenerationError::Validation(errors);
        let display = err.to_string();
        assert!(display.contains("Validation errors"));
    }

    #[test]
    fn test_spec_metadata_construction() {
        let metadata = SpecMetadata {
            title: "My API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("desc".to_string()),
            contact: Some(ContactInfo {
                name: Some("Alice".to_string()),
                email: Some("alice@example.com".to_string()),
                url: None,
            }),
            license: Some(LicenseInfo {
                name: "MIT".to_string(),
                url: Some("https://opensource.org/licenses/MIT".to_string()),
            }),
            servers: vec![ServerInfo {
                url: "https://api.example.com".to_string(),
                description: Some("Production".to_string()),
            }],
        };
        assert_eq!(metadata.title, "My API");
        assert_eq!(metadata.servers.len(), 1);
        assert!(metadata.contact.is_some());
        assert!(metadata.license.is_some());
    }
}
