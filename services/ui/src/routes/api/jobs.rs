use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::db::Db;
use crate::state::AppState;

pub use api_openapi::models::{Job, JobDetail, JobWithDetails, JobsQuery};

#[utoipa::path(
    get,
    path = "/api/jobs",
    tag = "resume",
    params(JobsQuery),
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
        .route(
            "/:slug/challenges",
            get(super::challenges::list_challenges_for_job),
        )
}
