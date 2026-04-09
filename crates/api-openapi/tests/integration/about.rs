/// Integration tests for the about API (`GET /about`, `/api/admin/about`).
use api_openapi::{
    apidoc::full_spec,
    models::{AboutSectionInput, AboutSectionResponse, ApiModel},
};

#[test]
fn about_schemas_present_in_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    for name in &["AboutSectionInput", "AboutSectionResponse"] {
        assert!(
            schemas.contains_key(*name),
            "About schema '{}' missing from full_spec()",
            name
        );
    }
}

#[test]
fn about_section_input_example_has_required_fields() {
    let input = AboutSectionInput::example();
    assert!(
        !input.slug.is_empty(),
        "AboutSectionInput.slug should not be empty"
    );
    assert!(
        !input.heading.is_empty(),
        "AboutSectionInput.heading should not be empty"
    );
    assert!(
        !input.body.is_empty(),
        "AboutSectionInput.body should not be empty"
    );
}

#[test]
fn about_section_response_example_has_id_and_fields() {
    let resp = AboutSectionResponse::example();
    assert!(
        resp.id > 0,
        "AboutSectionResponse.id should be a positive integer"
    );
    assert!(
        !resp.slug.is_empty(),
        "AboutSectionResponse.slug should not be empty"
    );
    assert!(
        !resp.heading.is_empty(),
        "AboutSectionResponse.heading should not be empty"
    );
}

#[test]
fn about_section_input_roundtrips() {
    let input = AboutSectionInput::example();
    let json = serde_json::to_string(&input).expect("serialize");
    let back: AboutSectionInput = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(input.slug, back.slug);
    assert_eq!(input.heading, back.heading);
    assert_eq!(input.body, back.body);
}

#[test]
fn about_section_input_is_admin_only_in_public_spec() {
    use api_openapi::filter::public_view;
    let public = public_view(&full_spec());

    // AboutSectionInput is an admin write type — should be GC'd from public spec
    if let Some(components) = &public.components {
        assert!(
            !components.schemas.contains_key("AboutSectionInput"),
            "AboutSectionInput leaked into public spec (it is an admin write model)"
        );
    }
}

#[test]
fn about_section_response_is_in_public_api_doc() {
    use api_openapi::apidoc::PublicApiDoc;
    use utoipa::OpenApi;
    let spec = PublicApiDoc::openapi();
    let schemas = &spec
        .components
        .as_ref()
        .expect("PublicApiDoc components")
        .schemas;
    assert!(
        schemas.contains_key("AboutSectionResponse"),
        "AboutSectionResponse should be in PublicApiDoc (public read model)"
    );
}
