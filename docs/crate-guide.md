# Crate-by-Crate API Guide

This guide provides a technical overview of each crate in the deploy-baba project, organized by architectural layer. Each crate exports a specific set of public types and traits designed for composition through Rust's zero-cost abstractions.

## Configuration Layer

### config-core
**Universal Configuration Parsing Traits**

Zero-cost abstraction layer providing format-agnostic configuration parsing, validation, and merging interfaces.

**Key Types:**
- `ConfigParser<T>` — Trait for parsing strings into typed configuration objects
- `ConfigValidator<T>` — Trait for validating configuration objects
- `ConfigMerger<T>` — Trait for merging multiple configurations
- `EnvironmentInterpolator<T>` — Trait for environment variable substitution
- `ConfigSource` — Enum indicating where configuration originates (File, Env, Remote)
- `ValidationError` — Field-specific validation failure with message
- `ConfigError` — Unified error type for IO, parsing, and validation failures

**Usage Pattern:**
```rust
use config_core::ConfigParser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AppConfig { port: u16 }

struct MyParser;

impl ConfigParser<AppConfig> for MyParser {
    type Error = config_core::ConfigError;

    fn parse(input: &str) -> Result<AppConfig, Self::Error> {
        // Parse implementation
        todo!()
    }

    fn validate(config: &AppConfig) -> Result<(), Vec<config_core::ValidationError>> {
        if config.port == 0 {
            return Err(vec![config_core::ValidationError::new("port", "Must be non-zero")]);
        }
        Ok(())
    }
}
```

**Common Patterns:**
- Implement both `parse()` and `validate()` for format-specific parsing
- Return `ValidationError` vectors for accumulating multiple field errors
- Use monomorphization to specialize implementations at compile time

---

### config-toml
**TOML Configuration Parser Implementation**

Format-specific implementation of universal configuration traits using `toml` crate for deserialization.

**Key Types:**
- `TomlParser<T>` — Zero-cost parser struct (uses `PhantomData`)
- `TomlValidatable` — Trait for custom validation in TOML types
- `TomlConfigError` — TOML-specific error wrapping `toml::de::Error`
- `load_toml_config()` — Convenience function for file I/O + parsing + validation
- `save_toml_config()` — Convenience function for serialization + file write

**Usage Pattern:**
```rust
use config_toml::{TomlParser, TomlValidatable};
use config_core::ConfigParser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config { port: u16 }

impl TomlValidatable for Config {
    fn validate_toml(&self) -> Result<(), Vec<config_core::ValidationError>> {
        if self.port == 0 {
            return Err(vec![config_core::ValidationError::new("port", "Non-zero")]);
        }
        Ok(())
    }
}

let config: Config = TomlParser::parse_and_validate(toml_string)?;
let loaded: Config = config_toml::load_toml_config("config.toml")?;
```

**Common Patterns:**
- Blanket implementations of `TomlValidatable` for primitives (String, i32, bool, etc.)
- `parse_and_validate()` combines parsing and validation in single operation
- File operations abstract away I/O error handling

---

### config-json
**JSON Configuration Parser Implementation**

Format-specific implementation for JSON parsing via `serde_json`.

**Key Types:**
- `JsonParser<T>` — Zero-cost parser struct
- `JsonValidatable` — Trait for custom validation in JSON types
- `JsonConfigError` — JSON-specific error type
- `load_json_config()` / `save_json_config()` — File I/O convenience functions

**Usage Pattern:**
```rust
use config_json::JsonParser;
use config_core::ConfigParser;

let config: MyConfig = JsonParser::parse(json_string)?;
let loaded: MyConfig = config_json::load_json_config("config.json")?;
```

---

### config-yaml
**YAML Configuration Parser Implementation**

Format-specific implementation for YAML parsing via `serde_yaml`.

**Key Types:**
- `YamlParser<T>` — Zero-cost parser struct
- `YamlValidatable` — Trait for custom validation in YAML types
- `YamlConfigError` — YAML-specific error type
- `load_yaml_config()` / `save_yaml_config()` — File I/O convenience functions

**Usage Pattern:**
```rust
use config_yaml::YamlParser;
use config_core::ConfigParser;

let config: MyConfig = YamlParser::parse(yaml_string)?;
```

---

## API Specification Layer

### api-core
**Universal API Specification Generation Traits**

Format-agnostic traits for API specification generation, validation, and merging across OpenAPI, GraphQL, and gRPC.

