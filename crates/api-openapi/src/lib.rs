//! OpenAPI Specification Generator — Single Source of Truth
//!
//! This crate is the **SSOT** for all API data models and the OpenAPI specification
//! for the deploy-baba portfolio service. It provides:
//!
//! * [`models`] — every public/admin request/response struct with `ToSchema` + `ApiModel`.
//! * [`registry`] — compile-time checked `ALL_MODELS` list (drives coverage tests).
//! * [`apidoc`] — `PublicApiDoc`, `AdminApiDoc`, and `full_spec()` for router init.
//! * [`filter`] — `public_view()` strips admin paths + unreferenced schemas.
//! * The original `OpenApiGenerator` / `OpenApiSchema` / `merge_openapi_specs` are
//!   preserved for use by `api-merger` and examples.
//!
//! `services/ui` imports types from this crate instead of defining them locally.
//! The `utoipa-axum` router in `services/ui` calls `api_openapi::apidoc::full_spec()`
//! to seed the router, then `.split_for_parts()` to obtain the merged spec with paths.
//!
//! # Example
//!
//! ```rust
//! use api_openapi::{OpenApiGenerator, OpenApiSchema};
//! use api_core::ApiSpecGenerator;
//! use utoipa::OpenApi;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(OpenApi)]
//! #[openapi(
//!     paths(get_users),
//!     components(schemas(User))
//! )]
//! struct ApiDoc;
//!
//! #[derive(Serialize, Deserialize, utoipa::ToSchema)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! #[utoipa::path(
//!     get,
//!     path = "/users",
//!     responses(
//!         (status = 200, description = "List users", body = [User])
//!     )
//! )]
//! async fn get_users() -> Vec<User> {
//!     vec![]
//! }
//!
//! impl OpenApiSchema for ApiDoc {
//!     fn api_schema() -> utoipa::openapi::OpenApi {
//!         ApiDoc::openapi()
//!     }
//! }
//!
//! let spec = OpenApiGenerator::<ApiDoc>::generate_spec(ApiDoc::api_schema()).unwrap();
//! let json = serde_json::to_string_pretty(&spec).unwrap();
//! ```

pub mod apidoc;
pub mod filter;
pub mod models;
pub mod registry;

use api_core::{ApiSpecGenerator, SpecError, SpecValidationError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

/// OpenAPI specification generator implementing universal API traits
///
/// This generator wraps utoipa's OpenAPI types and provides validation and merging
/// capabilities through the universal trait interface.
pub struct OpenApiGenerator<T> {
    _phantom: PhantomData<T>,
}

impl<T> OpenApiGenerator<T> {
    /// Create a new OpenAPI generator for the given schema type
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for OpenApiGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can provide OpenAPI schema information
///
/// Implementors should return a fully constructed OpenAPI specification
/// that includes all paths, components, and metadata.
pub trait OpenApiSchema {
    /// Get the OpenAPI specification for this API
    fn api_schema() -> utoipa::openapi::OpenApi;
}

/// OpenAPI specification wrapper that includes validation metadata
///
/// This struct wraps the utoipa OpenAPI type and adds standardized metadata
/// for tracking generation and validation status.
#[derive(Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    /// The OpenAPI specification
    pub openapi: utoipa::openapi::OpenApi,
    /// Generation metadata
    pub metadata: OpenApiMetadata,
}

impl std::fmt::Debug for OpenApiSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version_str = "3.0+";
        f.debug_struct("OpenApiSpec")
            .field("metadata", &self.metadata)
            .field(
                "openapi_info",
                &format!("OpenAPI v{} - {}", version_str, &self.openapi.info.title),
            )
            .finish()
    }
}

/// Metadata about OpenAPI specification generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiMetadata {
    /// Generator version and name
    pub generator: String,
    /// Generation timestamp
    pub generated_at: String,
    /// Specification validation status
    pub validated: bool,
    /// Number of paths in the specification
    pub path_count: usize,
    /// Number of schema components
    pub schema_count: usize,
}

