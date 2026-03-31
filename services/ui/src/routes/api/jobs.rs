use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::Db;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct Job {
    pub id: i64,
    pub slug: String,
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub summary: String,
    pub tech_stack: Option<Vec<String>>,
    pub sort_order: i64,
}

#[derive(Serialize, ToSchema)]
pub struct JobDetail {
    pub id: i64,
    pub detail_text: String,
    pub category: Option<String>,
    pub sort_order: i64,
}

#[derive(Serialize, ToSchema)]
pub struct JobWithDetails {
    #[serde(flatten)]
    pub job: Job,
    pub details: Vec<JobDetail>,
}

#[derive(Deserialize)]
pub struct JobsQuery {
    #[allow(dead_code)]
    pub view: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/jobs",
    tag = "resume",
    params(
        ("view" = Option<String>, Query, description = "View mode: 'chronological' (default)")
    ),
    responses(
        (status = 200, description = "List of all job positions", body = Vec<Job>)
    )
)]
pub async fn list_jobs(
    State(db): State<Arc<Db>>,
    Query(_query): Query<JobsQuery>,
) -> Result<Json<Vec<Job>>, (axum::http::StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order
             FROM jobs ORDER BY sort_order ASC",
        )
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let jobs = stmt
        .query_map([], |row| {
            let tech_raw: Option<String> = row.get(8)?;
            Ok(Job {
                id: row.get(0)?,
                slug: row.get(1)?,
                company: row.get(2)?,
                title: row.get(3)?,
                location: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                summary: row.get(7)?,
                tech_stack: tech_raw.map(|s| s.split(',').map(|t| t.trim().to_string()).collect()),
                sort_order: row.get(9)?,
            })
        })
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(jobs))
}

#[utoipa::path(
    get,
    path = "/api/jobs/{slug}",
    tag = "resume",
    params(
        ("slug" = String, Path, description = "Job slug, e.g. 'scala-computing'")
    ),
    responses(
        (status = 200, description = "Job detail with accomplishment bullets", body = JobWithDetails),
        (status = 404, description = "Job not found")
    )
)]
pub async fn get_job(
    State(db): State<Arc<Db>>,
    Path(slug): Path<String>,
) -> Result<Json<JobWithDetails>, (axum::http::StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let job = conn
        .query_row(
            "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order
             FROM jobs WHERE slug = ?1",
            rusqlite::params![slug],
            |row| {
                let tech_raw: Option<String> = row.get(8)?;
                Ok(Job {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    company: row.get(2)?,
                    title: row.get(3)?,
                    location: row.get(4)?,
                    start_date: row.get(5)?,
                    end_date: row.get(6)?,
                    summary: row.get(7)?,
                    tech_stack: tech_raw
                        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect()),
                    sort_order: row.get(9)?,
                })
            },
        )
        .map_err(|_| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Job '{}' not found", slug),
            )
        })?;

    let mut stmt = conn
        .prepare(
            "SELECT id, detail_text, category, sort_order
             FROM job_details WHERE job_id = ?1 ORDER BY sort_order ASC",
        )
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let details = stmt
        .query_map(rusqlite::params![job.id], |row| {
            Ok(JobDetail {
                id: row.get(0)?,
                detail_text: row.get(1)?,
                category: row.get(2)?,
                sort_order: row.get(3)?,
            })
        })
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(JobWithDetails { job, details }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs))
        .route("/:slug", get(get_job))
}
