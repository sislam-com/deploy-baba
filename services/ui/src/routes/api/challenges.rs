use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::db::Db;
use crate::state::AppState;

pub use api_openapi::models::Challenge;

pub fn row_to_challenge(row: &rusqlite::Row<'_>) -> rusqlite::Result<Challenge> {
    let tech_raw: Option<String> = row.get(6)?;
    let featured_int: i64 = row.get(10)?;
    Ok(Challenge {
        id: row.get(0)?,
        slug: row.get(1)?,
        title: row.get(2)?,
        job_id: row.get(3)?,
        description: row.get(4)?,
        short_description: row.get(5)?,
        tech_stack: tech_raw.map(|s| s.split(',').map(|t| t.trim().to_string()).collect()),
        category: row.get(7)?,
        url: row.get(8)?,
        image_url: row.get(9)?,
        featured: featured_int != 0,
        sort_order: row.get(11)?,
    })
}

const SELECT_COLS: &str =
    "id, slug, title, job_id, description, short_description, tech_stack, category, url, image_url, featured, sort_order";

#[utoipa::path(
    get,
    path = "/api/challenges",
    tag = "portfolio",
    responses(
        (status = 200, description = "List of all challenges/projects", body = Vec<Challenge>)
    )
)]
pub async fn list_challenges(
    State(db): State<Arc<Db>>,
) -> Result<Json<Vec<Challenge>>, (axum::http::StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let query = format!(
        "SELECT {} FROM challenges ORDER BY sort_order ASC",
        SELECT_COLS
    );
    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let challenges = stmt
        .query_map([], row_to_challenge)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(challenges))
}

#[utoipa::path(
    get,
    path = "/api/challenges/{slug}",
    tag = "portfolio",
    params(
        ("slug" = String, Path, description = "Challenge slug, e.g. 'deploy-baba-portfolio'")
    ),
    responses(
        (status = 200, description = "Challenge detail", body = Challenge),
        (status = 404, description = "Challenge not found")
    )
)]
pub async fn get_challenge(
    State(db): State<Arc<Db>>,
    Path(slug): Path<String>,
) -> Result<Json<Challenge>, (axum::http::StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let query = format!("SELECT {} FROM challenges WHERE slug = ?1", SELECT_COLS);
    let challenge = conn
        .query_row(&query, rusqlite::params![slug], row_to_challenge)
        .map_err(|_| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Challenge '{}' not found", slug),
            )
        })?;

    Ok(Json(challenge))
}

#[utoipa::path(
    get,
    path = "/api/jobs/{slug}/challenges",
    tag = "portfolio",
    params(
        ("slug" = String, Path, description = "Job slug, e.g. 'scala-computing'")
    ),
    responses(
        (status = 200, description = "Challenges linked to this job", body = Vec<Challenge>),
        (status = 404, description = "Job not found")
    )
)]
pub async fn list_challenges_for_job(
    State(db): State<Arc<Db>>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<Challenge>>, (axum::http::StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let job_id: i64 = conn
        .query_row(
            "SELECT id FROM jobs WHERE slug = ?1",
            rusqlite::params![slug],
            |row| row.get(0),
        )
        .map_err(|_| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Job '{}' not found", slug),
            )
        })?;

    let query = format!(
        "SELECT {} FROM challenges WHERE job_id = ?1 ORDER BY sort_order ASC",
        SELECT_COLS
    );
    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let challenges = stmt
        .query_map(rusqlite::params![job_id], row_to_challenge)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(challenges))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_challenges))
        .route("/:slug", get(get_challenge))
}
