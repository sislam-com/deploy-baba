pub mod admin;
pub mod competencies;
pub mod crates;
pub mod demo;
pub mod jobs;
pub mod stack;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/crates", crates::router())
        .nest("/stack", stack::router())
        .nest("/demo", demo::router())
        .nest("/jobs", jobs::router())
        .nest("/competencies", competencies::router())
}
