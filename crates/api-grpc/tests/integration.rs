//! Integration tests for api-grpc
//!
//! Tests proto generation and proto structure.

use api_core::ApiSpecGenerator;
use api_grpc::{
    GrpcGenerator, GrpcSchema, MethodStreaming, ProtoDefinition, ProtoField, ProtoMessage,
    ProtoMethod, ProtoService,
};

#[derive(Debug, Clone)]
struct TestSchema;

impl GrpcSchema for TestSchema {
    fn proto_definition() -> ProtoDefinition {
        ProtoDefinition {
            package: "test".to_string(),
            messages: vec![ProtoMessage::with_fields(
                "TestMessage".to_string(),
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
            )],
            services: vec![ProtoService {
                name: "TestService".to_string(),
                methods: vec![ProtoMethod {
                    name: "GetTest".to_string(),
                    input_type: "TestMessage".to_string(),
                    output_type: "TestMessage".to_string(),
                    streaming: MethodStreaming::Unary,
                }],
            }],
            imports: vec![],
        }
    }
}

// Test 1: Proto generation
#[test]
fn test_proto_generation() {
    let schema = TestSchema::proto_definition();
    let spec = GrpcGenerator::<TestSchema>::generate_spec(schema).expect("Should generate spec");

    assert!(!spec.proto_content.is_empty(), "Proto should not be empty");
    assert!(
        spec.proto_content.contains("package test"),
        "Proto should contain package declaration"
    );
    assert!(
        spec.proto_content.contains("message TestMessage"),
        "Proto should contain TestMessage message"
    );
    assert!(
        spec.proto_content.contains("service TestService"),
        "Proto should contain TestService service"
    );
    assert!(
        spec.proto_content.contains("rpc GetTest"),
        "Proto should contain GetTest RPC"
    );
}

// Test 2: ProtoDefinition field validation
#[test]
fn test_proto_definition_fields() {
    let definition = ProtoDefinition {
        package: "example".to_string(),
        messages: vec![],
        services: vec![],
        imports: vec![],
    };

    assert_eq!(definition.package, "example");
    assert!(definition.messages.is_empty());
    assert!(definition.services.is_empty());
    assert!(definition.imports.is_empty());
}

// Test 3: ProtoField properties
#[test]
fn test_proto_field_properties() {
    let field = ProtoField {
        name: "test_field".to_string(),
        field_type: "string".to_string(),
        number: 5,
        optional: true,
        repeated: false,
    };

    assert_eq!(field.name, "test_field");
    assert_eq!(field.field_type, "string");
    assert_eq!(field.number, 5);
    assert!(field.optional);
    assert!(!field.repeated);
}

// Test 4: ProtoMessage with helper
#[test]
fn test_proto_message_helper() {
    let message = ProtoMessage::with_fields(
        "TestMsg".to_string(),
        vec![
            ProtoField {
                name: "field1".to_string(),
                field_type: "int32".to_string(),
                number: 1,
                optional: false,
                repeated: false,
            },
            ProtoField {
                name: "field2".to_string(),
                field_type: "string".to_string(),
                number: 2,
                optional: false,
                repeated: true,
            },
        ],
    );

    assert_eq!(message.name, "TestMsg");
    assert_eq!(message.fields.len(), 2);
    assert_eq!(message.fields[0].field_type, "int32");
    assert!(message.fields[1].repeated);
}

// Test 5: MethodStreaming variants
#[test]
fn test_method_streaming_variants() {
    let unary = MethodStreaming::Unary;
    let server_streaming = MethodStreaming::ServerStreaming;
    let client_streaming = MethodStreaming::ClientStreaming;

    // These are enum variants
    // Note: MethodStreaming doesn't implement PartialEq, so we can't directly compare
    // Just verify they can be created
    let _ = (unary, server_streaming, client_streaming);
}

// Test 6: ProtoService with multiple methods
#[test]
fn test_proto_service_multiple_methods() {
    let service = ProtoService {
        name: "MultiMethodService".to_string(),
        methods: vec![
            ProtoMethod {
                name: "Method1".to_string(),
                input_type: "Request".to_string(),
                output_type: "Response".to_string(),
                streaming: MethodStreaming::Unary,
            },
            ProtoMethod {
                name: "Method2".to_string(),
                input_type: "Request".to_string(),
                output_type: "Response".to_string(),
                streaming: MethodStreaming::ServerStreaming,
            },
        ],
    };

    assert_eq!(service.name, "MultiMethodService");
    assert_eq!(service.methods.len(), 2);
    // MethodStreaming doesn't implement PartialEq, so we can't directly compare
    // Just verify the methods exist
    assert_eq!(service.methods[0].name, "Method1");
    assert_eq!(service.methods[1].name, "Method2");
}

// Test 7: Complex proto_content with nested messages
#[test]
fn test_complex_proto_content_nested_messages() {
    let definition = ProtoDefinition {
        package: "complex".to_string(),
        messages: vec![
            ProtoMessage::with_fields(
                "Address".to_string(),
                vec![ProtoField {
                    name: "street".to_string(),
                    field_type: "string".to_string(),
                    number: 1,
                    optional: false,
                    repeated: false,
                }],
            ),
            ProtoMessage::with_fields(
                "Person".to_string(),
                vec![
                    ProtoField {
                        name: "name".to_string(),
                        field_type: "string".to_string(),
                        number: 1,
                        optional: false,
                        repeated: false,
                    },
                    ProtoField {
                        name: "address".to_string(),
                        field_type: "Address".to_string(),
                        number: 2,
                        optional: true,
                        repeated: false,
                    },
                ],
            ),
        ],
        services: vec![],
        imports: vec![],
    };

    let spec =
        GrpcGenerator::<TestSchema>::generate_spec(definition).expect("Should generate spec");

    assert!(
        spec.proto_content.contains("message Address"),
        "Should have Address message"
    );
    assert!(
        spec.proto_content.contains("message Person"),
        "Should have Person message"
    );
    assert!(
        spec.proto_content.contains("address"),
        "Person should have address field"
    );
}

// Test 8: Proto with imports
#[test]
fn test_proto_with_imports() {
    let definition = ProtoDefinition {
        package: "with_imports".to_string(),
        messages: vec![],
        services: vec![],
        imports: vec!["google/protobuf/timestamp.proto".to_string()],
    };

    let spec =
        GrpcGenerator::<TestSchema>::generate_spec(definition).expect("Should generate spec");

    assert!(
        spec.proto_content.contains("import"),
        "Should contain import statement"
    );
    assert!(
        spec.proto_content
            .contains("google/protobuf/timestamp.proto"),
        "Should import timestamp"
    );
}