impl<T> ApiSpecGenerator for OpenApiGenerator<T>
where
    T: OpenApiSchema,
{
    type Schema = utoipa::openapi::OpenApi;
    type Output = OpenApiSpec;

    fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError> {
        // Create metadata from the OpenAPI specification
        let metadata = OpenApiMetadata {
            generator: format!("api-openapi v{}", env!("CARGO_PKG_VERSION")),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| format!("{}s since epoch", d.as_secs()))
                .unwrap_or_else(|_| "unknown".to_string()),
            validated: false,
            path_count: schema.paths.paths.len(),
            schema_count: schema
                .components
                .as_ref()
                .map(|c| c.schemas.len())
                .unwrap_or(0),
        };

        let spec = OpenApiSpec {
            openapi: schema,
            metadata,
        };

        Ok(spec)
    }

    fn validate_spec(spec: &Self::Output) -> Result<(), Vec<SpecValidationError>> {
        let mut errors = Vec::new();

        // Validate required fields
        if spec.openapi.info.title.is_empty() {
            errors.push(SpecValidationError::new(
                "info.title",
                "OpenAPI specification must have a title",
            ));
        }

        if spec.openapi.info.version.is_empty() {
            errors.push(SpecValidationError::new(
                "info.version",
                "OpenAPI specification must have a version",
            ));
        }

        // Validate paths
        if spec.openapi.paths.paths.is_empty() {
            errors.push(SpecValidationError::new(
                "paths",
                "OpenAPI specification should have at least one path",
            ));
        }

        // Validate path structure
        for (path_name, path_item) in &spec.openapi.paths.paths {
            if !path_name.starts_with('/') {
                errors.push(SpecValidationError::new(
                    "paths",
                    format!("Path '{}' must start with '/'", path_name),
                ));
            }

            // Check if path has at least one operation
            let has_operations = !path_item.operations.is_empty();

            if !has_operations {
                errors.push(SpecValidationError::new(
                    "paths",
                    format!("Path '{}' has no operations defined", path_name),
                ));
            }
        }

        // Validate component schemas if present
        if let Some(components) = &spec.openapi.components {
            for schema_name in components.schemas.keys() {
                if schema_name.is_empty() {
                    errors.push(SpecValidationError::new(
                        "components.schemas",
                        "Schema component names cannot be empty",
                    ));
                }
            }
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

        // Convert to utoipa specs for merging
        let utoipa_specs: Vec<utoipa::openapi::OpenApi> =
            specs.into_iter().map(|s| s.openapi).collect();

        merge_openapi_specs(utoipa_specs)
    }
}

/// OpenAPI-specific specification errors
#[derive(Error, Debug)]
pub enum OpenApiSpecError {
    /// OpenAPI serialization error
    #[error("OpenAPI serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// OpenAPI validation failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    Validation(Vec<SpecValidationError>),

    /// OpenAPI generation error
    #[error("OpenAPI generation error: {0}")]
    Generation(String),
}

impl From<OpenApiSpecError> for SpecError {
    fn from(error: OpenApiSpecError) -> Self {
        match error {
            OpenApiSpecError::Serialization(e) => SpecError::GenerationFailed(e.to_string()),
            OpenApiSpecError::Validation(errors) => {
                let error_msg = format_validation_errors(&errors);
                SpecError::InvalidSchema(error_msg)
            }
            OpenApiSpecError::Generation(msg) => SpecError::GenerationFailed(msg),
        }
    }
}

impl From<SpecError> for OpenApiSpecError {
    fn from(error: SpecError) -> Self {
        match error {
            SpecError::InvalidSchema(msg) | SpecError::GenerationFailed(msg) => {
                OpenApiSpecError::Generation(msg)
            }
            _ => OpenApiSpecError::Generation(error.to_string()),
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

/// Convenience function to generate OpenAPI JSON from a schema
///
/// # Example
///
/// ```rust,ignore
/// let json = generate_openapi_json::<MyApiDoc>()?;
/// println!("{}", json);
/// ```
pub fn generate_openapi_json<T: OpenApiSchema>() -> Result<String, OpenApiSpecError> {
    let schema = T::api_schema();
    let spec = OpenApiGenerator::<T>::generate_spec(schema)?;

    OpenApiGenerator::<T>::validate_spec(&spec).map_err(OpenApiSpecError::Validation)?;

    let json = serde_json::to_string_pretty(&spec.openapi)?;
    Ok(json)
}

/// Convenience function to generate OpenAPI specification from multiple schemas
///
/// This function merges multiple OpenAPI specifications into a single unified spec.
/// It will fail if duplicate paths or schemas are found across specs.
pub fn merge_openapi_specs(specs: Vec<utoipa::openapi::OpenApi>) -> Result<OpenApiSpec, SpecError> {
    if specs.is_empty() {
        return Err(SpecError::MergeError(
            "Cannot merge empty specification list".to_string(),
        ));
    }

    let mut merged = specs[0].clone();

    // Merge paths from all specifications
    for spec in specs.iter().skip(1) {
        for (path, path_item) in &spec.paths.paths {
            if merged.paths.paths.contains_key(path) {
                return Err(SpecError::MergeError(format!(
                    "Duplicate path found during merge: {}",
                    path
                )));
            }
            merged.paths.paths.insert(path.clone(), path_item.clone());
        }

        // Merge components if present
        if let (Some(merged_components), Some(spec_components)) =
            (&mut merged.components, &spec.components)
        {
            // Merge schemas
            for (name, schema) in &spec_components.schemas {
                if merged_components.schemas.contains_key(name) {
                    return Err(SpecError::MergeError(format!(
                        "Duplicate schema found during merge: {}",
                        name
                    )));
                }
                merged_components
                    .schemas
                    .insert(name.clone(), schema.clone());
            }
        } else if spec.components.is_some() {
            merged.components = spec.components.clone();
        }
    }

    // Generate the merged specification
    let metadata = OpenApiMetadata {
        generator: format!("api-openapi v{}", env!("CARGO_PKG_VERSION")),
        generated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| format!("{}s since epoch", d.as_secs()))
            .unwrap_or_else(|_| "unknown".to_string()),
        validated: false,
        path_count: merged.paths.paths.len(),
        schema_count: merged
            .components
            .as_ref()
            .map(|c| c.schemas.len())
            .unwrap_or(0),
    };

    Ok(OpenApiSpec {
        openapi: merged,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use utoipa::{OpenApi, ToSchema};

    #[derive(OpenApi)]
    #[openapi(
        paths(get_users, create_user),
        components(schemas(User, CreateUserRequest))
    )]
    struct TestApiDoc;

    #[derive(Serialize, Deserialize, ToSchema)]
    struct User {
        id: u32,
        name: String,
        email: String,
    }

    #[derive(Serialize, Deserialize, ToSchema)]
    struct CreateUserRequest {
        name: String,
        email: String,
    }

    #[utoipa::path(
        get,
        path = "/users",
        responses(
            (status = 200, description = "List users successfully", body = [User])
        )
    )]
    #[allow(dead_code)]
    async fn get_users() -> Vec<User> {
        vec![]
    }