**Key Types:**
- `ApiSpecGenerator` — Universal trait with `generate_spec()`, `validate_spec()`, `merge_specs()`
- `SpecFormat` — Enum: OpenApi, GraphQL, Grpc, AsyncApi, JsonSchema
- `SpecValidationError` — Error with path, message, and optional error code
- `SpecMetadata` — Standardized metadata (title, version, contact, license, servers)
- `ContactInfo`, `LicenseInfo`, `ServerInfo` — Documentation support types
- `SpecVersioning` — Trait for version compatibility checking

**Usage Pattern:**
```rust
use api_core::{ApiSpecGenerator, SpecError};

struct MyGenerator;

impl ApiSpecGenerator for MyGenerator {
    type Schema = MySchemaType;
    type Output = String;

    fn generate_spec(schema: Self::Schema) -> Result<Self::Output, SpecError> {
        // Generate JSON/YAML/proto from schema
        Ok("generated_spec".to_string())
    }

    fn validate_spec(spec: &Self::Output) -> Result<(), Vec<api_core::SpecValidationError>> {
        // Validate required fields
        Ok(())
    }
}

let output = MyGenerator::generate_and_validate(schema)?;
```

**Common Patterns:**
- Implement `generate_spec()` for format-specific generation
- Return `SpecValidationError` vectors with full paths (e.g., "paths./users.post")
- Use associated types `Schema` and `Output` for flexibility

---

### api-openapi
**OpenAPI 3.0 Specification Generator**

Production implementation generating OpenAPI specifications using utoipa ecosystem.

**Key Types:**
- `OpenApiGenerator<T>` — Generator implementing universal `ApiSpecGenerator` trait
- `OpenApiSchema` — Trait for types providing utoipa OpenAPI specifications
- `OpenApiSpec` — Output wrapper with metadata about generation
- `OpenApiMetadata` — Tracks generator version, path/schema counts, validation status

**Usage Pattern:**
```rust
use api_openapi::{OpenApiGenerator, OpenApiSchema};
use api_core::ApiSpecGenerator;
use utoipa::OpenApi;
use serde::{Deserialize, Serialize};

#[derive(OpenApi)]
#[openapi(paths(get_users), components(schemas(User)))]
struct ApiDoc;

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
struct User { id: u32, name: String }

#[utoipa::path(get, path = "/users", responses((status = 200, body = [User])))]
async fn get_users() -> Vec<User> { vec![] }

impl OpenApiSchema for ApiDoc {
    fn api_schema() -> utoipa::openapi::OpenApi {
        ApiDoc::openapi()
    }
}

let spec = OpenApiGenerator::<ApiDoc>::generate_and_validate(ApiDoc::api_schema())?;
let json = serde_json::to_string_pretty(&spec.openapi)?;
```

**Common Patterns:**
- Paths must start with `/` and include at least one HTTP method
- Validation checks title, version, path structure, and schema names
- `merge_openapi_specs()` combines multiple specs, failing on duplicate paths

---

### api-graphql
**GraphQL Schema Generator**

Implementation for GraphQL Schema Definition Language (SDL) specification generation.

**Key Types:**
- `GraphQLGenerator<T>` — Generator for GraphQL SDL
- `GraphQLSchema` — Trait for types providing schema definitions
- `GraphQLSpec` — Output wrapper with SDL content and metadata
- `GraphQLSchemaDefinition` — Raw SDL string container
- `GraphQLMetadata` — Tracks type/query/mutation/subscription counts

**Usage Pattern:**
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

let spec = GraphQLGenerator::<MySchema>::generate_and_validate(
    MySchema::schema_definition()
)?;
```

---

### api-grpc
**gRPC Protocol Buffers Generator**

Implementation for Protocol Buffer service definitions and messages.

**Key Types:**
- `GrpcGenerator<T>` — Generator for .proto files
- `GrpcSchema` — Trait for types providing proto definitions
- `GrpcSpec` — Output wrapper with proto content and metadata
- `ProtoDefinition` — Complete proto file structure
- `ProtoMessage` / `ProtoField` — Message type definitions
- `ProtoService` / `ProtoMethod` — RPC service definitions
- `MethodStreaming` — Unary/ClientStream/ServerStream/BidiStream

**Usage Pattern:**
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

let spec = GrpcGenerator::<UserService>::generate_and_validate(
    UserService::proto_definition()
)?;
```

---

## Infrastructure Layer

### api-merger
**Universal API Specification Merging System**

Format-agnostic merging system combining specifications across OpenAPI, GraphQL, and gRPC with conflict resolution.

**Key Types:**
- `SpecificationMerger` — Main merger with configurable strategies
- `UnifiedApiSpec` — Enum wrapping any supported format (OpenApi, GraphQL, Grpc)
- `MergedApiSpec` — Result with spec + metadata
- `ConflictResolutionStrategy` — FailOnConflict, FirstWins, LastWins, Merge
- `MergeConflict` — Details about conflicts encountered
- `ConflictType` — DuplicateType, DuplicatePath, IncompatibleType, etc.

