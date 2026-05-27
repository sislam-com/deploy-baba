use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::db::Db;
use crate::state::AppState;

pub use api_openapi::models::LegalDocumentResponse;

#[utoipa::path(
    get,
    path = "/api/legal/{slug}",
    tag = "portfolio",
    params(
        ("slug" = String, Path, description = "Document slug, e.g. 'terms' or 'privacy'")
    ),
    responses(
        (status = 200, description = "Legal document found", body = LegalDocumentResponse),
        (status = 404, description = "Legal document not found")
    )
)]
pub async fn get_legal_document(
    State(db): State<Arc<Db>>,
    Path(slug): Path<String>,
) -> Result<Json<LegalDocumentResponse>, (StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let doc = conn
        .query_row(
            "SELECT id, slug, title, content, updated_at
             FROM legal_documents WHERE slug = ?1",
            rusqlite::params![slug],
            |row| {
                Ok(LegalDocumentResponse {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                format!("Legal document '{}' not found", slug),
            )
        })?;

    Ok(Json(doc))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/:slug", get(get_legal_document))
}
