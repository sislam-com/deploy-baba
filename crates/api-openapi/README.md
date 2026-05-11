# api-openapi

OpenAPI 3.0 specification generator implementing universal API traits for zero-cost abstraction.

## Usage

```rust
use api_openapi::{OpenApiGenerator, OpenApiSchema};
use api_core::ApiSpecGenerator;
use utoipa::OpenApi;
use serde::{Deserialize, Serialize};

#[derive(OpenApi)]
#[openapi(
    paths(get_users),
    components(schemas(User))
)]
struct ApiDoc;

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
struct User {
    id: u32,
    name: String,
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List users", body = [User])
    )
)]
async fn get_users() -> Vec<User> {
    vec![]
}

impl OpenApiSchema for ApiDoc {
    fn api_schema() -> utoipa::openapi::OpenApi {
        // Generate OpenAPI spec
        todo!()
    }
}
```

## Features

- `OpenApiGenerator` - OpenAPI 3.0 generator implementing `ApiSpecGenerator` trait
- `OpenApiSchema` - Trait for OpenAPI schema generation
- `models` - All public/admin request/response structs with `ToSchema` + `ApiModel`
- `registry` - Compile-time checked `ALL_MODELS` list for coverage tests
- `apidoc` - `PublicApiDoc`, `AdminApiDoc`, and `full_spec()` for router init
- `filter` - `public_view()` strips admin paths + unreferenced schemas

## License

MIT
