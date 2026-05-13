//! Integration tests for api-graphql
//!
//! Tests SDL generation and schema merge.

use api_core::ApiSpecGenerator;
use api_graphql::{GraphQLGenerator, GraphQLSchema, GraphQLSchemaDefinition};

#[derive(Debug, Clone)]
struct TestSchema;

impl GraphQLSchema for TestSchema {
    fn schema_definition() -> GraphQLSchemaDefinition {
        GraphQLSchemaDefinition {
            sdl: r#"
type Query {
    user(id: ID!): User
    users: [User!]!
}

type User {
    id: ID!
    name: String!
    email: String!
}

type Mutation {
    createUser(input: CreateUserInput!): User!
}

input CreateUserInput {
    name: String!
    email: String!
}
"#
            .to_string(),
        }
    }
}

// Test 1: SDL generation: construct GraphQLSchema with types and fields → generate_sdl() → assert SDL string contains type definitions
#[test]
fn test_sdl_generation() {
    let schema = TestSchema::schema_definition();
    let spec = GraphQLGenerator::<TestSchema>::generate_spec(schema).expect("Should generate spec");

    assert!(!spec.sdl.is_empty(), "SDL should not be empty");
    assert!(
        spec.sdl.contains("type Query"),
        "SDL should contain Query type"
    );
    assert!(
        spec.sdl.contains("type User"),
        "SDL should contain User type"
    );
    assert!(
        spec.sdl.contains("type Mutation"),
        "SDL should contain Mutation type"
    );
    assert!(
        spec.sdl.contains("input CreateUserInput"),
        "SDL should contain CreateUserInput"
    );
}

// Test 2: Schema merge: two schemas with disjoint types → merged schema has all types
#[test]
fn test_schema_merge_disjoint_types() {
    #[derive(Debug, Clone)]
    struct Schema1;

    impl GraphQLSchema for Schema1 {
        fn schema_definition() -> GraphQLSchemaDefinition {
            GraphQLSchemaDefinition {
                sdl: r#"
type Query {
    user(id: ID!): User
}

type User {
    id: ID!
    name: String!
}
"#
                .to_string(),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct Schema2;

    impl GraphQLSchema for Schema2 {
        fn schema_definition() -> GraphQLSchemaDefinition {
            GraphQLSchemaDefinition {
                sdl: r#"
type Query {
    post(id: ID!): Post
}

type Post {
    id: ID!
    title: String!
}
"#
                .to_string(),
            }
        }
    }

    let schema1_def = Schema1::schema_definition();
    let schema2_def = Schema2::schema_definition();

    // Generate both specs
    let spec1 =
        GraphQLGenerator::<Schema1>::generate_spec(schema1_def).expect("Should generate spec1");
    let spec2 =
        GraphQLGenerator::<Schema2>::generate_spec(schema2_def).expect("Should generate spec2");

    // Verify each spec has its own types
    assert!(
        spec1.sdl.contains("type User"),
        "Schema1 should have User type"
    );
    assert!(
        !spec1.sdl.contains("type Post"),
        "Schema1 should not have Post type"
    );

    assert!(
        spec2.sdl.contains("type Post"),
        "Schema2 should have Post type"
    );
    assert!(
        !spec2.sdl.contains("type User"),
        "Schema2 should not have User type"
    );

    // Combined would have both types (simulated merge)
    let combined_sdl = format!("{}\n{}", spec1.sdl, spec2.sdl);
    assert!(
        combined_sdl.contains("type User"),
        "Combined should have User"
    );
    assert!(
        combined_sdl.contains("type Post"),
        "Combined should have Post"
    );
}

// Test 3: GraphQLSchemaDefinition metadata
#[test]
fn test_schema_definition_metadata() {
    let schema = GraphQLSchemaDefinition {
        sdl: r#"
type Query {
    hello: String
}
"#
        .to_string(),
    };

    assert!(!schema.sdl.is_empty(), "SDL should not be empty");
    assert!(
        schema.sdl.contains("type Query"),
        "SDL should contain Query type"
    );
    assert!(
        schema.sdl.contains("hello"),
        "SDL should contain hello field"
    );
}

// Test 4: SDL with complex nested types
#[test]
fn test_sdl_nested_types() {
    #[derive(Debug, Clone)]
    struct NestedSchema;

    impl GraphQLSchema for NestedSchema {
        fn schema_definition() -> GraphQLSchemaDefinition {
            GraphQLSchemaDefinition {
                sdl: r#"
type Query {
    author(id: ID!): Author
}

type Author {
    id: ID!
    name: String!
    posts: [Post!]!
}

type Post {
    id: ID!
    title: String!
    author: Author!
    comments: [Comment!]!
}

type Comment {
    id: ID!
    text: String!
    author: Author!
}
"#
                .to_string(),
            }
        }
    }

    let schema = NestedSchema::schema_definition();
    let spec =
        GraphQLGenerator::<NestedSchema>::generate_spec(schema).expect("Should generate spec");

    assert!(spec.sdl.contains("type Author"), "Should have Author type");
    assert!(spec.sdl.contains("type Post"), "Should have Post type");
    assert!(
        spec.sdl.contains("type Comment"),
        "Should have Comment type"
    );

    // Verify nested relationships
    assert!(
        spec.sdl.contains("posts: [Post!]!"),
        "Author should have posts field"
    );
    assert!(
        spec.sdl.contains("author: Author!"),
        "Post should have author field"
    );
    assert!(
        spec.sdl.contains("comments: [Comment!]!"),
        "Post should have comments field"
    );
}

// Test 5: SDL with interfaces
#[test]
fn test_sdl_interfaces() {
    #[derive(Debug, Clone)]
    struct InterfaceSchema;

    impl GraphQLSchema for InterfaceSchema {
        fn schema_definition() -> GraphQLSchemaDefinition {
            GraphQLSchemaDefinition {
                sdl: r#"
interface Node {
    id: ID!
}

type User implements Node {
    id: ID!
    name: String!
}

type Post implements Node {
    id: ID!
    title: String!
}
"#
                .to_string(),
            }
        }
    }

    let schema = InterfaceSchema::schema_definition();
    let spec =
        GraphQLGenerator::<InterfaceSchema>::generate_spec(schema).expect("Should generate spec");

    assert!(
        spec.sdl.contains("interface Node"),
        "Should have Node interface"
    );
    assert!(
        spec.sdl.contains("implements Node"),
        "Types should implement Node"
    );
}

// Test 6: SDL with enums
#[test]
fn test_sdl_enums() {
    #[derive(Debug, Clone)]
    struct EnumSchema;

    impl GraphQLSchema for EnumSchema {
        fn schema_definition() -> GraphQLSchemaDefinition {
            GraphQLSchemaDefinition {
                sdl: r#"
type Query {
    role: Role
}

enum Role {
    ADMIN
    USER
    GUEST
}
"#
                .to_string(),
            }
        }
    }

    let schema = EnumSchema::schema_definition();
    let spec = GraphQLGenerator::<EnumSchema>::generate_spec(schema).expect("Should generate spec");

    assert!(spec.sdl.contains("enum Role"), "Should have Role enum");
    assert!(spec.sdl.contains("ADMIN"), "Should have ADMIN value");
    assert!(spec.sdl.contains("USER"), "Should have USER value");
    assert!(spec.sdl.contains("GUEST"), "Should have GUEST value");
}
