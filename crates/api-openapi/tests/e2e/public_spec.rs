/// E2e tests for the public OpenAPI spec (`GET /api/openapi.json`).
///
/// Tests what `public_view()` guarantees when applied to `full_spec()` from
/// `api-openapi`. Because `full_spec()` carries no path operations (only
/// `components.schemas`), the schema GC pass produces an empty schema set —
/// this is the expected behaviour for a schema-only spec.
///
/// The important e2e guarantees verified here:
/// - No `cookieAuth` / `bearerAuth` security schemes in the public spec.
/// - No top-level `security` requirements.
/// - No admin-tagged operations (if any paths are present).
/// - The public spec JSON is valid and serialisable.
/// - The public spec has fewer schemas than the full spec (GC removes everything
///   when there are no path references, which is correct for this scenario).
///
/// Schema-survival tests live in `tests/integration/` where each public schema
/// is verified to exist in `full_spec()` directly (before filtering).
use api_openapi::{apidoc::full_spec, filter::public_view};

fn public() -> utoipa::openapi::OpenApi {
    public_view(&full_spec())
}

// ─── Security ───────────────────────────────────────────────────────────────

#[test]
fn public_spec_has_no_cookie_auth_scheme() {
    let spec = public();
    if let Some(components) = &spec.components {
        assert!(
            !components.security_schemes.contains_key("cookieAuth"),
            "cookieAuth security scheme must not appear in the public spec"
        );
    }
}

#[test]
fn public_spec_has_no_bearer_auth_scheme() {
    let spec = public();
    if let Some(components) = &spec.components {
        assert!(
            !components.security_schemes.contains_key("bearerAuth"),
            "bearerAuth security scheme must not appear in the public spec"
        );
    }
}

#[test]
fn public_spec_has_no_top_level_security() {
    let spec = public();
    assert!(
        spec.security.is_none(),
        "top-level security field must be absent from the public spec"
    );
}

// ─── Admin paths ─────────────────────────────────────────────────────────────

#[test]
fn public_spec_has_no_admin_tagged_operations() {
    let spec = public();
    for (path, item) in &spec.paths.paths {
        for op in item.operations.values() {
            let has_admin_tag = op
                .tags
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .any(|t| t == "admin");
            assert!(
                !has_admin_tag,
                "Path '{}' has an admin-tagged operation in the public spec",
                path
            );
        }
    }
}

// ─── Filter removes admin-only schemas via GC ────────────────────────────────
//
// When `full_spec()` has no path operations (api-openapi is schema-only),
// the GC pass clears ALL schemas from the public spec. The following tests
// verify that admin-only schemas are absent after filtering — a consequence
// of the GC pass being applied to a path-free spec.

#[test]
fn public_spec_does_not_contain_job_input() {
    let spec = public();
    if let Some(c) = &spec.components {
        assert!(
            !c.schemas.contains_key("JobInput"),
            "JobInput (admin write model) must not appear in the public spec"
        );
    }
}

#[test]
fn public_spec_does_not_contain_social_link_input() {
    let spec = public();
    if let Some(c) = &spec.components {
        assert!(
            !c.schemas.contains_key("SocialLinkInput"),
            "SocialLinkInput (admin write model) must not appear in the public spec"
        );
    }
}

#[test]
fn public_spec_does_not_contain_about_section_input() {
    let spec = public();
    if let Some(c) = &spec.components {
        assert!(
            !c.schemas.contains_key("AboutSectionInput"),
            "AboutSectionInput (admin write model) must not appear in the public spec"
        );
    }
}

// ─── Spec integrity ──────────────────────────────────────────────────────────

#[test]
fn public_spec_has_title_and_version() {
    let spec = public();
    assert!(!spec.info.title.is_empty(), "public spec title is empty");
    assert!(
        !spec.info.version.is_empty(),
        "public spec version is empty"
    );
}

/// The public spec must serialise to JSON (the `/api/openapi.json` response).
#[test]
fn public_spec_serialises_to_json() {
    let spec = public();
    let json = serde_json::to_string(&spec).expect("public spec must serialise to JSON");
    assert!(!json.is_empty());
    let _: serde_json::Value = serde_json::from_str(&json).expect("public spec JSON must be valid");
}

/// When filtering a path-free spec, GC removes all schemas.
/// This verifies the filter produces a consistent (smaller or equal) schema set.
#[test]
fn public_spec_has_no_more_schemas_than_full_spec() {
    let full = full_spec();
    let public = public_view(&full);

    let full_count = full
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    let public_count = public
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);

    assert!(
        public_count <= full_count,
        "public spec must have no more schemas than full spec. full={}, public={}",
        full_count,
        public_count
    );
}
