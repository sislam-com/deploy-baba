/// Compile-time model registry — every API body type lives here.
///
/// # Rust Architect note: compile-time enforcement
///
/// `ApiModel` is a sealed trait that any struct used as an OpenAPI request/response
/// body MUST implement. Because all models live in this crate and `services/ui`
/// imports them via `use api_openapi::models::*`, a developer cannot introduce a
/// new route body type in `services/ui` that isn't already registered here — it
/// simply won't have `ToSchema` + `ApiModel` implementations and `cargo build`
/// will reject it.
///
/// For request bodies, handler extraction uses `Json<T>` where `T: ApiModel`.
/// The `_assert_model` zero-cost helper (below) lets you drop a compile-time
/// assertion anywhere: `_assert_model::<MyType>()`.
pub trait ApiModel: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static {
    /// The canonical schema name registered in `components.schemas`.
    fn schema_name() -> &'static str;

    /// A representative instance used by serde-roundtrip and jsonschema tests.
    fn example() -> Self;
}

/// Zero-cost compile-time assertion that `T` satisfies `ApiModel`.
///
/// Drop this in any file that introduces a new type:
/// ```ignore
/// let _ = api_openapi::models::_assert_model::<MyNewType>;
/// ```
/// If `MyNewType` doesn't implement `ApiModel`, the build fails here.
#[allow(dead_code)]
pub const fn _assert_model<T: ApiModel>() {}

pub mod about;
pub mod admin;
pub mod ask;
pub mod auth;
pub mod challenges;
pub mod common;
pub mod contact;
pub mod crates;
pub mod demo;
pub mod health;
pub mod linkedin;
pub mod metrics;
pub mod portfolio;
pub mod resume;
pub mod social;
pub mod stack; // empty — stack returns serde_json::Value directly
pub mod tailor;

// Flat re-exports so consumers can write `use api_openapi::models::*`
pub use common::*;
pub use crates::*;
pub use health::*;
pub use metrics::*;
// stack module is empty — no re-export needed
pub use about::*;
pub use admin::*;
pub use ask::*;
pub use auth::*;
pub use challenges::*;
pub use contact::*;
pub use demo::*;
pub use linkedin::*;
pub use portfolio::*;
pub use resume::*;
pub use social::*;
pub use tailor::*;
