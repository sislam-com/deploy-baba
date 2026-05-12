use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use std::sync::Arc;

use crate::db::{load_social_links, Db};
use crate::state::AppState;

use crate::routes::api::challenges::row_to_challenge;
pub use api_openapi::models::{Challenge, Competency, Job, ResumeData};

type ApiError = (StatusCode, String);

fn db_err(e: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[utoipa::path(
    get,
    path = "/api/resume",
    tag = "portfolio",
    responses(
        (status = 200, description = "Combined resume: bio, jobs, competencies, social links", body = ResumeData),
        (status = 500, description = "Database error")
    )
)]
pub async fn get_resume_data(State(db): State<Arc<Db>>) -> Result<Json<ResumeData>, ApiError> {
    let conn = db.conn.lock().unwrap();

    let bio = conn
        .query_row(
            "SELECT body FROM about_sections WHERE slug = 'me-bio'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default();

    let summary_text = conn
        .query_row(
            "SELECT body FROM about_sections WHERE slug = 'me-summary'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| bio.clone());

    // Use summary as title, with hardcoded name
    let name = "Sharful Islam".to_string();
    let title = if summary_text.is_empty() {
        "AI Systems Engineer".to_string()
    } else {
        summary_text
    };
    let summary = String::new();

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order \
             FROM jobs ORDER BY sort_order ASC",
        )
        .map_err(db_err)?;

    let jobs: Vec<Job> = stmt
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
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt2 = conn
        .prepare(
            "SELECT id, slug, name, description, icon, sort_order \
             FROM competencies ORDER BY sort_order ASC",
        )
        .map_err(db_err)?;

    let competencies: Vec<Competency> = stmt2
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
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    let social_links = load_social_links(&conn);

    let mut stmt3 = conn
        .prepare(
            "SELECT id, slug, title, job_id, description, short_description, tech_stack, \
             category, url, image_url, featured, sort_order \
             FROM challenges WHERE featured = 1 ORDER BY sort_order ASC",
        )
        .map_err(db_err)?;

    let challenges: Vec<Challenge> = stmt3
        .query_map([], row_to_challenge)
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(ResumeData {
        name,
        title,
        bio,
        summary,
        jobs,
        competencies,
        social_links,
        challenges,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_resume_data))
}
