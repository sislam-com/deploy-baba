# api-graphql

GraphQL schema generator implementing universal API traits for zero-cost abstraction.

## Usage

```rust
use api_graphql::{GraphQLGenerator, GraphQLSchema, GraphQLSchemaDefinition};
use api_core::ApiSpecGenerator;

struct MySchema;

impl GraphQLSchema for MySchema {
    fn schema_definition() -> GraphQLSchemaDefinition {
        GraphQLSchemaDefinition {
            sdl: r#"
            type Query {
                users: [User!]!
            }

            type User {
                id: ID!
                name: String!
            }
            "#.to_string(),
        }
    }
}

let schema_def = MySchema::schema_definition();
let spec = GraphQLGenerator::<MySchema>::generate_spec(schema_def).unwrap();
```

## Features

- `GraphQLGenerator` - GraphQL SDL generator implementing `ApiSpecGenerator` trait
- `GraphQLSchema` - Trait for types providing schema information
- `GraphQLSchemaDefinition` - Schema definition wrapper with SDL
- `GraphQLSpec` - Output specification wrapper with metadata
- Support for GraphQL query, mutation, and subscription types

## License

MIT
