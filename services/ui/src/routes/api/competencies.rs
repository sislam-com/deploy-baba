use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::Db;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct Competency {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub sort_order: i64,
}

#[derive(Serialize, ToSchema)]
pub struct EvidenceItem {
    pub id: i64,
    pub job_id: i64,
    pub job_slug: String,
    pub company: String,
    pub detail_id: Option<i64>,
    pub highlight_text: Option<String>,
    pub detail_text: Option<String>,
    pub sort_order: i64,
}

#[derive(Serialize, ToSchema)]
pub struct CompetencyWithEvidence {
    #[serde(flatten)]
    pub competency: Competency,
    pub evidence: Vec<EvidenceItem>,
}

#[utoipa::path(
    get,
    path = "/api/competencies",
    tag = "resume",
    responses(
        (status = 200, description = "List of all competency categories", body = Vec<Competency>)
    )
)]
pub async fn list_competencies(
    State(db): State<Arc<Db>>,
) -> Result<Json<Vec<Competency>>, (axum::http::StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, name, description, icon, sort_order
             FROM competencies ORDER BY sort_order ASC",
        )
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let competencies = stmt
        .query_map([], |row| {
            Ok(Competency {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                sort_order: row.get(5)?,
            })
        })
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(competencies))
}

#[utoipa::path(
    get,
    path = "/api/competencies/{slug}",
    tag = "resume",
    params(
        ("slug" = String, Path, description = "Competency slug, e.g. 'cloud-infrastructure'")
    ),
    responses(
        (status = 200, description = "Competency detail with cross-referenced evidence", body = CompetencyWithEvidence),
        (status = 404, description = "Competency not found")
    )
)]
pub async fn get_competency(
    State(db): State<Arc<Db>>,
    Path(slug): Path<String>,
) -> Result<Json<CompetencyWithEvidence>, (axum::http::StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let competency = conn
        .query_row(
            "SELECT id, slug, name, description, icon, sort_order FROM competencies WHERE slug = ?1",
            rusqlite::params![slug],
            |row| {
                Ok(Competency {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    icon: row.get(4)?,
                    sort_order: row.get(5)?,
                })
            },
        )
        .map_err(|_| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Competency '{}' not found", slug),
            )
        })?;

    let mut stmt = conn
        .prepare(
            "SELECT ce.id, ce.job_id, j.slug, j.company, ce.detail_id,
                    ce.highlight_text, jd.detail_text, ce.sort_order
             FROM competency_evidence ce
             JOIN jobs j ON j.id = ce.job_id
             LEFT JOIN job_details jd ON jd.id = ce.detail_id
             WHERE ce.competency_id = ?1
             ORDER BY ce.sort_order ASC",
        )
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let evidence = stmt
        .query_map(rusqlite::params![competency.id], |row| {
            Ok(EvidenceItem {
                id: row.get(0)?,
                job_id: row.get(1)?,
                job_slug: row.get(2)?,
                company: row.get(3)?,
                detail_id: row.get(4)?,
                highlight_text: row.get(5)?,
                detail_text: row.get(6)?,
                sort_order: row.get(7)?,
            })
        })
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(CompetencyWithEvidence {
        competency,
        evidence,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_competencies))
        .route("/:slug", get(get_competency))
}
