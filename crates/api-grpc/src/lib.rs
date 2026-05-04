//! gRPC Protocol Buffers Generator
//!
//! This crate provides a gRPC protocol buffers generator that implements the universal
//! API specification traits from `api-core`. It generates .proto files and service
//! definitions that can be used with various gRPC implementations.
//!
//! # Core Types
//!
//! - [`GrpcGenerator`]: The main generator implementing [`ApiSpecGenerator`]
//! - [`GrpcSpec`]: The output specification wrapper with metadata
//! - [`ProtoDefinition`]: Complete Protocol Buffer definition
//! - [`ProtoMessage`], [`ProtoField`]: Message type definitions
//! - [`ProtoService`], [`ProtoMethod`]: Service and RPC method definitions
//! - [`MethodStreaming`]: Streaming configuration for methods
//!
//! # Example
//!
//! ```rust
//! use api_grpc::{
//!     GrpcGenerator, GrpcSchema, ProtoDefinition, ProtoMessage, ProtoField,
//!     ProtoService, ProtoMethod, MethodStreaming,
//! };
//! use api_core::ApiSpecGenerator;
//!
//! struct UserService;
//!
//! impl GrpcSchema for UserService {
//!     fn proto_definition() -> ProtoDefinition {
//!         ProtoDefinition {
//!             package: "user".to_string(),
//!             messages: vec![
//!                 ProtoMessage::with_fields(
//!                     "User".to_string(),
//!                     vec![
//!                         ProtoField {
//!                             name: "id".to_string(),
//!                             field_type: "uint32".to_string(),
//!                             number: 1,
//!                             optional: false,
//!                             repeated: false,
//!                         },
//!                     ],
//!                 ),
//!             ],
//!             services: vec![],
//!             imports: vec![],
//!         }
//!     }
//! }
//! ```

use api_core::{ApiSpecGenerator, SpecError, SpecValidationError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

/// gRPC specification generator implementing universal API traits
///
/// This generator produces Protocol Buffer 3 (.proto) files and provides validation
/// and merging capabilities for gRPC service definitions.
pub struct GrpcGenerator<T> {
    _phantom: PhantomData<T>,
}

impl<T> GrpcGenerator<T> {
    /// Create a new gRPC generator for the given schema type
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for GrpcGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can provide gRPC schema information
pub trait GrpcSchema {
    /// Get the Protocol Buffers definition for this API
    fn proto_definition() -> ProtoDefinition;
}

/// Complete Protocol Buffers definition
///
/// This struct represents a complete .proto file including package declaration,
/// message types, service definitions, and imports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoDefinition {
    /// Package name for the proto file
    pub package: String,
    /// Message type definitions
    pub messages: Vec<ProtoMessage>,
    /// Service definitions
    pub services: Vec<ProtoService>,
    /// Import statements
    pub imports: Vec<String>,
}

/// Protocol Buffers message definition
///
/// Represents a message type in protobuf with its fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoMessage {
    /// Message name
    pub name: String,
    /// Field definitions
    pub fields: Vec<ProtoField>,
}

/// Protocol Buffers field definition
///
/// Represents a single field within a protobuf message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoField {
    /// Field name
    pub name: String,
    /// Field type (e.g., "string", "int32", "User")
    pub field_type: String,
    /// Field number (1-based index for protobuf encoding)
    pub number: u32,
    /// Whether the field is optional (proto3 syntax)
    pub optional: bool,
    /// Whether the field is repeated (array/list)
    pub repeated: bool,
}

/// gRPC service definition
///
/// Represents a service with its RPC methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoService {
    /// Service name
    pub name: String,
    /// RPC method definitions
    pub methods: Vec<ProtoMethod>,
}

/// gRPC method definition
///
/// Represents a single RPC method within a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoMethod {
    /// Method name
    pub name: String,
    /// Input message type
    pub input_type: String,
    /// Output message type
    pub output_type: String,
    /// Streaming configuration
    pub streaming: MethodStreaming,
}

