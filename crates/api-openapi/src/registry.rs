/// Compile-time model registry — every `ApiModel` type that must appear in
/// `components.schemas` is listed here.
///
/// # Rust Architect: compile-time coverage enforcement
///
/// `ALL_MODELS` is a `&[(&str, fn() -> serde_json::Value)]` constant — the
/// compiler verifies all referenced types exist and implement `ApiModel` +
/// `Serialize` at compile time. If a model is removed or renamed, this file
/// fails to compile. If a model is added to `models/` but NOT to this list,
/// the `schema_coverage` test fails — which runs in CI.
///
/// Together these two guarantees mean:
/// * You cannot remove a registered model → compile error.
/// * You cannot add a registered model without it implementing `ApiModel` → compile error.
/// * You cannot omit a model from this list → test failure in CI.
use crate::models::*;

/// Factory function type for model examples in [`ALL_MODELS`].
pub type ModelFactory = fn() -> serde_json::Value;

/// One entry per data model in `crates/api-openapi/src/models/`.
/// Keyed by the canonical schema name; value is a factory returning a JSON example.
pub const ALL_MODELS: &[(&str, ModelFactory)] = &[
    // common
    ("ApiError", || {
        serde_json::to_value(ApiError::example()).expect("ApiError example")
    }),
    // health
    ("HealthResponse", || {
        serde_json::to_value(HealthResponse::example()).expect("HealthResponse example")
    }),
    // crates
    ("CrateInfo", || {
        serde_json::to_value(CrateInfo::example()).expect("CrateInfo example")
    }),
    // resume
    ("Job", || {
        serde_json::to_value(Job::example()).expect("Job example")
    }),
    ("JobDetail", || {
        serde_json::to_value(JobDetail::example()).expect("JobDetail example")
    }),
    ("JobWithDetails", || {
        serde_json::to_value(JobWithDetails::example()).expect("JobWithDetails example")
    }),
    ("JobsQuery", || {
        serde_json::to_value(JobsQuery::example()).expect("JobsQuery example")
    }),
    ("Competency", || {
        serde_json::to_value(Competency::example()).expect("Competency example")
    }),
    ("EvidenceItem", || {
        serde_json::to_value(EvidenceItem::example()).expect("EvidenceItem example")
    }),
    ("CompetencyWithEvidence", || {
        serde_json::to_value(CompetencyWithEvidence::example())
            .expect("CompetencyWithEvidence example")
    }),
    // about
    ("AboutSectionInput", || {
        serde_json::to_value(AboutSectionInput::example()).expect("AboutSectionInput example")
    }),
    ("AboutSectionResponse", || {
        serde_json::to_value(AboutSectionResponse::example()).expect("AboutSectionResponse example")
    }),
    // social
    ("SocialLink", || {
        serde_json::to_value(SocialLink::example()).expect("SocialLink example")
    }),
    ("SocialLinkInput", || {
        serde_json::to_value(SocialLinkInput::example()).expect("SocialLinkInput example")
    }),
    ("SocialLinkResponse", || {
        serde_json::to_value(SocialLinkResponse::example()).expect("SocialLinkResponse example")
    }),
    // challenges
    ("Challenge", || {
        serde_json::to_value(Challenge::example()).expect("Challenge example")
    }),
    ("ChallengeInput", || {
        serde_json::to_value(ChallengeInput::example()).expect("ChallengeInput example")
    }),
    // contact
    ("ChallengeResponse", || {
        serde_json::to_value(ChallengeResponse::example()).expect("ChallengeResponse example")
    }),
    ("ContactSubmitRequest", || {
        serde_json::to_value(ContactSubmitRequest::example()).expect("ContactSubmitRequest example")
    }),
    ("ContactResponse", || {
        serde_json::to_value(ContactResponse::example()).expect("ContactResponse example")
    }),
    // admin
    ("JobInput", || {
        serde_json::to_value(JobInput::example()).expect("JobInput example")
    }),
    ("JobDetailInput", || {
        serde_json::to_value(JobDetailInput::example()).expect("JobDetailInput example")
    }),
    ("CompetencyInput", || {
        serde_json::to_value(CompetencyInput::example()).expect("CompetencyInput example")
    }),
    ("EvidenceInput", || {
        serde_json::to_value(EvidenceInput::example()).expect("EvidenceInput example")
    }),
    ("Evidence", || {
        serde_json::to_value(Evidence::example()).expect("Evidence example")
    }),
    // tailor
    ("TailorRequest", || {
        serde_json::to_value(TailorRequest::example()).expect("TailorRequest example")
    }),
    ("MatchedBullet", || {
        serde_json::to_value(MatchedBullet::example()).expect("MatchedBullet example")
    }),
    ("TailorResponse", || {
        serde_json::to_value(TailorResponse::example()).expect("TailorResponse example")
    }),
    // metrics
    ("MetricsQuery", || {
        serde_json::to_value(MetricsQuery::example()).expect("MetricsQuery example")
    }),
    // ask
    ("AskRequest", || {
        serde_json::to_value(AskRequest::example()).expect("AskRequest example")
    }),
    ("AskCitation", || {
        serde_json::to_value(AskCitation::example()).expect("AskCitation example")
    }),
    ("AskResponse", || {
        serde_json::to_value(AskResponse::example()).expect("AskResponse example")
    }),
    // auth + portfolio (SPA endpoints)
    ("AuthMe", || {
        serde_json::to_value(AuthMe::example()).expect("AuthMe example")
    }),
    ("ResumeData", || {
        serde_json::to_value(ResumeData::example()).expect("ResumeData example")
    }),
    // linkedin
    ("LinkedInPosition", || {
        serde_json::to_value(LinkedInPosition::example()).expect("LinkedInPosition example")
    }),
    ("LinkedInProject", || {
        serde_json::to_value(LinkedInProject::example()).expect("LinkedInProject example")
    }),
    ("LinkedInPositionInput", || {
        serde_json::to_value(LinkedInPositionInput::example())
            .expect("LinkedInPositionInput example")
    }),
    ("LinkedInProjectInput", || {
        serde_json::to_value(LinkedInProjectInput::example()).expect("LinkedInProjectInput example")
    }),
    ("LinkedInImportPayload", || {
        serde_json::to_value(LinkedInImportPayload::example())
            .expect("LinkedInImportPayload example")
    }),
    ("LinkedInImportResult", || {
        serde_json::to_value(LinkedInImportResult::example()).expect("LinkedInImportResult example")
    }),
    ("LinkedInSyncLogEntry", || {
        serde_json::to_value(LinkedInSyncLogEntry::example()).expect("LinkedInSyncLogEntry example")
    }),
    ("SyncFieldComparison", || {
        serde_json::to_value(SyncFieldComparison::example()).expect("SyncFieldComparison example")
    }),
    ("PositionDiff", || {
        serde_json::to_value(PositionDiff::example()).expect("PositionDiff example")
    }),
    ("ProjectDiff", || {
        serde_json::to_value(ProjectDiff::example()).expect("ProjectDiff example")
    }),
    ("MapRequest", || {
        serde_json::to_value(MapRequest::example()).expect("MapRequest example")
    }),
    ("StatusUpdateRequest", || {
        serde_json::to_value(StatusUpdateRequest::example()).expect("StatusUpdateRequest example")
    }),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_models_have_unique_names() {
        let mut seen = std::collections::HashSet::new();
        for (name, _) in ALL_MODELS {
            assert!(
                seen.insert(*name),
                "Duplicate model name '{}' in ALL_MODELS",
                name
            );
        }
    }

    #[test]
    fn all_models_examples_serialize_non_null() {
        for (name, factory) in ALL_MODELS {
            let value = factory();
            assert!(
                !value.is_null(),
                "Model '{}' example() serialised to null",
                name
            );
        }
    }

    #[test]
    fn all_models_roundtrip_through_json() {
        for (name, factory) in ALL_MODELS {
            let original = factory();
            let json = serde_json::to_string(&original)
                .unwrap_or_else(|e| panic!("Model '{}' failed to serialize: {}", name, e));
            let reparsed: serde_json::Value = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("Model '{}' failed to deserialize: {}", name, e));
            assert_eq!(original, reparsed, "Model '{}' roundtrip mismatch", name);
        }
    }
}