    #[utoipa::path(
        post,
        path = "/users",
        request_body = CreateUserRequest,
        responses(
            (status = 201, description = "User created successfully", body = User),
            (status = 400, description = "Invalid user data")
        )
    )]
    #[allow(dead_code)]
    async fn create_user() -> User {
        User {
            id: 1,
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    impl OpenApiSchema for TestApiDoc {
        fn api_schema() -> utoipa::openapi::OpenApi {
            TestApiDoc::openapi()
        }
    }

    #[test]
    fn test_openapi_generation() {
        let schema = TestApiDoc::api_schema();
        let spec = OpenApiGenerator::<TestApiDoc>::generate_spec(schema).unwrap();

        assert!(!spec.openapi.paths.paths.is_empty());
        assert!(spec.openapi.components.is_some());

        let components = spec.openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("User"));
        assert!(components.schemas.contains_key("CreateUserRequest"));
    }

    #[test]
    fn test_openapi_validation() {
        let schema = TestApiDoc::api_schema();
        let spec = OpenApiGenerator::<TestApiDoc>::generate_spec(schema).unwrap();

        let validation_result = OpenApiGenerator::<TestApiDoc>::validate_spec(&spec);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_openapi_validation_failure() {
        let mut invalid_schema = TestApiDoc::api_schema();
        invalid_schema.info.title = "".to_string();
        invalid_schema.info.version = "".to_string();

        let spec = OpenApiGenerator::<TestApiDoc>::generate_spec(invalid_schema).unwrap();
        let validation_result = OpenApiGenerator::<TestApiDoc>::validate_spec(&spec);

        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
        assert_eq!(errors.len(), 2);

        let error_fields: Vec<&str> = errors.iter().map(|e| e.path.as_str()).collect();
        assert!(error_fields.contains(&"info.title"));
        assert!(error_fields.contains(&"info.version"));
    }

    #[test]
    fn test_generate_openapi_json() {
        let json = generate_openapi_json::<TestApiDoc>().unwrap();

        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"/users\""));
        assert!(json.contains("\"User\""));
        assert!(json.contains("\"CreateUserRequest\""));
    }

    #[test]
    fn test_openapi_metadata() {
        let schema = TestApiDoc::api_schema();
        let spec = OpenApiGenerator::<TestApiDoc>::generate_spec(schema).unwrap();

        assert!(spec.metadata.generator.starts_with("api-openapi"));
        assert!(!spec.metadata.generated_at.is_empty());
        assert_eq!(spec.metadata.path_count, spec.openapi.paths.paths.len());

        let components = spec.openapi.components.as_ref().unwrap();
        assert_eq!(spec.metadata.schema_count, components.schemas.len());
    }

    #[test]
    fn test_openapi_spec_merging() {
        #[derive(OpenApi)]
        #[openapi(paths(get_posts), components(schemas(Post)))]
        struct PostsApiDoc;

        #[derive(Serialize, Deserialize, ToSchema)]
        struct Post {
            id: u32,
            title: String,
            content: String,
        }

        #[utoipa::path(
            get,
            path = "/posts",
            responses(
                (status = 200, description = "List posts", body = [Post])
            )
        )]
        #[allow(dead_code)]
        async fn get_posts() -> Vec<Post> {
            vec![]
        }

        let user_spec = TestApiDoc::openapi();
        let post_spec = PostsApiDoc::openapi();

        let merged = merge_openapi_specs(vec![user_spec, post_spec]).unwrap();

        assert!(merged.openapi.paths.paths.contains_key("/users"));
        assert!(merged.openapi.paths.paths.contains_key("/posts"));

        let components = merged.openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("User"));
        assert!(components.schemas.contains_key("CreateUserRequest"));
        assert!(components.schemas.contains_key("Post"));
    }

    #[test]
    fn test_openapi_merge_conflict() {
        let spec1 = TestApiDoc::openapi();
        let spec2 = TestApiDoc::openapi();

        let result = merge_openapi_specs(vec![spec1, spec2]);
        assert!(result.is_err());

        if let Err(SpecError::MergeError(msg)) = result {
            assert!(msg.contains("Duplicate path"));
        }
    }
}
