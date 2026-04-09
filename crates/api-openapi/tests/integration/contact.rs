/// Integration tests for the contact API
/// (`POST /api/contact`, `GET /api/contact/challenge`, `POST /api/contact/verify`).
use api_openapi::{
    apidoc::full_spec,
    models::{ApiModel, ChallengeResponse, ContactResponse, ContactSubmitRequest},
};

#[test]
fn contact_schemas_present_in_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    for name in &[
        "ChallengeResponse",
        "ContactSubmitRequest",
        "ContactResponse",
    ] {
        assert!(
            schemas.contains_key(*name),
            "Contact schema '{}' missing from full_spec()",
            name
        );
    }
}

#[test]
fn challenge_response_example_has_required_fields() {
    let cr = ChallengeResponse::example();
    assert!(
        !cr.nonce.is_empty(),
        "ChallengeResponse.nonce should not be empty"
    );
    assert!(
        !cr.signature.is_empty(),
        "ChallengeResponse.signature should not be empty"
    );
    assert!(
        cr.difficulty > 0,
        "ChallengeResponse.difficulty should be positive"
    );
}

#[test]
fn contact_submit_request_example_has_required_fields() {
    let req = ContactSubmitRequest::example();
    assert!(
        !req.name.is_empty(),
        "ContactSubmitRequest.name should not be empty"
    );
    assert!(
        !req.email.is_empty(),
        "ContactSubmitRequest.email should not be empty"
    );
    assert!(
        !req.message.is_empty(),
        "ContactSubmitRequest.message should not be empty"
    );
}

#[test]
fn contact_response_example_has_message() {
    let resp = ContactResponse::example();
    assert!(
        !resp.message.is_empty(),
        "ContactResponse.message should not be empty"
    );
}

#[test]
fn contact_submit_request_roundtrips() {
    let req = ContactSubmitRequest::example();
    let json = serde_json::to_string(&req).expect("serialize");
    let back: ContactSubmitRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(req.name, back.name);
    assert_eq!(req.email, back.email);
    assert_eq!(req.message, back.message);
}

#[test]
fn contact_schemas_are_in_public_api_doc() {
    use api_openapi::apidoc::PublicApiDoc;
    use utoipa::OpenApi;
    let spec = PublicApiDoc::openapi();
    let schemas = &spec
        .components
        .as_ref()
        .expect("PublicApiDoc components")
        .schemas;

    // Contact schemas belong in PublicApiDoc (not AdminApiDoc)
    for name in &[
        "ChallengeResponse",
        "ContactSubmitRequest",
        "ContactResponse",
    ] {
        assert!(
            schemas.contains_key(*name),
            "Contact schema '{}' missing from PublicApiDoc",
            name
        );
    }
}
