pub mod about;
pub mod admin;
pub mod ask;
pub mod auth_me;
pub mod challenges;
pub mod competencies;
pub mod crates;
pub mod demo;
pub mod jobs;
pub mod legal;
pub mod linkedin;
pub mod metrics;
pub mod rag;
pub mod resume_data;
pub mod social_links;
pub mod stack;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/crates", crates::router())
        .nest("/stack", stack::router())
        .nest("/demo", demo::router())
        .nest("/jobs", jobs::router())
        .nest("/challenges", challenges::router())
        .nest("/competencies", competencies::router())
        .nest("/about", about::router())
        .nest("/legal", legal::router())
        .nest("/social-links", social_links::router())
        .nest("/resume", resume_data::router())
        .nest("/auth", auth_me::router())
        .nest("/v1/rag", rag::router())
        .merge(ask::router())
}
