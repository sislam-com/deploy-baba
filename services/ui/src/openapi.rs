/// OpenAPI spec — single source of truth for schema definitions is
/// `api_openapi::models`; this file wires the handler paths into the spec.
///
/// NOTE: utoipa-axum (which would eliminate the `paths(...)` list) requires
/// utoipa v5 but this workspace uses v4. Migration is tracked as W-APIO.4.4 and
/// deferred until the workspace upgrades to utoipa v5. For now we keep the
/// hand-maintained list; all *schema types* come from `api_openapi::models`.
use std::collections::HashMap;

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "cookieAuth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("auth_token"))),
        );
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// API Version Metadata Modifier (ADR-024)
///
/// Adds version information and deprecation schedule to the OpenAPI spec.
/// This enables API consumers to understand versioning and plan migrations.
pub struct ApiVersionModifier;

impl Modify for ApiVersionModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Update API version to reflect URL-based versioning (v1.0.0)
        openapi.info.version = "1.0.0".to_string();

        // Add version metadata extensions
        // These help API consumers understand the versioning strategy
        let version_metadata = serde_json::json!({
            "x-api-versioning": {
                "strategy": "url-path",
                "current": "v1",
                "deprecated": [],
                "sunset": null,
                "documentation": "https://sislam.com/docs/api-versioning"
            }
        });

        // Convert serde_json::Map to HashMap for utoipa compatibility
        let mut extensions: HashMap<String, serde_json::Value> = HashMap::new();
        for (key, value) in version_metadata.as_object().unwrap() {
            extensions.insert(key.clone(), value.clone());
        }

        // Merge into existing extensions or set new ones
        if let Some(existing) = openapi.info.extensions.as_mut() {
            for (key, value) in extensions {
                existing.insert(key, value);
            }
        } else {
            openapi.info.extensions = Some(extensions);
        }
    }
}

/// Full combined spec — all handler paths, all schemas, security schemes.
///
/// Served at `/api/openapi.json` (unauthenticated) so the public `/docs` page
/// shows every endpoint including admin. Admin paths carry `security` annotations
/// so RapiDoc renders lock icons; the actual `require_auth` middleware on those
/// routes is unchanged.
///
/// `demo/config/parse` and `demo/spec/generate` are intentionally excluded —
/// those endpoints are internal utilities, not public API surface.
///
/// The `tailor` tag is reserved for the upcoming W-RST pipeline (job description
/// textbox → RAG + Anthropic → downloadable docx/pdf).
#[derive(OpenApi)]
#[openapi(
    info(
        title = "deploy-baba Portfolio & API",
        version = "0.1.0",
        description = "Live demos and documentation for the deploy-baba ecosystem"
    ),
    paths(
        // ── Health ───────────────────────────────────────────────────────────
        crate::routes::health::get_health,
        // ── Crates ───────────────────────────────────────────────────────────
        crate::routes::api::crates::list_crates,
        crate::routes::api::crates::get_crate,
        // ── Stack ────────────────────────────────────────────────────────────
        crate::routes::api::stack::get_stack,
        // ── Portfolio (public read) ──────────────────────────────────────────
        crate::routes::api::jobs::list_jobs,
        crate::routes::api::jobs::get_job,
        crate::routes::api::competencies::list_competencies,
        crate::routes::api::competencies::get_competency,
        crate::routes::api::resume_data::get_resume_data,
        crate::routes::api::about::list_about_sections,
        crate::routes::api::social_links::list_social_links,
        // ── Challenges (public read) ─────────────────────────────────────────
        crate::routes::api::challenges::list_challenges,
        crate::routes::api::challenges::get_challenge,
        crate::routes::api::challenges::list_challenges_for_job,
        // ── Contact ──────────────────────────────────────────────────────────
        crate::routes::contact::challenge_issue,
        crate::routes::contact::contact_submit,
        // ── Auth ─────────────────────────────────────────────────────────────
        crate::routes::api::auth_me::auth_me,
        // ── Ask (RAG) ────────────────────────────────────────────────────────
        crate::routes::api::ask::ask,
        // ── Admin — jobs ─────────────────────────────────────────────────────
        crate::routes::api::admin::create_job,
        crate::routes::api::admin::update_job,
        crate::routes::api::admin::delete_job,
        crate::routes::api::admin::create_job_detail,
        crate::routes::api::admin::update_job_detail,
        crate::routes::api::admin::delete_job_detail,
        // ── Admin — competencies ─────────────────────────────────────────────
        crate::routes::api::admin::create_competency,
        crate::routes::api::admin::update_competency,
        crate::routes::api::admin::delete_competency,
        crate::routes::api::admin::create_evidence,
        crate::routes::api::admin::update_evidence,
        crate::routes::api::admin::delete_evidence,
        // ── Admin — about ────────────────────────────────────────────────────
        crate::routes::api::admin::create_about_section,
        crate::routes::api::admin::update_about_section,
        crate::routes::api::admin::delete_about_section,
        // ── Admin — social links ─────────────────────────────────────────────
        crate::routes::api::admin::create_social_link,
        crate::routes::api::admin::update_social_link,
        crate::routes::api::admin::delete_social_link,
        // ── Admin — challenges ──────────────────────────────────────────────
        crate::routes::api::admin::create_challenge,
        crate::routes::api::admin::update_challenge,
        crate::routes::api::admin::delete_challenge,
    ),
    components(schemas(
        api_openapi::models::ApiError,
        api_openapi::models::HealthResponse,
        api_openapi::models::CrateInfo,
        api_openapi::models::Job,
        api_openapi::models::JobDetail,
        api_openapi::models::JobWithDetails,
        api_openapi::models::JobsQuery,
        api_openapi::models::Competency,
        api_openapi::models::EvidenceItem,
        api_openapi::models::CompetencyWithEvidence,
        api_openapi::models::AboutSectionResponse,
        api_openapi::models::SocialLink,
        api_openapi::models::Challenge,
        api_openapi::models::ChallengeResponse,
        api_openapi::models::ContactSubmitRequest,
        api_openapi::models::ContactResponse,
        api_openapi::models::AuthMe,
        api_openapi::models::AskRequest,
        api_openapi::models::AskCitation,
        api_openapi::models::AskResponse,
        api_openapi::models::ResumeData,
        // Admin input types
        api_openapi::models::JobInput,
        api_openapi::models::JobDetailInput,
        api_openapi::models::CompetencyInput,
        api_openapi::models::EvidenceInput,
        api_openapi::models::Evidence,
        api_openapi::models::AboutSectionInput,
        api_openapi::models::SocialLinkInput,
        api_openapi::models::SocialLinkResponse,
        api_openapi::models::ChallengeInput,
        // W-RST: tailor pipeline (reserved — routes not yet implemented)
        api_openapi::models::TailorRequest,
        api_openapi::models::TailorResponse,
        api_openapi::models::MatchedBullet,
    )),
    modifiers(&SecurityAddon, &ApiVersionModifier),
    tags(
        (name = "health", description = "Service health checks"),
        (name = "crates", description = "deploy-baba crate information"),
        (name = "stack", description = "Stack configuration examples"),
        (name = "portfolio", description = "Resume, jobs, competencies, about, and social links"),
        (name = "contact", description = "Contact form and PoW challenge"),
        (name = "auth", description = "Authentication and session"),
        (name = "ask", description = "RAG Q&A over the deploy-baba codebase"),
        (name = "admin", description = "Protected admin CRUD (requires auth)"),
        (name = "tailor", description = "JD-driven resume tailoring — docx/pdf download (requires auth, W-RST)"),
    ),
)]
pub struct ApiDoc;
