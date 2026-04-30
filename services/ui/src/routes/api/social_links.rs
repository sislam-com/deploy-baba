use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;

use crate::db::{load_social_links, Db};
use crate::state::AppState;

pub use api_openapi::models::SocialLink;

#[utoipa::path(
    get,
    path = "/api/social-links",
    tag = "portfolio",
    responses(
        (status = 200, description = "All visible social links ordered by sort_order", body = Vec<SocialLink>)
    )
)]
pub async fn list_social_links(
    State(db): State<Arc<Db>>,
) -> Json<Vec<SocialLink>> {
    let conn = db.conn.lock().unwrap();
    Json(load_social_links(&conn))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_social_links))
}
