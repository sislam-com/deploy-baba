/// E2e tests for the admin OpenAPI spec (`GET /api/openapi-admin.json`).
///
/// The admin spec is served behind `require_auth` middleware and rendered by
/// rapidoc at `GET /docs/admin`. It must contain:
/// - All public paths (a superset of the public spec)
/// - All admin paths under `/api/admin/*`
/// - Admin-only schemas (`JobInput`, `SocialLinkInput`, `AboutSectionInput`, etc.)
/// - `cookieAuth` and `bearerAuth` security schemes
/// - The correct spec title and version
use api_openapi::apidoc::{full_spec, AdminApiDoc, PublicApiDoc};
use utoipa::OpenApi;

// ─── Security schemes ────────────────────────────────────────────────────────

#[test]
fn full_spec_has_cookie_auth_scheme() {
    let spec = full_spec();
    let schemes = &spec
        .components
        .as_ref()
        .expect("components")
        .security_schemes;
    assert!(
        schemes.contains_key("cookieAuth"),
        "full_spec() missing cookieAuth security scheme"
    );
}

#[test]
fn full_spec_has_bearer_auth_scheme() {
    let spec = full_spec();
    let schemes = &spec
        .components
        .as_ref()
        .expect("components")
        .security_schemes;
    assert!(
        schemes.contains_key("bearerAuth"),
        "full_spec() missing bearerAuth security scheme"
    );
}

#[test]
fn admin_api_doc_has_both_security_schemes() {
    let spec = AdminApiDoc::openapi();
    let schemes = spec
        .components
        .as_ref()
        .map(|c| &c.security_schemes)
        .expect("AdminApiDoc components");
    assert!(
        schemes.contains_key("cookieAuth"),
        "AdminApiDoc missing cookieAuth"
    );
    assert!(
        schemes.contains_key("bearerAuth"),
        "AdminApiDoc missing bearerAuth"
    );
}

// ─── Admin schema coverage ───────────────────────────────────────────────────

#[test]
fn full_spec_contains_all_admin_schemas() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    let admin_schemas = [
        "JobInput",
        "JobDetailInput",
        "CompetencyInput",
        "EvidenceInput",
        "Evidence",
        "AboutSectionInput",
        "SocialLinkInput",
        "SocialLinkResponse",
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

// ─── Spec integrity ──────────────────────────────────────────────────────────

#[test]
fn full_spec_title_and_version() {
    let spec = full_spec();
    assert_eq!(spec.info.title, "deploy-baba Portfolio & API");
    assert!(!spec.info.version.is_empty(), "full_spec version is empty");
}

#[test]
fn full_spec_has_at_least_ten_schemas() {
    let spec = full_spec();
    let count = spec
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    assert!(
        count >= 10,
        "expected at least 10 schemas in full_spec(), got {}",
        count
    );
}

#[test]
fn full_spec_does_not_duplicate_schemas_vs_individual_docs() {
    let full = full_spec();
    let public_doc = PublicApiDoc::openapi();
    let admin_doc = AdminApiDoc::openapi();

    let full_count = full
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    let pub_count = public_doc
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    let adm_count = admin_doc
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);

    // full_spec is a merge of Public + Admin — schema count must equal pub + adm
    // (no overlapping schemas between the two docs by design)
    assert_eq!(
        full_count,
        pub_count + adm_count,
        "full_spec schema count ({}) != PublicApiDoc ({}) + AdminApiDoc ({}) — \
         check for duplicate schemas between the two docs",
        full_count,
        pub_count,
        adm_count
    );
}

#[test]
fn full_spec_public_doc_has_schemas() {
    let spec = PublicApiDoc::openapi();
    let count = spec
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    assert!(count > 0, "PublicApiDoc has no schemas");
}

/// Verify the full spec serialises to JSON (the `/api/openapi-admin.json` response).
#[test]
fn full_spec_serialises_to_json() {
    let spec = full_spec();
    let json = serde_json::to_string(&spec).expect("full spec must serialise to JSON");
    assert!(!json.is_empty());
    let _: serde_json::Value = serde_json::from_str(&json).expect("full spec JSON must be valid");
}

// ─── Relationship between public and full ────────────────────────────────────

#[test]
fn full_spec_is_superset_of_public_spec_schemas() {
    use api_openapi::filter::public_view;

    let full = full_spec();
    let public = public_view(&full);

    let full_schemas = &full.components.as_ref().expect("full components").schemas;
    let public_schemas = &public
        .components
        .as_ref()
        .expect("public components")
        .schemas;

    // Every schema in the public spec must also be in the full spec
    for name in public_schemas.keys() {
        assert!(
            full_schemas.contains_key(name),
            "Schema '{}' is in public spec but not in full spec — internal inconsistency",
            name
        );
    }
}