/// Streaming configuration for gRPC methods
///
/// Defines whether and how a gRPC method uses streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MethodStreaming {
    /// Unary call (single request, single response)
    Unary,
    /// Server streaming (single request, stream of responses)
    ServerStreaming,
    /// Client streaming (stream of requests, single response)
    ClientStreaming,
    /// Bidirectional streaming (stream of requests and responses)
    BiDirectional,
}

impl ProtoMessage {
    /// Create a basic message from a type name
    pub fn from_type(type_name: &str) -> Self {
        ProtoMessage {
            name: type_name.to_string(),
            fields: vec![],
        }
    }

    /// Create a message with explicit field definitions
    pub fn with_fields(name: String, fields: Vec<ProtoField>) -> Self {
        ProtoMessage { name, fields }
    }
}

/// gRPC specification wrapper that includes validation metadata
///
/// This struct wraps the generated .proto file content and provides metadata
/// about the specification including counts of messages, services, and methods.
#[derive(Clone, Serialize, Deserialize)]
pub struct GrpcSpec {
    /// The generated .proto file content
    pub proto_content: String,
    /// Generation metadata
    pub metadata: GrpcMetadata,
}

impl std::fmt::Debug for GrpcSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrpcSpec")
            .field("metadata", &self.metadata)
            .field("proto_length", &self.proto_content.len())
            .field("message_count", &self.metadata.message_count)
            .field("service_count", &self.metadata.service_count)
            .finish()
    }
}

/// Metadata about gRPC specification generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcMetadata {
    /// Generator version and name
    pub generator: String,
    /// Generation timestamp
    pub generated_at: String,
    /// Specification validation status
    pub validated: bool,
    /// Package name
    pub package: String,
    /// Number of message types
    pub message_count: usize,
    /// Number of services
    pub service_count: usize,
    /// Number of RPC methods total
    pub method_count: usize,
}

