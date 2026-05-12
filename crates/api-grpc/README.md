# api-grpc

gRPC Protocol Buffers generator implementing universal API traits for zero-cost abstraction.

## Usage

```rust
use api_grpc::{
    GrpcGenerator, GrpcSchema, ProtoDefinition, ProtoMessage, ProtoField,
    ProtoService, ProtoMethod, MethodStreaming,
};
use api_core::ApiSpecGenerator;

struct UserService;

impl GrpcSchema for UserService {
    fn proto_definition() -> ProtoDefinition {
        ProtoDefinition {
            package: "user".to_string(),
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
                    ],
                ),
            ],
            services: vec![],
            imports: vec![],
        }
    }
}
```

## Features

- `GrpcGenerator` - gRPC .proto generator implementing `ApiSpecGenerator` trait
- `GrpcSchema` - Trait for types providing Protocol Buffer definitions
- `ProtoDefinition` - Complete Protocol Buffer definition
- `ProtoMessage`, `ProtoField` - Message type definitions
- `ProtoService`, `ProtoMethod` - Service and RPC method definitions
- `MethodStreaming` - Streaming configuration for methods (unary, server, client, bidi)

## License

MIT
