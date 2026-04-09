/// Integration tests for the admin API (`/api/admin/*`).
///
/// Admin schemas must be present in `full_spec()` (the authenticated admin endpoint)
/// but must NOT appear in the public-filtered spec.
use api_openapi::{
    apidoc::full_spec,
    models::{ApiModel, CompetencyInput, Evidence, EvidenceInput, JobDetailInput, JobInput},
};
use utoipa::OpenApi;

#[test]
fn admin_schemas_present_in_full_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    let admin_schemas = [
        "JobInput",
        "JobDetailInput",
        "CompetencyInput",
        "EvidenceInput",
        "Evidence",
    ];

    let missing: Vec<_> = admin_schemas
        .iter()
        .filter(|&&name| !schemas.contains_key(name))
        .copied()
        .collect();

    assert!(
        missing.is_empty(),
        "Admin schemas missing from full_spec(): {:?}",
        missing
    );
}

#[test]
fn admin_schemas_absent_from_public_spec() {
    use api_openapi::filter::public_view;
    let public = public_view(&full_spec());

    // Admin-only schemas must be GC'd by the public filter
    let admin_only = [
        "JobInput",
        "JobDetailInput",
        "CompetencyInput",
        "EvidenceInput",
    ];

    if let Some(components) = &public.components {
        let schemas = &components.schemas;
        for name in &admin_only {
            assert!(
                !schemas.contains_key(*name),
                "Admin schema '{}' leaked into public spec — check public_view() GC logic",
                name
            );
        }
    }
}

#[test]
fn job_input_example_has_required_fields() {
    let input = JobInput::example();
    assert!(
        !input.title.is_empty(),
        "JobInput.title should not be empty"
    );
    assert!(
        !input.company.is_empty(),
        "JobInput.company should not be empty"
    );
    assert!(!input.slug.is_empty(), "JobInput.slug should not be empty");
}

#[test]
fn job_detail_input_example_has_detail_text() {
    let input = JobDetailInput::example();
    assert!(
        !input.detail_text.is_empty(),
        "JobDetailInput.detail_text should not be empty"
    );
}

#[test]
fn competency_input_example_has_required_fields() {
    let input = CompetencyInput::example();
    assert!(
        !input.name.is_empty(),
        "CompetencyInput.name should not be empty"
    );
    assert!(
        !input.slug.is_empty(),
        "CompetencyInput.slug should not be empty"
    );
}

#[test]
fn evidence_input_example_has_required_ids() {
    let input = EvidenceInput::example();
    assert!(
        input.competency_id > 0,
        "EvidenceInput.competency_id should be positive"
    );
    assert!(input.job_id > 0, "EvidenceInput.job_id should be positive");
}

#[test]
fn evidence_example_has_required_ids() {
    let e = Evidence::example();
    assert!(e.id > 0, "Evidence.id should be positive");
    assert!(
        e.competency_id > 0,
        "Evidence.competency_id should be positive"
    );
}

#[test]
fn admin_api_doc_has_security_schemes() {
    use api_openapi::apidoc::AdminApiDoc;
    let spec = AdminApiDoc::openapi();
    let schemes = spec
        .components
        .as_ref()
        .map(|c| &c.security_schemes)
        .expect("AdminApiDoc components");
    assert!(
        schemes.contains_key("cookieAuth"),
        "AdminApiDoc missing cookieAuth scheme"
    );
    assert!(
        schemes.contains_key("bearerAuth"),
        "AdminApiDoc missing bearerAuth scheme"
    );
}
