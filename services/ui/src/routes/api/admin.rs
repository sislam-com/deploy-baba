use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{post, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::Db;
use crate::routes::api::competencies::Competency;
use crate::routes::api::jobs::{Job, JobDetail};
use crate::state::AppState;

type ApiResult<T> = Result<T, (StatusCode, String)>;

fn db_err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn not_found(msg: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, msg.into())
}

// ── Input types ──────────────────────────────────────────────────────────────

#[derive(Deserialize, ToSchema)]
pub struct JobInput {
    pub slug: String,
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub summary: String,
    /// Comma-separated list, matches DB storage format.
    pub tech_stack: Option<String>,
    pub sort_order: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct JobDetailInput {
    pub detail_text: String,
    pub category: Option<String>,
    pub sort_order: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CompetencyInput {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub sort_order: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct EvidenceInput {
    pub competency_id: i64,
    pub job_id: i64,
    pub detail_id: Option<i64>,
    pub highlight_text: Option<String>,
    pub sort_order: i64,
}

// ── Job handlers ─────────────────────────────────────────────────────────────

/// Create a new job entry.
#[utoipa::path(
    post,
    path = "/api/admin/jobs",
    tag = "admin",
    request_body = JobInput,
    responses(
        (status = 201, description = "Job created", body = Job),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn create_job(
    State(db): State<Arc<Db>>,
    Json(input): Json<JobInput>,
) -> ApiResult<(StatusCode, Json<Job>)> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            input.slug, input.company, input.title, input.location,
            input.start_date, input.end_date, input.summary,
            input.tech_stack, input.sort_order,
        ],
    ).map_err(db_err)?;

    let id = conn.last_insert_rowid();
    let job = conn.query_row(
        "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order
         FROM jobs WHERE id = ?1",
        rusqlite::params![id],
        row_to_job,
    ).map_err(db_err)?;

    Ok((StatusCode::CREATED, Json(job)))
}

/// Update an existing job entry.
#[utoipa::path(
    put,
    path = "/api/admin/jobs/{id}",
    tag = "admin",
    params(("id" = i64, Path, description = "Job ID")),
    request_body = JobInput,
    responses(
        (status = 200, description = "Job updated", body = Job),
        (status = 404, description = "Job not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn update_job(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(input): Json<JobInput>,
) -> ApiResult<Json<Job>> {
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "UPDATE jobs SET slug=?1, company=?2, title=?3, location=?4, start_date=?5,
         end_date=?6, summary=?7, tech_stack=?8, sort_order=?9 WHERE id=?10",
            rusqlite::params![
                input.slug,
                input.company,
                input.title,
                input.location,
                input.start_date,
                input.end_date,
                input.summary,
                input.tech_stack,
                input.sort_order,
                id,
            ],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Job {} not found", id)));
    }

    let job = conn.query_row(
        "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order
         FROM jobs WHERE id = ?1",
        rusqlite::params![id],
        row_to_job,
    ).map_err(db_err)?;

    Ok(Json(job))
}

/// Delete a job entry.
#[utoipa::path(
    delete,
    path = "/api/admin/jobs/{id}",
    tag = "admin",
    params(("id" = i64, Path, description = "Job ID")),
    responses(
        (status = 204, description = "Job deleted"),
        (status = 404, description = "Job not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn delete_job(State(db): State<Arc<Db>>, Path(id): Path<i64>) -> ApiResult<StatusCode> {
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute("DELETE FROM jobs WHERE id = ?1", rusqlite::params![id])
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Job {} not found", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── JobDetail handlers ────────────────────────────────────────────────────────

/// Create a new job detail entry.
#[utoipa::path(
    post,
    path = "/api/admin/jobs/{job_id}/details",
    tag = "admin",
    params(("job_id" = i64, Path, description = "Job ID")),
    request_body = JobDetailInput,
    responses(
        (status = 201, description = "Job detail created", body = JobDetail),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn create_job_detail(
    State(db): State<Arc<Db>>,
    Path(job_id): Path<i64>,
    Json(input): Json<JobDetailInput>,
) -> ApiResult<(StatusCode, Json<JobDetail>)> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO job_details (job_id, detail_text, category, sort_order) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![job_id, input.detail_text, input.category, input.sort_order],
    ).map_err(db_err)?;

    let id = conn.last_insert_rowid();
    let detail = conn
        .query_row(
            "SELECT id, detail_text, category, sort_order FROM job_details WHERE id = ?1",
            rusqlite::params![id],
            row_to_detail,
        )
        .map_err(db_err)?;

    Ok((StatusCode::CREATED, Json(detail)))
}

/// Update a job detail entry.
#[utoipa::path(
    put,
    path = "/api/admin/jobs/{job_id}/details/{id}",
    tag = "admin",
    params(
        ("job_id" = i64, Path, description = "Job ID"),
        ("id" = i64, Path, description = "Job detail ID"),
    ),
    request_body = JobDetailInput,
    responses(
        (status = 200, description = "Job detail updated", body = JobDetail),
        (status = 404, description = "Job detail not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn update_job_detail(
    State(db): State<Arc<Db>>,
    Path(params): Path<(i64, i64)>,
    Json(input): Json<JobDetailInput>,
) -> ApiResult<Json<JobDetail>> {
    let (_, id) = params;
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "UPDATE job_details SET detail_text=?1, category=?2, sort_order=?3 WHERE id=?4",
            rusqlite::params![input.detail_text, input.category, input.sort_order, id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("JobDetail {} not found", id)));
    }

    let detail = conn
        .query_row(
            "SELECT id, detail_text, category, sort_order FROM job_details WHERE id = ?1",
            rusqlite::params![id],
            row_to_detail,
        )
        .map_err(db_err)?;

    Ok(Json(detail))
}

/// Delete a job detail entry.
#[utoipa::path(
    delete,
    path = "/api/admin/jobs/{job_id}/details/{id}",
    tag = "admin",
    params(
        ("job_id" = i64, Path, description = "Job ID"),
        ("id" = i64, Path, description = "Job detail ID"),
    ),
    responses(
        (status = 204, description = "Job detail deleted"),
        (status = 404, description = "Job detail not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn delete_job_detail(
    State(db): State<Arc<Db>>,
    Path(params): Path<(i64, i64)>,
) -> ApiResult<StatusCode> {
    let (_, id) = params;
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "DELETE FROM job_details WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("JobDetail {} not found", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── Competency handlers ───────────────────────────────────────────────────────

/// Create a new competency.
#[utoipa::path(
    post,
    path = "/api/admin/competencies",
    tag = "admin",
    request_body = CompetencyInput,
    responses(
        (status = 201, description = "Competency created", body = Competency),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn create_competency(
    State(db): State<Arc<Db>>,
    Json(input): Json<CompetencyInput>,
) -> ApiResult<(StatusCode, Json<Competency>)> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO competencies (slug, name, description, icon, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![input.slug, input.name, input.description, input.icon, input.sort_order],
    ).map_err(db_err)?;

    let id = conn.last_insert_rowid();
    let comp = conn
        .query_row(
            "SELECT id, slug, name, description, icon, sort_order FROM competencies WHERE id = ?1",
            rusqlite::params![id],
            row_to_competency,
        )
        .map_err(db_err)?;

    Ok((StatusCode::CREATED, Json(comp)))
}

/// Update an existing competency.
#[utoipa::path(
    put,
    path = "/api/admin/competencies/{id}",
    tag = "admin",
    params(("id" = i64, Path, description = "Competency ID")),
    request_body = CompetencyInput,
    responses(
        (status = 200, description = "Competency updated", body = Competency),
        (status = 404, description = "Competency not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn update_competency(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(input): Json<CompetencyInput>,
) -> ApiResult<Json<Competency>> {
    let conn = db.conn.lock().unwrap();
    let rows = conn.execute(
        "UPDATE competencies SET slug=?1, name=?2, description=?3, icon=?4, sort_order=?5 WHERE id=?6",
        rusqlite::params![input.slug, input.name, input.description, input.icon, input.sort_order, id],
    ).map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Competency {} not found", id)));
    }

    let comp = conn
        .query_row(
            "SELECT id, slug, name, description, icon, sort_order FROM competencies WHERE id = ?1",
            rusqlite::params![id],
            row_to_competency,
        )
        .map_err(db_err)?;

    Ok(Json(comp))
}

/// Delete a competency.
#[utoipa::path(
    delete,
    path = "/api/admin/competencies/{id}",
    tag = "admin",
    params(("id" = i64, Path, description = "Competency ID")),
    responses(
        (status = 204, description = "Competency deleted"),
        (status = 404, description = "Competency not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn delete_competency(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "DELETE FROM competencies WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Competency {} not found", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── Evidence handlers ─────────────────────────────────────────────────────────

#[derive(serde::Serialize, ToSchema)]
pub struct Evidence {
    pub id: i64,
    pub competency_id: i64,
    pub job_id: i64,
    pub detail_id: Option<i64>,
    pub highlight_text: Option<String>,
    pub sort_order: i64,
}

/// Create a new competency evidence link.
#[utoipa::path(
    post,
    path = "/api/admin/evidence",
    tag = "admin",
    request_body = EvidenceInput,
    responses(
        (status = 201, description = "Evidence created", body = Evidence),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn create_evidence(
    State(db): State<Arc<Db>>,
    Json(input): Json<EvidenceInput>,
) -> ApiResult<(StatusCode, Json<Evidence>)> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO competency_evidence (competency_id, job_id, detail_id, highlight_text, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            input.competency_id, input.job_id, input.detail_id,
            input.highlight_text, input.sort_order,
        ],
    ).map_err(db_err)?;

    let id = conn.last_insert_rowid();
    let ev = conn
        .query_row(
            "SELECT id, competency_id, job_id, detail_id, highlight_text, sort_order
         FROM competency_evidence WHERE id = ?1",
            rusqlite::params![id],
            row_to_evidence,
        )
        .map_err(db_err)?;

    Ok((StatusCode::CREATED, Json(ev)))
}

/// Update a competency evidence link.
#[utoipa::path(
    put,
    path = "/api/admin/evidence/{id}",
    tag = "admin",
    params(("id" = i64, Path, description = "Evidence ID")),
    request_body = EvidenceInput,
    responses(
        (status = 200, description = "Evidence updated", body = Evidence),
        (status = 404, description = "Evidence not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn update_evidence(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(input): Json<EvidenceInput>,
) -> ApiResult<Json<Evidence>> {
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "UPDATE competency_evidence SET competency_id=?1, job_id=?2, detail_id=?3,
         highlight_text=?4, sort_order=?5 WHERE id=?6",
            rusqlite::params![
                input.competency_id,
                input.job_id,
                input.detail_id,
                input.highlight_text,
                input.sort_order,
                id,
            ],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Evidence {} not found", id)));
    }

    let ev = conn
        .query_row(
            "SELECT id, competency_id, job_id, detail_id, highlight_text, sort_order
         FROM competency_evidence WHERE id = ?1",
            rusqlite::params![id],
            row_to_evidence,
        )
        .map_err(db_err)?;

    Ok(Json(ev))
}

/// Delete a competency evidence link.
#[utoipa::path(
    delete,
    path = "/api/admin/evidence/{id}",
    tag = "admin",
    params(("id" = i64, Path, description = "Evidence ID")),
    responses(
        (status = 204, description = "Evidence deleted"),
        (status = 404, description = "Evidence not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn delete_evidence(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "DELETE FROM competency_evidence WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Evidence {} not found", id)));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ── Row mappers ───────────────────────────────────────────────────────────────

fn row_to_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<Job> {
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
}

fn row_to_detail(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobDetail> {
    Ok(JobDetail {
        id: row.get(0)?,
        detail_text: row.get(1)?,
        category: row.get(2)?,
        sort_order: row.get(3)?,
    })
}

fn row_to_competency(row: &rusqlite::Row<'_>) -> rusqlite::Result<Competency> {
    Ok(Competency {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        icon: row.get(4)?,
        sort_order: row.get(5)?,
    })
}

fn row_to_evidence(row: &rusqlite::Row<'_>) -> rusqlite::Result<Evidence> {
    Ok(Evidence {
        id: row.get(0)?,
        competency_id: row.get(1)?,
        job_id: row.get(2)?,
        detail_id: row.get(3)?,
        highlight_text: row.get(4)?,
        sort_order: row.get(5)?,
    })
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs", post(create_job))
        .route("/jobs/:id", put(update_job).delete(delete_job))
        .route("/jobs/:job_id/details", post(create_job_detail))
        .route(
            "/jobs/:job_id/details/:id",
            put(update_job_detail).delete(delete_job_detail),
        )
        .route("/competencies", post(create_competency))
        .route(
            "/competencies/:id",
            put(update_competency).delete(delete_competency),
        )
        .route("/evidence", post(create_evidence))
        .route(
            "/evidence/:id",
            put(update_evidence).delete(delete_evidence),
        )
}
