use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

struct SecurityAddon;

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
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Service health checks"),
        (name = "crates", description = "deploy-baba crate information"),
        (name = "stack", description = "Stack configuration examples"),
        (name = "demo", description = "Live API demonstrations"),
        (name = "resume", description = "Career timeline and competency data"),
        (name = "admin", description = "Protected admin CRUD (requires auth)"),
    ),
)]
pub struct ApiDoc;