impl<T> ApiSpecGenerator for GrpcGenerator<T>
where
    T: GrpcSchema,
{
    type Schema = ProtoDefinition;
    type Output = GrpcSpec;

    fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError> {
        let proto_content = generate_proto_file(&schema)?;

        let method_count = schema.services.iter().map(|s| s.methods.len()).sum();

        let metadata = GrpcMetadata {
            generator: format!("api-grpc v{}", env!("CARGO_PKG_VERSION")),
            generated_at: {
                let d = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                format!("{}", d.as_secs())
            },
            validated: false,
            package: schema.package.clone(),
            message_count: schema.messages.len(),
            service_count: schema.services.len(),
            method_count,
        };

        let spec = GrpcSpec {
            proto_content,
            metadata,
        };

        Ok(spec)
    }

    fn validate_spec(spec: &Self::Output) -> Result<(), Vec<SpecValidationError>> {
        let mut errors = Vec::new();

        if spec.proto_content.trim().is_empty() {
            errors.push(SpecValidationError::new(
                "proto_content",
                "Protocol buffers content cannot be empty",
            ));
        }

        if !spec.proto_content.starts_with("syntax = \"proto3\";") {
            errors.push(SpecValidationError::new(
                "syntax",
                "Proto file must start with 'syntax = \"proto3\";'",
            ));
        }

        if !spec.proto_content.contains("package ") {
            errors.push(SpecValidationError::new(
                "package",
                "Proto file must contain a package declaration",
            ));
        }

        if let Err(validation_errors) = validate_proto_syntax(&spec.proto_content) {
            errors.extend(validation_errors);
        }

        if let Err(validation_errors) = validate_proto_definitions(&spec.proto_content) {
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

        merge_proto_specs(specs)
    }
}

/// Generate .proto file content from a ProtoDefinition
fn generate_proto_file(definition: &ProtoDefinition) -> Result<String, SpecError> {
    let mut proto = String::new();

    proto.push_str("syntax = \"proto3\";\n\n");
    proto.push_str(&format!("package {};\n\n", definition.package));

    for import in &definition.imports {
        proto.push_str(&format!("import \"{}\";\n", import));
    }
    if !definition.imports.is_empty() {
        proto.push('\n');
    }

    for message in &definition.messages {
        proto.push_str(&generate_message_definition(message)?);
        proto.push('\n');
    }

    for service in &definition.services {
        proto.push_str(&generate_service_definition(service)?);
        proto.push('\n');
    }

    Ok(proto)
}

/// Generate a message definition
fn generate_message_definition(message: &ProtoMessage) -> Result<String, SpecError> {
    let mut msg = format!("message {} {{\n", message.name);

    for field in &message.fields {
        let field_line = generate_field_definition(field)?;
        msg.push_str(&format!("  {}\n", field_line));
    }

    msg.push_str("}\n");
    Ok(msg)
}

/// Generate a field definition
fn generate_field_definition(field: &ProtoField) -> Result<String, SpecError> {
    let mut field_def = String::new();

    if field.repeated {
        field_def.push_str("repeated ");
    }

    field_def.push_str(&field.field_type);
    field_def.push(' ');
    field_def.push_str(&field.name);
    field_def.push_str(&format!(" = {};", field.number));

    Ok(field_def)
}

/// Generate a service definition
fn generate_service_definition(service: &ProtoService) -> Result<String, SpecError> {
    let mut svc = format!("service {} {{\n", service.name);

    for method in &service.methods {
        let method_line = generate_method_definition(method)?;
        svc.push_str(&format!("  {}\n", method_line));
    }

    svc.push_str("}\n");
    Ok(svc)
}

/// Generate a method definition
fn generate_method_definition(method: &ProtoMethod) -> Result<String, SpecError> {
    let (input_stream, output_stream) = match method.streaming {
        MethodStreaming::Unary => ("", ""),
        MethodStreaming::ServerStreaming => ("", "stream "),
        MethodStreaming::ClientStreaming => ("stream ", ""),
        MethodStreaming::BiDirectional => ("stream ", "stream "),
    };

    Ok(format!(
        "rpc {}({}{}) returns ({}{});",
        method.name, input_stream, method.input_type, output_stream, method.output_type
    ))
}

/// Validate basic proto syntax
fn validate_proto_syntax(proto_content: &str) -> Result<(), Vec<SpecValidationError>> {
    let mut errors = Vec::new();

    let mut brace_count = 0;
    for char in proto_content.chars() {
        match char {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            _ => {}
        }
        if brace_count < 0 {
            errors.push(SpecValidationError::new(
                "syntax",
                "Unbalanced closing brace in proto file",
            ));
            break;
        }
    }

    if brace_count > 0 {
        errors.push(SpecValidationError::new(
            "syntax",
            "Unbalanced opening brace in proto file",
        ));
    }

    let lines: Vec<&str> = proto_content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if (trimmed.starts_with("syntax")
            || trimmed.starts_with("package")
            || trimmed.starts_with("import"))
            && !trimmed.ends_with(';')
        {
            errors.push(SpecValidationError::new(
                format!("line_{}", i + 1),
                format!("Missing semicolon at end of statement: {}", trimmed),
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate proto definitions (messages and services)
fn validate_proto_definitions(proto_content: &str) -> Result<(), Vec<SpecValidationError>> {
    let mut errors = Vec::new();

    let mut message_names = std::collections::HashSet::new();
    let mut service_names = std::collections::HashSet::new();

    for line in proto_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("message ") {
            if let Some(name) = extract_definition_name(trimmed, "message") {
                if !message_names.insert(name.clone()) {
                    errors.push(SpecValidationError::new(
                        format!("messages.{}", name),
                        format!("Duplicate message definition: {}", name),
                    ));
                }
            }
        }

        if trimmed.starts_with("service ") {
            if let Some(name) = extract_definition_name(trimmed, "service") {
                if !service_names.insert(name.clone()) {
                    errors.push(SpecValidationError::new(
                        format!("services.{}", name),
                        format!("Duplicate service definition: {}", name),
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

/// Extract definition name from a line
fn extract_definition_name(line: &str, keyword: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 && parts[0] == keyword {
        Some(parts[1].trim_end_matches('{').to_string())
    } else {
        None
    }
}

/// gRPC-specific specification errors
#[derive(Error, Debug)]
pub enum GrpcSpecError {
    /// Protocol buffers generation error
    #[error("Proto generation error: {0}")]
    ProtoGeneration(String),

    /// gRPC validation failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    Validation(Vec<SpecValidationError>),

    /// Proto parsing error
    #[error("Proto parsing error: {0}")]
    ProtoParsing(String),
}

impl From<GrpcSpecError> for SpecError {
    fn from(error: GrpcSpecError) -> Self {
        match error {
            GrpcSpecError::ProtoGeneration(msg) | GrpcSpecError::ProtoParsing(msg) => {
                SpecError::GenerationFailed(msg)
            }
            GrpcSpecError::Validation(errors) => {
                let error_msg = format_validation_errors(&errors);
                SpecError::InvalidSchema(error_msg)
            }
        }
    }
}

impl From<SpecError> for GrpcSpecError {
    fn from(error: SpecError) -> Self {
        match error {
            SpecError::InvalidSchema(msg) | SpecError::GenerationFailed(msg) => {
                GrpcSpecError::ProtoGeneration(msg)
            }
            _ => GrpcSpecError::ProtoGeneration(error.to_string()),
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

/// Convenience function to generate proto file from a schema
pub fn generate_proto_file_content<T: GrpcSchema>() -> Result<String, GrpcSpecError> {
    let proto_def = T::proto_definition();
    let spec = GrpcGenerator::<T>::generate_spec(proto_def)?;

    GrpcGenerator::<T>::validate_spec(&spec).map_err(GrpcSpecError::Validation)?;

    Ok(spec.proto_content)
}

/// Convenience function to merge multiple proto specifications
pub fn merge_proto_specs(specs: Vec<GrpcSpec>) -> Result<GrpcSpec, SpecError> {
    if specs.is_empty() {
        return Err(SpecError::MergeError(
            "Cannot merge empty specification list".to_string(),
        ));
    }

    if specs.len() == 1 {
        return Ok(specs.into_iter().next().unwrap());
    }

    let mut merged_content = String::new();
    let mut all_packages = std::collections::HashSet::new();
    let mut seen_messages = std::collections::HashSet::new();
    let mut seen_services = std::collections::HashSet::new();

    merged_content.push_str("syntax = \"proto3\";\n\n");

    for spec in &specs {
        if let Some(package_line) = spec
            .proto_content
            .lines()
            .find(|line| line.starts_with("package "))
        {
            all_packages.insert(package_line.to_string());
        }
    }

    if all_packages.len() > 1 {
        return Err(SpecError::MergeError(
            "Cannot merge proto files with different packages".to_string(),
        ));
    }

    if let Some(package) = all_packages.iter().next() {
        merged_content.push_str(package);
        merged_content.push_str("\n\n");
    }

    let mut all_imports = std::collections::HashSet::new();
    for spec in &specs {
        for line in spec.proto_content.lines() {
            if line.starts_with("import ") {
                all_imports.insert(line.to_string());
            }
        }
    }

    for import in &all_imports {
        merged_content.push_str(import);
        merged_content.push('\n');
    }
    if !all_imports.is_empty() {
        merged_content.push('\n');
    }

    for spec in &specs {
        let mut current_definition = String::new();
        let mut in_definition = false;

        for line in spec.proto_content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("syntax")
                || trimmed.starts_with("package")
                || trimmed.starts_with("import")
            {
                continue;
            }

            if trimmed.starts_with("message ") || trimmed.starts_with("service ") {
                in_definition = true;
                current_definition = String::new();

                if trimmed.starts_with("message ") {
                    let definition_name =
                        extract_definition_name(trimmed, "message").unwrap_or_default();

                    if seen_messages.contains(&definition_name) {
                        return Err(SpecError::MergeError(format!(
                            "Duplicate message found during merge: {}",
                            definition_name
                        )));
                    }
                    seen_messages.insert(definition_name);
                } else if trimmed.starts_with("service ") {
                    let definition_name =
                        extract_definition_name(trimmed, "service").unwrap_or_default();

                    if seen_services.contains(&definition_name) {
                        return Err(SpecError::MergeError(format!(
                            "Duplicate service found during merge: {}",
                            definition_name
                        )));
                    }
                    seen_services.insert(definition_name);
                }
            }

            if in_definition {
                current_definition.push_str(line);
                current_definition.push('\n');

                if trimmed == "}" {
                    in_definition = false;
                    merged_content.push_str(&current_definition);
                    merged_content.push('\n');
                }
            }
        }
    }

    let total_messages = seen_messages.len();
    let total_services = seen_services.len();
    let total_methods = specs.iter().map(|s| s.metadata.method_count).sum();

    let package = specs
        .first()
        .map(|s| s.metadata.package.clone())
        .unwrap_or_else(|| "merged".to_string());

    let metadata = GrpcMetadata {
        generator: format!("api-grpc v{} (merged)", env!("CARGO_PKG_VERSION")),
        generated_at: {
            let d = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}", d.as_secs())
        },
        validated: false,
        package,
        message_count: total_messages,
        service_count: total_services,
        method_count: total_methods,
    };

    Ok(GrpcSpec {
        proto_content: merged_content,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestGrpcService;

    impl GrpcSchema for TestGrpcService {
        fn proto_definition() -> ProtoDefinition {
            ProtoDefinition {
                package: "test".to_string(),
                messages: vec![
                    ProtoMessage::with_fields(
                        "User".to_string(),
                        vec![
                            ProtoField {
                                name: "id".to_string(),
                                field_type: "uint32".to_string(),
                                number: 1,
                                optional: false,
                                repeated: false,
                            },
                            ProtoField {
                                name: "name".to_string(),
                                field_type: "string".to_string(),
                                number: 2,
                                optional: false,
                                repeated: false,
                            },
                        ],
                    ),
                    ProtoMessage::with_fields(
                        "CreateUserRequest".to_string(),
                        vec![ProtoField {
                            name: "name".to_string(),
                            field_type: "string".to_string(),
                            number: 1,
                            optional: false,
                            repeated: false,
                        }],
                    ),
                ],
                services: vec![ProtoService {
                    name: "UserService".to_string(),
                    methods: vec![
                        ProtoMethod {
                            name: "CreateUser".to_string(),
                            input_type: "CreateUserRequest".to_string(),
                            output_type: "User".to_string(),
                            streaming: MethodStreaming::Unary,
                        },
                        ProtoMethod {
                            name: "GetUsers".to_string(),
                            input_type: "google.protobuf.Empty".to_string(),
                            output_type: "User".to_string(),
                            streaming: MethodStreaming::ServerStreaming,
                        },
                    ],
                }],
                imports: vec!["google/protobuf/empty.proto".to_string()],
            }
        }
    }

    #[test]
    fn test_grpc_generation() {
        let proto_def = TestGrpcService::proto_definition();
        let spec = GrpcGenerator::<TestGrpcService>::generate_spec(proto_def).unwrap();

        assert!(!spec.proto_content.is_empty());
        assert!(spec.proto_content.contains("syntax = \"proto3\";"));
        assert!(spec.proto_content.contains("package test;"));
        assert!(spec.proto_content.contains("message User"));
        assert!(spec.proto_content.contains("service UserService"));
        assert_eq!(spec.metadata.message_count, 2);
        assert_eq!(spec.metadata.service_count, 1);
        assert_eq!(spec.metadata.method_count, 2);
    }

    #[test]
    fn test_grpc_validation() {
        let proto_def = TestGrpcService::proto_definition();
        let spec = GrpcGenerator::<TestGrpcService>::generate_spec(proto_def).unwrap();

        let validation_result = GrpcGenerator::<TestGrpcService>::validate_spec(&spec);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_grpc_validation_failure() {
        let invalid_spec = GrpcSpec {
            proto_content: "invalid proto syntax".to_string(),
            metadata: GrpcMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: false,
                package: "test".to_string(),
                message_count: 0,
                service_count: 0,
                method_count: 0,
            },
        };

        let validation_result = GrpcGenerator::<TestGrpcService>::validate_spec(&invalid_spec);
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_generate_proto_file_content() {
        let proto_content = generate_proto_file_content::<TestGrpcService>().unwrap();

        assert!(proto_content.contains("syntax = \"proto3\";"));
        assert!(proto_content.contains("package test;"));
        assert!(proto_content.contains("import \"google/protobuf/empty.proto\";"));
        assert!(proto_content.contains("message User"));
        assert!(proto_content.contains("message CreateUserRequest"));
        assert!(proto_content.contains("service UserService"));
        assert!(proto_content.contains("rpc CreateUser"));
        assert!(proto_content.contains("rpc GetUsers"));
        assert!(proto_content.contains("stream User"));
    }

    #[test]
    fn test_method_streaming_generation() {
        let unary = ProtoMethod {
            name: "Unary".to_string(),
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            streaming: MethodStreaming::Unary,
        };

        let server_streaming = ProtoMethod {
            name: "ServerStream".to_string(),
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            streaming: MethodStreaming::ServerStreaming,
        };

        assert_eq!(
            generate_method_definition(&unary).unwrap(),
            "rpc Unary(Request) returns (Response);"
        );
        assert_eq!(
            generate_method_definition(&server_streaming).unwrap(),
            "rpc ServerStream(Request) returns (stream Response);"
        );
    }

    #[test]
    fn test_proto_syntax_validation() {
        assert!(validate_proto_syntax("syntax = \"proto3\"; package test;").is_ok());
        assert!(validate_proto_syntax("syntax = \"proto3\"\npackage test;").is_err());
        assert!(validate_proto_syntax("message Test { }").is_ok());
        assert!(validate_proto_syntax("message Test { ").is_err());
    }

    #[test]
    fn test_definition_name_extraction() {
        assert_eq!(
            extract_definition_name("message User {", "message"),
            Some("User".to_string())
        );
        assert_eq!(
            extract_definition_name("service UserService {", "service"),
            Some("UserService".to_string())
        );
    }

    #[test]
    fn test_proto_specs_merging() {
        let spec1 = GrpcSpec {
            proto_content: r#"syntax = "proto3";
package test;

message User {
  uint32 id = 1;
  string name = 2;
}
"#
            .to_string(),
            metadata: GrpcMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                package: "test".to_string(),
                message_count: 1,
                service_count: 0,
                method_count: 0,
            },
        };

        let spec2 = GrpcSpec {
            proto_content: r#"syntax = "proto3";
package test;

message Post {
  uint32 id = 1;
  string title = 2;
}
"#
            .to_string(),
            metadata: GrpcMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                package: "test".to_string(),
                message_count: 1,
                service_count: 0,
                method_count: 0,
            },
        };

        let merged = merge_proto_specs(vec![spec1, spec2]).unwrap();
        assert!(merged.proto_content.contains("message User"));
        assert!(merged.proto_content.contains("message Post"));
        assert_eq!(merged.metadata.message_count, 2);
        assert_eq!(merged.metadata.package, "test");
    }

    #[test]
    fn test_method_streaming_client_and_bidi() {
        let client = ProtoMethod {
            name: "Upload".to_string(),
            input_type: "Chunk".to_string(),
            output_type: "UploadResponse".to_string(),
            streaming: MethodStreaming::ClientStreaming,
        };
        let bidi = ProtoMethod {
            name: "Chat".to_string(),
            input_type: "Message".to_string(),
            output_type: "Message".to_string(),
            streaming: MethodStreaming::BiDirectional,
        };
        assert_eq!(
            generate_method_definition(&client).unwrap(),
            "rpc Upload(stream Chunk) returns (UploadResponse);"
        );
        assert_eq!(
            generate_method_definition(&bidi).unwrap(),
            "rpc Chat(stream Message) returns (stream Message);"
        );
    }

    #[test]
    fn test_grpc_generator_default() {
        let _gen = GrpcGenerator::<TestGrpcService>::default();
    }

    #[test]
    fn test_merge_specs_empty() {
        let result = GrpcGenerator::<TestGrpcService>::merge_specs(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_specs_single() {
        let proto_def = TestGrpcService::proto_definition();
        let spec = GrpcGenerator::<TestGrpcService>::generate_spec(proto_def).unwrap();
        let result = GrpcGenerator::<TestGrpcService>::merge_specs(vec![spec]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_grpc_error_conversions() {
        let validation_err =
            GrpcSpecError::Validation(vec![SpecValidationError::new("field", "bad")]);
        let spec_err: SpecError = validation_err.into();
        assert!(matches!(spec_err, SpecError::InvalidSchema(_)));

        let gen_err = GrpcSpecError::ProtoGeneration("fail".to_string());
        let spec_err: SpecError = gen_err.into();
        assert!(matches!(spec_err, SpecError::GenerationFailed(_)));

        let parsing_err = GrpcSpecError::ProtoParsing("parse fail".to_string());
        let spec_err2: SpecError = parsing_err.into();
        assert!(matches!(spec_err2, SpecError::GenerationFailed(_)));

        let merge_err = SpecError::MergeError("merge".to_string());
        let grpc_err: GrpcSpecError = merge_err.into();
        assert!(matches!(grpc_err, GrpcSpecError::ProtoGeneration(_)));
    }

    #[test]
    fn test_validate_proto_duplicate_service() {
        let proto_content = r#"syntax = "proto3";
package test;

service MyService {
}

service MyService {
}
"#;
        let result = validate_proto_definitions(proto_content);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.path.contains("MyService")));
    }

    #[test]
    fn test_merge_different_packages_error() {
        let spec1 = GrpcSpec {
            proto_content: "syntax = \"proto3\";\npackage a;\nmessage A {\n}\n".to_string(),
            metadata: GrpcMetadata {
                generator: "t".to_string(),
                generated_at: "x".to_string(),
                validated: false,
                package: "a".to_string(),
                message_count: 1,
                service_count: 0,
                method_count: 0,
            },
        };
        let spec2 = GrpcSpec {
            proto_content: "syntax = \"proto3\";\npackage b;\nmessage B {\n}\n".to_string(),
            metadata: GrpcMetadata {
                generator: "t".to_string(),
                generated_at: "x".to_string(),
                validated: false,
                package: "b".to_string(),
                message_count: 1,
                service_count: 0,
                method_count: 0,
            },
        };
        let result = merge_proto_specs(vec![spec1, spec2]);
        assert!(matches!(result, Err(SpecError::MergeError(_))));
    }

    #[test]
    fn test_grpc_spec_debug() {
        let proto_def = TestGrpcService::proto_definition();
        let spec = GrpcGenerator::<TestGrpcService>::generate_spec(proto_def).unwrap();
        let debug_str = format!("{:?}", spec);
        assert!(debug_str.contains("GrpcSpec"));
    }

    #[test]
    fn test_proto_message_from_type() {
        let msg = ProtoMessage::from_type("EmptyMsg");
        assert_eq!(msg.name, "EmptyMsg");
        assert!(msg.fields.is_empty());
    }

    #[test]
    fn test_validate_spec_empty_content() {
        let invalid = GrpcSpec {
            proto_content: "   ".to_string(),
            metadata: GrpcMetadata {
                generator: "t".to_string(),
                generated_at: "x".to_string(),
                validated: false,
                package: "t".to_string(),
                message_count: 0,
                service_count: 0,
                method_count: 0,
            },
        };
        let result = GrpcGenerator::<TestGrpcService>::validate_spec(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_proto_merge_conflict() {
        let spec1 = GrpcSpec {
            proto_content: r#"syntax = "proto3";
package test;

message User {
  uint32 id = 1;
}
"#
            .to_string(),
            metadata: GrpcMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                package: "test".to_string(),
                message_count: 1,
                service_count: 0,
                method_count: 0,
            },
        };

        let spec2 = GrpcSpec {
            proto_content: r#"syntax = "proto3";
package test;

message User {
  string name = 1;
}
"#
            .to_string(),
            metadata: GrpcMetadata {
                generator: "test".to_string(),
                generated_at: "2025-01-01T00:00:00Z".to_string(),
                validated: true,
                package: "test".to_string(),
                message_count: 1,
                service_count: 0,
                method_count: 0,
            },
        };

        let result = merge_proto_specs(vec![spec1, spec2]);
        assert!(result.is_err());

        if let Err(SpecError::MergeError(msg)) = result {
            assert!(msg.contains("Duplicate message"));
        }
    }
}
