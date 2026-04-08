/// Registry integration tests — verifies ALL_MODELS ↔ full_spec() schemas bijection.
///
/// Unlike the inline unit tests in `src/registry.rs` (which test the registry in isolation),
/// these tests integrate the registry with the full spec to verify consistency.
use api_openapi::{apidoc::full_spec, registry::ALL_MODELS};

#[test]
fn all_registered_models_present_in_full_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    let mut missing = Vec::new();
    for (name, _) in ALL_MODELS {
        if !schemas.contains_key(*name) {
            missing.push(*name);
        }
    }

    assert!(
        missing.is_empty(),
        "Models in ALL_MODELS not found in full_spec() schemas: {:?}\n\
         Add them to the components(schemas(...)) list in apidoc.rs.",
        missing
    );
}

#[test]
fn full_spec_schema_count_matches_registry() {
    let spec = full_spec();
    let schema_count = spec
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    let registry_count = ALL_MODELS.len();

    assert_eq!(
        registry_count, schema_count,
        "ALL_MODELS ({} entries) and full_spec() ({} schemas) are out of sync",
        registry_count, schema_count
    );
}
