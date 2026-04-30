use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::IntoParams;

use crate::db::Db;
use crate::state::AppState;

pub use api_openapi::models::AboutSectionResponse;

#[derive(Debug, Deserialize, IntoParams)]
pub struct AboutQuery {
    /// Filter by page: `"me"` or `"repo"`. Returns all sections if omitted.
    pub page: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/about/sections",
    tag = "portfolio",
    params(AboutQuery),
    responses(
        (status = 200, description = "About sections, optionally filtered by page", body = Vec<AboutSectionResponse>)
    )
)]
pub async fn list_about_sections(
    State(db): State<Arc<Db>>,
    Query(query): Query<AboutQuery>,
) -> Result<Json<Vec<AboutSectionResponse>>, (StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let (sql, has_filter) = if query.page.is_some() {
        (
            "SELECT id, page, slug, heading, body, icon, sort_order \
             FROM about_sections WHERE page = ?1 ORDER BY sort_order ASC",
            true,
        )
    } else {
        (
            "SELECT id, page, slug, heading, body, icon, sort_order \
             FROM about_sections ORDER BY sort_order ASC",
            false,
        )
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let sections: Vec<AboutSectionResponse> = if has_filter {
        let page = query.page.as_deref().unwrap_or("");
        stmt.query_map(rusqlite::params![page], |row| {
            Ok(AboutSectionResponse {
                id: row.get(0)?,
                page: row.get(1)?,
                slug: row.get(2)?,
                heading: row.get(3)?,
                body: row.get(4)?,
                icon: row.get(5)?,
                sort_order: row.get(6)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        stmt.query_map([], |row| {
            Ok(AboutSectionResponse {
                id: row.get(0)?,
                page: row.get(1)?,
                slug: row.get(2)?,
                heading: row.get(3)?,
                body: row.get(4)?,
                icon: row.get(5)?,
                sort_order: row.get(6)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect()
    };

    Ok(Json(sections))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/sections", get(list_about_sections))
}