**Usage Pattern:**
```rust
use api_merger::{SpecificationMerger, ConflictResolutionStrategy};
use api_core::SpecFormat;

let merger = SpecificationMerger::new(SpecFormat::OpenApi)
    .with_conflict_resolution(ConflictResolutionStrategy::FirstWins)
    .with_validation(true);

let merged = merger.merge_specifications(vec![spec1, spec2])?;
println!("Merged {} specs with {} conflicts",
    merged.metadata.source_count,
    merged.metadata.conflicts.len());
```

**Common Patterns:**
- All specs must be the same format before merging
- `FailOnConflict` raises errors; other strategies record and continue
- Metadata includes resolution strategy, merge timestamp, validation status

---

### infra-types
**Cloud-Agnostic Infrastructure Configuration Types**

Root configuration types for deployment stacks with database and observability support.

**Key Types (Stack Layer):**
- `Stack` — Root configuration combining all infrastructure components
- `ProjectConfig` — Project metadata (name, version, region)
- `DeployConfig` — Deployment mode and settings (function name, memory, timeout)
- `Environment` — Dev, Staging, Prod enum
- `Provider` — Aws, Gcp, Azure, Local
- `DeployMode` — Lambda or EcsFargateSpot

**Key Types (Database):**
- `SqliteConfig` — SQLite database path and WAL mode
- `S3BackupConfig` — Optional backup bucket and schedule

**Key Types (Observability):**
- `ObservabilityConfig` — Logging, metrics, and alerting settings
- `LogLevel` — Debug, Info, Warn, Error
- `MetricsConfig` — Cloudwatch namespace, retention
- `AlertConfig` — Alert thresholds and SNS topics

**Key Types (Network):**
- `NetworkConfig` — VPC, security groups, subnets
- `SecurityGroup` — Ingress/egress rules
- `IngressRule` / `EgressRule` — CIDR blocks and port ranges

**Key Types (AWS):**
- `AwsConfig` — AWS-specific settings (profile, region, account ID)

**Usage Pattern:**
```rust
use infra_types::{Stack, ProjectConfig, DeployConfig, SqliteConfig};

let stack = Stack {
    project: ProjectConfig::new("my-app", "0.1.0", "us-east-1"),
    deploy: DeployConfig {
        mode: "lambda".to_string(),
        function_name: "my-func".to_string(),
        runtime: "provided.al2023".to_string(),
        architecture: "arm64".to_string(),
        memory_mb: 256,
        timeout_seconds: 30,
    },
    database: SqliteConfig::with_path("/mnt/db/app.db"),
    observability: Default::default(),
    aws: Default::default(),
};

println!("Identifier: {}", stack.identifier()); // "my-app-us-east-1"
```

**Common Patterns:**
- Deserialize from TOML for production configuration
- All types implement `Serialize`/`Deserialize` via serde
- Builder patterns available via `.with_*()` methods
- Database only supports SQLite (no direct ORM)

---

## LLM Layer

### llm-core
**Vendor-Agnostic LLM Provider Traits and Agent Loop**

Defines the contract for LLM providers, message types, tool dispatch, and grounding. Any provider adapter (Anthropic, OpenAI) implements `LlmProvider` and plugs in without changing calling code. See [ADR-015](../plans/adr/ADR-015-llm-provider-abstraction-and-grounding-contract.md).

**Key Types:**
- `LlmProvider` — async trait: `provider_id()`, `generate()`, `generate_with_tools()`
- `EmbeddingProvider` — async trait for text embedding
- `ChatMessage` / `MessageContent` / `MessageRole` — conversation message types
- `GenerationConfig` — temperature, max tokens, model selection
- `LlmRequest` / `LlmResponse` — request/response envelopes
- `ToolCall` / `ToolDef` / `ToolResult` — tool-use types for agentic dispatch
- `ToolExecutor` — async trait for executing tool calls
- `GroundingContract` / `RefusalPolicy` — prompt-assembly grounding rules
- `LlmError` — structured error type (via `thiserror`)

**Key Functions:**
- `run_agent_loop()` — executes a multi-turn tool-dispatch loop: sends a request, receives tool calls, executes them, feeds results back, repeats until the model produces a final text response. Returns `AgentResult`.
- `assemble_grounded_prompt()` — wraps retrieved context in citation tags following the grounding contract.

