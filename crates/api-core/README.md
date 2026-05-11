# api-core

Universal API specification generation traits for zero-cost abstraction over multiple API formats.

## Usage

```rust
use api_core::{ApiSpecGenerator, SpecError, SpecFormat, SpecValidationError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ApiSchema {
    title: String,
    version: String,
    paths: Vec<String>,
}

struct OpenApiGenerator;

impl ApiSpecGenerator for OpenApiGenerator {
    type Schema = ApiSchema;
    type Output = String;

    fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError> {
        // Implementation would generate OpenAPI JSON
        Ok(format!("{{\"title\":\"{}\",\"version\":\"{}\"}}", schema.title, schema.version))
    }

    fn validate_spec(spec: &Self::Output) -> Result<(), Vec<SpecValidationError>> {
        // Implementation would validate OpenAPI spec
        Ok(())
    }
}
```

## Features

- `ApiSpecGenerator` - Universal trait for generating API specifications
- `SpecFormat` - Enumeration of supported specification formats
- `SpecError` - Error type for specification generation
- `SpecValidationError` - Structured validation error for specifications
- Support for OpenAPI, GraphQL, gRPC, AsyncAPI, and JSON Schema

## License

MIT
