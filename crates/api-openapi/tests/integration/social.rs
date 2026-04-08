/// Integration tests for the social links API (`GET /api/social-links`, `/api/admin/social-links`).
use api_openapi::{
    apidoc::full_spec,
    models::{ApiModel, SocialLink, SocialLinkInput, SocialLinkResponse},
};

#[test]
fn social_schemas_present_in_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    for name in &["SocialLink", "SocialLinkInput", "SocialLinkResponse"] {
        assert!(
            schemas.contains_key(*name),
            "Social schema '{}' missing from full_spec()",
            name
        );
    }
}

#[test]
fn social_link_example_has_url_and_label() {
    let link = SocialLink::example();
    assert!(!link.url.is_empty(), "SocialLink.url should not be empty");
    assert!(
        !link.label.is_empty(),
        "SocialLink.label should not be empty"
    );
}

#[test]
fn social_link_input_example_has_required_fields() {
    let input = SocialLinkInput::example();
    assert!(
        !input.platform.is_empty(),
        "SocialLinkInput.platform should not be empty"
    );
    assert!(
        !input.url.is_empty(),
        "SocialLinkInput.url should not be empty"
    );
    assert!(
        !input.label.is_empty(),
        "SocialLinkInput.label should not be empty"
    );
}

#[test]
fn social_link_response_example_has_id_and_platform() {
    let resp = SocialLinkResponse::example();
    assert!(
        resp.id > 0,
        "SocialLinkResponse.id should be a positive integer"
    );
    assert!(
        !resp.platform.is_empty(),
        "SocialLinkResponse.platform should not be empty"
    );
}

#[test]
fn social_link_roundtrips() {
    let link = SocialLink::example();
    let json = serde_json::to_string(&link).expect("serialize");
    let back: SocialLink = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(link.url, back.url);
    assert_eq!(link.label, back.label);
}

#[test]
fn social_link_input_is_admin_only_in_public_spec() {
    use api_openapi::filter::public_view;
    let public = public_view(&full_spec());

    if let Some(components) = &public.components {
        assert!(
            !components.schemas.contains_key("SocialLinkInput"),
            "SocialLinkInput leaked into public spec (it is an admin write model)"
        );
    }
}