**Usage Pattern:**
```rust
use llm_core::{LlmProvider, LlmRequest, ChatMessage, MessageRole, GenerationConfig};

async fn ask(provider: &impl LlmProvider, question: &str) -> Result<String, llm_core::LlmError> {
    let request = LlmRequest {
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: question.into(),
        }],
        config: GenerationConfig::default(),
        tools: vec![],
    };
    let response = provider.generate(request).await?;
    Ok(response.content)
}
```

---

### llm-anthropic
**Anthropic Claude Adapter**

Implements `LlmProvider` for the Anthropic Messages API. Supports text generation, tool_use, and multi-turn conversations.

**Key Types:**
- `AnthropicProvider` — implements `LlmProvider`, holds API key and model config

**Usage Pattern:**
```rust
use llm_anthropic::AnthropicProvider;
use llm_core::LlmProvider;

let provider = AnthropicProvider::new("your-api-key", "claude-sonnet-4-20250514");
let response = provider.generate(request).await?;
```

See the per-crate README at `crates/llm-anthropic/README.md`.

---

### llm-openai
**OpenAI Adapter (WIP)**

Implements `LlmProvider` for the OpenAI Chat Completions API. Currently a work-in-progress — tracked as W-LLM.4.15.

See the per-crate README at `crates/llm-openai/README.md`.

---

## RAG Layer

### rag-core
**Vendor-Agnostic Retrieval Traits and Chunkers**

Defines the trait contract for retrieval-augmented generation: embedding, retrieval, prompt assembly, and document chunking. See [ADR-016](../plans/adr/ADR-016-rag-architecture.md).

**Key Types:**
- `Retriever` — async trait: `retrieve(query, top_k)` → `Vec<RankedChunk>`
- `Embedder` — async trait for text embedding
- `PromptAssembler` — trait for wrapping chunks in citation tags for the LLM
- `DefaultPromptAssembler` — built-in implementation of `PromptAssembler`
- `HybridRetriever` — combines multiple retrieval strategies
- `PortfolioDataProvider` — trait for loading portfolio-specific data
- `Chunk` / `RankedChunk` — document chunk types with relevance scores
- `CitationRef` — source attribution for grounded generation
- `PromptBundle` — assembled prompt with citations ready for LLM
- `SourceKind` — enum identifying the corpus source (Rust, HCL, Plans, Cache, OpenAPI, Portfolio, Challenges)
- `RagError` — structured error type

**Usage Pattern:**
```rust
use rag_core::{Retriever, RankedChunk};

async fn search(retriever: &impl Retriever, query: &str) -> Vec<RankedChunk> {
    retriever.retrieve(query, 5).await.unwrap_or_default()
}
```

---

### rag-sqlite
**SQLite FTS5 Retrieval Backend**

Implements `Retriever` using SQLite's FTS5 full-text search engine for BM25-ranked retrieval across 7 indexed corpora.

**Key Types:**
- `RagStore` — main struct: manages the SQLite FTS5 index, implements `Retriever`

**Usage Pattern:**
```rust
use rag_sqlite::RagStore;
use rag_core::Retriever;

let store = RagStore::open("deploy-baba.db")?;
let results = store.retrieve("how does auth work", 5).await?;
```

Hybrid retrieval: combines FTS5 BM25 ranking with keyword matching for improved relevance.

---

### portfolio-rag-mcp
**MCP Server for RAG Integration**

A standalone binary that wraps `rag-sqlite` as a Model Context Protocol (MCP) server. This allows Claude Code to query the portfolio's indexed content directly during development sessions.

---

## Architectural Insights

### Zero-Cost Abstractions
All parser implementations (config-toml, config-json, config-yaml) and generators (api-openapi, api-graphql, api-grpc) use:
- Generic `PhantomData<T>` to avoid runtime overhead
- Monomorphization at compile time
- Trait implementations specialized per type
- No vtable or dynamic dispatch

### Error Handling Strategy
- **Parsing errors** are format-specific (TomlConfigError, JsonConfigError)
- **Validation errors** are accumulated in vectors to report all failures
- **ConfigParseError** distinguishes between parse-time and validation-time failures
- All errors implement `thiserror::Error` for ergonomic `.map_err()`

### Validation Philosophy
- Validation is separate from parsing
- Validators return vectors to accumulate multiple errors
- Field-specific error messages aid debugging
- `is_valid()` convenience method available via trait

### Configuration Composition
Stack is built from nested configurations:
- ProjectConfig → metadata
- DeployConfig → compute settings
- SqliteConfig → persistence
- ObservabilityConfig → monitoring
- AwsConfig → cloud provider settings

Each sub-config is independently serializable/deserializable, enabling flexible composition.
