/// OpenAPI spec — single source of truth for schema definitions is
/// `api_openapi::models`; this file wires the handler paths into the spec.
///
/// NOTE: utoipa-axum (which would eliminate the `paths(...)` list) requires
/// utoipa v5 but this workspace uses v4. Migration is tracked as W-APIO.4.4 and
/// deferred until the workspace upgrades to utoipa v5. For now we keep the
/// hand-maintained list; all *schema types* come from `api_openapi::models`.
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

/// Full combined spec (public + admin schemas, all handler paths, security schemes).
///
/// `router.rs` builds two variants:
///  * `/api/openapi.json`        — filtered public view (via `api_openapi::filter::public_view`)
///  * `/api/openapi-admin.json`  — this full spec, auth-gated
#[derive(OpenApi)]
#[openapi(
    info(
        title = "deploy-baba Portfolio & API",
        version = "0.1.0",
        description = "Live demos and documentation for the deploy-baba ecosystem"
    ),
    paths(
        crate::routes::health::get_health,
        crate::routes::api::crates::list_crates,
        crate::routes::api::crates::get_crate,
        crate::routes::api::stack::get_stack,
        crate::routes::api::demo::parse_config,
        crate::routes::api::demo::generate_spec,
        crate::routes::api::jobs::list_jobs,
        crate::routes::api::jobs::get_job,
        crate::routes::api::competencies::list_competencies,
        crate::routes::api::competencies::get_competency,
        crate::routes::api::admin::create_job,
        crate::routes::api::admin::update_job,
        crate::routes::api::admin::delete_job,
        crate::routes::api::admin::create_job_detail,
        crate::routes::api::admin::update_job_detail,
        crate::routes::api::admin::delete_job_detail,
        crate::routes::api::admin::create_competency,
        crate::routes::api::admin::update_competency,
        crate::routes::api::admin::delete_competency,
        crate::routes::api::admin::create_evidence,
        crate::routes::api::admin::update_evidence,
        crate::routes::api::admin::delete_evidence,
        crate::routes::api::admin::create_about_section,
        crate::routes::api::admin::update_about_section,
        crate::routes::api::admin::delete_about_section,
    ),
    components(schemas(
        // All schemas from api_openapi::models — SSOT enforced by ApiModel trait
        api_openapi::models::ApiError,
        api_openapi::models::HealthResponse,
        api_openapi::models::CrateInfo,
        api_openapi::models::ParseConfigRequest,
        api_openapi::models::ParseConfigResponse,
        api_openapi::models::Field,
        api_openapi::models::GenerateSpecRequest,
        api_openapi::models::GenerateSpecResponse,
        api_openapi::models::Job,
        api_openapi::models::JobDetail,
        api_openapi::models::JobWithDetails,
        api_openapi::models::JobsQuery,
        api_openapi::models::Competency,
        api_openapi::models::EvidenceItem,
        api_openapi::models::CompetencyWithEvidence,
        api_openapi::models::AboutSectionInput,
        api_openapi::models::AboutSectionResponse,
        api_openapi::models::SocialLink,
        api_openapi::models::SocialLinkInput,
        api_openapi::models::SocialLinkResponse,
        api_openapi::models::ChallengeResponse,
        api_openapi::models::ContactSubmitRequest,
        api_openapi::models::ContactResponse,
        api_openapi::models::JobInput,
        api_openapi::models::JobDetailInput,
        api_openapi::models::CompetencyInput,
        api_openapi::models::EvidenceInput,
        api_openapi::models::Evidence,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Service health checks"),
        (name = "crates", description = "deploy-baba crate information"),
        (name = "stack", description = "Stack configuration examples"),
        (name = "demo", description = "Live API demonstrations"),
        (name = "resume", description = "Career timeline and competency data"),
        (name = "contact", description = "Contact form and PoW challenge"),
        (name = "about", description = "About page content"),
        (name = "social", description = "Social links"),
        (name = "admin", description = "Protected admin CRUD (requires auth)"),
    ),
)]
pub struct ApiDoc;
