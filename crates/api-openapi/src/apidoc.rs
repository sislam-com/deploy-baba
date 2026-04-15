/// Public and admin OpenAPI marker types.
///
/// `PublicApiDoc` carries the schemas for all publicly accessible API responses
/// and request bodies. `AdminApiDoc` carries schemas for admin-only types plus
/// the security scheme modifier.
///
/// `full_spec()` merges both into a single `utoipa::openapi::OpenApi` that is
/// used as the base when constructing the `utoipa_axum::router::OpenApiRouter`
/// in `services/ui`. Paths are NOT declared here — they are collected
/// automatically by `utoipa-axum` via `routes!()` macros in the router build
/// function, so there is no hand-maintained `paths(...)` list anywhere.
use crate::models::{
    AboutSectionInput, AboutSectionResponse, ApiError, ChallengeResponse, Competency,
    CompetencyInput, CompetencyWithEvidence, ContactResponse, ContactSubmitRequest, CrateInfo,
    Evidence, EvidenceInput, EvidenceItem, Field, GenerateSpecRequest, GenerateSpecResponse,
    HealthResponse, Job, JobDetail, JobDetailInput, JobInput, JobWithDetails, JobsQuery,
    MatchedBullet, ParseConfigRequest, ParseConfigResponse, SocialLink, SocialLinkInput,
    SocialLinkResponse, TailorRequest, TailorResponse,
};
use utoipa::OpenApi;

/// Public-facing schema registry.
///
/// Contains every type that can appear in a response body or request body of an
/// unauthenticated endpoint. Security schemes are deliberately absent — consumers
/// of the filtered spec should not discover that admin endpoints exist at all.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "deploy-baba Portfolio & API",
        version = "0.1.0",
        description = "Live demos and documentation for the deploy-baba ecosystem"
    ),
    components(schemas(
        ApiError,
        HealthResponse,
        CrateInfo,
        ParseConfigRequest,
        ParseConfigResponse,
        Field,
        GenerateSpecRequest,
        GenerateSpecResponse,
        Job,
        JobDetail,
        JobWithDetails,
        JobsQuery,
        Competency,
        EvidenceItem,
        CompetencyWithEvidence,
        AboutSectionResponse,
        SocialLink,
        ChallengeResponse,
        ContactSubmitRequest,
        ContactResponse,
    )),
    tags(
        (name = "health", description = "Service health checks"),
        (name = "crates", description = "deploy-baba crate information"),
        (name = "stack", description = "Stack configuration examples"),
        (name = "demo", description = "Live API demonstrations"),
        (name = "resume", description = "Career timeline and competency data"),
        (name = "contact", description = "Contact form and PoW challenge"),
        (name = "about", description = "About page content"),
        (name = "social", description = "Social links"),
    ),
)]
pub struct PublicApiDoc;

/// Admin schema registry + security modifier.
///
/// Contains the request body types used exclusively by authenticated admin
/// endpoints. Also attaches `cookieAuth` and `bearerAuth` security schemes so
/// they appear in the full spec (but NOT the public-filtered spec).
#[derive(OpenApi)]
#[openapi(
    components(schemas(
        AboutSectionInput,
        SocialLinkInput,
        SocialLinkResponse,
        JobInput,
        JobDetailInput,
        CompetencyInput,
        EvidenceInput,
        Evidence,
        TailorRequest,
        MatchedBullet,
        TailorResponse,
    )),
    tags(
        (name = "admin", description = "Protected admin CRUD (requires auth)"),
        (name = "tailor", description = "JD-driven resume tailoring (admin only)"),
    ),
    modifiers(&SecurityAddon),
)]
pub struct AdminApiDoc;

/// Attaches `cookieAuth` and `bearerAuth` security schemes to `AdminApiDoc`.
pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{
            ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme,
        };
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

/// Merge `PublicApiDoc` and `AdminApiDoc` into a single specification.
///
/// Used as the base for `utoipa_axum::router::OpenApiRouter::with_openapi(...)`.
/// Paths are then collected automatically via `routes!()` macros; this function
/// only provides the info block, tag definitions, component schemas, and security
/// schemes.
pub fn full_spec() -> utoipa::openapi::OpenApi {
    let mut public = PublicApiDoc::openapi();
    let admin = AdminApiDoc::openapi();

    // Merge admin tags
    if let Some(admin_tags) = admin.tags {
        let pub_tags = public.tags.get_or_insert_with(Vec::new);
        for tag in admin_tags {
            if !pub_tags.iter().any(|t| t.name == tag.name) {
                pub_tags.push(tag);
            }
        }
    }

    // Merge admin component schemas + security schemes
    if let Some(admin_components) = admin.components {
        let pub_components = public.components.get_or_insert_with(Default::default);
        for (name, schema) in admin_components.schemas {
            pub_components.schemas.entry(name).or_insert(schema);
        }
        for (name, scheme) in admin_components.security_schemes {
            pub_components
                .security_schemes
                .entry(name)
                .or_insert(scheme);
        }
    }

    public
}
