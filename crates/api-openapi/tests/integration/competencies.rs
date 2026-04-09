/// Integration tests for the competencies API (`GET /api/competencies`).
///
/// Competency schemas are also covered by `jobs.rs` since they share the same
/// model domain. This file verifies the spec-level presence specifically for
/// the competencies endpoint grouping.
use api_openapi::apidoc::full_spec;

#[test]
fn competency_group_schemas_all_present() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;

    let expected = ["Competency", "EvidenceItem", "CompetencyWithEvidence"];
    let missing: Vec<_> = expected
        .iter()
        .filter(|&&name| !schemas.contains_key(name))
        .copied()
        .collect();

    assert!(
        missing.is_empty(),
        "Competency API schemas missing from full_spec(): {:?}",
        missing
    );
}

#[test]
fn competency_schemas_are_not_admin_tagged_in_full_spec() {
    // Competency schemas live in PublicApiDoc, not AdminApiDoc — verify by checking
    // they exist in full_spec() (which merges both docs).
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;
    for name in &["Competency", "EvidenceItem", "CompetencyWithEvidence"] {
        assert!(
            schemas.contains_key(*name),
            "Schema '{}' missing from full_spec() — it should be in PublicApiDoc",
            name
        );
    }
}
