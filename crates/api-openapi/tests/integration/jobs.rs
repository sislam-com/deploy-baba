/// Integration tests for the jobs/resume API (`GET /api/jobs`, `GET /api/jobs/:slug`).
use api_openapi::{
    apidoc::full_spec,
    models::{
        ApiModel, Competency, CompetencyWithEvidence, EvidenceItem, Job, JobDetail, JobWithDetails,
        JobsQuery,
    },
};

#[test]
fn job_schemas_present_in_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;
    for name in &["Job", "JobDetail", "JobWithDetails", "JobsQuery"] {
        assert!(
            schemas.contains_key(*name),
            "Schema '{}' missing from full_spec()",
            name
        );
    }
}

#[test]
fn job_example_has_required_fields() {
    let job = Job::example();
    assert!(!job.title.is_empty(), "Job.title should not be empty");
    assert!(!job.company.is_empty(), "Job.company should not be empty");
    assert!(!job.slug.is_empty(), "Job.slug should not be empty");
}

#[test]
fn job_detail_example_has_required_fields() {
    let detail = JobDetail::example();
    assert!(
        !detail.detail_text.is_empty(),
        "JobDetail.detail_text should not be empty"
    );
}

#[test]
fn job_with_details_example_has_job_and_details() {
    let jwd = JobWithDetails::example();
    assert!(
        !jwd.job.title.is_empty(),
        "JobWithDetails.job.title should not be empty"
    );
    assert!(
        !jwd.details.is_empty(),
        "JobWithDetails.details should not be empty"
    );
}

#[test]
fn jobs_query_example_roundtrips() {
    let q = JobsQuery::example();
    let json = serde_json::to_string(&q).expect("serialize JobsQuery");
    let back: JobsQuery = serde_json::from_str(&json).expect("deserialize JobsQuery");
    assert_eq!(q.view, back.view);
}

#[test]
fn competency_schemas_present_in_spec() {
    let spec = full_spec();
    let schemas = &spec.components.as_ref().expect("components").schemas;
    for name in &["Competency", "EvidenceItem", "CompetencyWithEvidence"] {
        assert!(
            schemas.contains_key(*name),
            "Schema '{}' missing from full_spec()",
            name
        );
    }
}

#[test]
fn competency_example_has_required_fields() {
    let c = Competency::example();
    assert!(!c.name.is_empty(), "Competency.name should not be empty");
    assert!(!c.slug.is_empty(), "Competency.slug should not be empty");
}

#[test]
fn evidence_item_example_has_required_fields() {
    let e = EvidenceItem::example();
    assert!(
        !e.job_slug.is_empty(),
        "EvidenceItem.job_slug should not be empty"
    );
    assert!(
        !e.company.is_empty(),
        "EvidenceItem.company should not be empty"
    );
}

#[test]
fn competency_with_evidence_example_has_nested_data() {
    let cwe = CompetencyWithEvidence::example();
    assert!(
        !cwe.competency.name.is_empty(),
        "CompetencyWithEvidence.competency.name empty"
    );
    assert!(
        !cwe.evidence.is_empty(),
        "CompetencyWithEvidence.evidence should not be empty"
    );
}
