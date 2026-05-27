use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use std::sync::Arc;

use crate::db::Db;
use crate::state::AppState;

pub use api_openapi::models::{
    LinkedInImportPayload, LinkedInImportResult, LinkedInPosition, LinkedInProject,
    LinkedInSyncLogEntry, MapRequest, PositionDiff, ProjectDiff, StatusUpdateRequest,
    SyncFieldComparison,
};

type ApiResult<T> = Result<T, (StatusCode, String)>;

fn db_err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn not_found(msg: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, msg.into())
}

const VALID_STATUSES: &[&str] = &[
    "unreviewed",
    "synced",
    "diverged",
    "linkedin_only",
    "local_only",
];

fn row_to_position(row: &rusqlite::Row<'_>) -> rusqlite::Result<LinkedInPosition> {
    Ok(LinkedInPosition {
        id: row.get(0)?,
        linkedin_id: row.get(1)?,
        company: row.get(2)?,
        title: row.get(3)?,
        location: row.get(4)?,
        start_date: row.get(5)?,
        end_date: row.get(6)?,
        description: row.get(7)?,
        mapped_job_id: row.get(8)?,
        sync_status: row.get(9)?,
        imported_at: row.get(10)?,
        reviewed_at: row.get(11)?,
    })
}

fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<LinkedInProject> {
    Ok(LinkedInProject {
        id: row.get(0)?,
        linkedin_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        url: row.get(4)?,
        start_date: row.get(5)?,
        end_date: row.get(6)?,
        associated_position: row.get(7)?,
        mapped_challenge_id: row.get(8)?,
        sync_status: row.get(9)?,
        imported_at: row.get(10)?,
        reviewed_at: row.get(11)?,
    })
}

// ── Import ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/linkedin/import",
    tag = "admin-linkedin",
    request_body = LinkedInImportPayload,
    responses(
        (status = 200, description = "Import complete", body = LinkedInImportResult),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn import_linkedin_data(
    State(db): State<Arc<Db>>,
    Json(payload): Json<LinkedInImportPayload>,
) -> ApiResult<Json<LinkedInImportResult>> {
    let conn = db.conn.lock().unwrap();

    let mut positions_imported: i64 = 0;
    for p in &payload.positions {
        conn.execute(
            "INSERT INTO linkedin_positions (company, title, location, start_date, end_date, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![p.company, p.title, p.location, p.start_date, p.end_date, p.description],
        )
        .map_err(db_err)?;
        positions_imported += 1;
    }

    let mut projects_imported: i64 = 0;
    for p in &payload.projects {
        conn.execute(
            "INSERT INTO linkedin_projects (title, description, url, start_date, end_date, associated_position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![p.title, p.description, p.url, p.start_date, p.end_date, p.associated_position],
        )
        .map_err(db_err)?;
        projects_imported += 1;
    }

    let positions_matched = auto_match_positions(&conn);
    let projects_matched = auto_match_projects(&conn);

    conn.execute(
        "INSERT INTO linkedin_sync_log (source, positions_count, projects_count) VALUES ('upload', ?1, ?2)",
        rusqlite::params![positions_imported, projects_imported],
    )
    .map_err(db_err)?;

    Ok(Json(LinkedInImportResult {
        positions_imported,
        projects_imported,
        positions_matched,
        projects_matched,
    }))
}

fn auto_match_positions(conn: &rusqlite::Connection) -> i64 {
    let mut matched: i64 = 0;
    let result = conn.execute(
        "UPDATE linkedin_positions SET
            mapped_job_id = (
                SELECT j.id FROM jobs j
                WHERE LOWER(j.company) = LOWER(linkedin_positions.company)
                LIMIT 1
            ),
            sync_status = 'diverged'
         WHERE mapped_job_id IS NULL
           AND EXISTS (
                SELECT 1 FROM jobs j
                WHERE LOWER(j.company) = LOWER(linkedin_positions.company)
           )",
        [],
    );
    if let Ok(n) = result {
        matched = n as i64;
    }
    matched
}

fn auto_match_projects(conn: &rusqlite::Connection) -> i64 {
    let mut matched: i64 = 0;
    let result = conn.execute(
        "UPDATE linkedin_projects SET
            mapped_challenge_id = (
                SELECT c.id FROM challenges c
                WHERE LOWER(c.title) = LOWER(linkedin_projects.title)
                LIMIT 1
            ),
            sync_status = 'diverged'
         WHERE mapped_challenge_id IS NULL
           AND EXISTS (
                SELECT 1 FROM challenges c
                WHERE LOWER(c.title) = LOWER(linkedin_projects.title)
           )",
        [],
    );
    if let Ok(n) = result {
        matched = n as i64;
    }
    matched
}

// ── List ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/linkedin/positions",
    tag = "admin-linkedin",
    responses(
        (status = 200, description = "LinkedIn positions", body = Vec<LinkedInPosition>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn list_linkedin_positions(
    State(db): State<Arc<Db>>,
) -> ApiResult<Json<Vec<LinkedInPosition>>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, linkedin_id, company, title, location, start_date, end_date,
                    description, mapped_job_id, sync_status, imported_at, reviewed_at
             FROM linkedin_positions ORDER BY start_date DESC",
        )
        .map_err(db_err)?;

    let positions = stmt
        .query_map([], row_to_position)
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(positions))
}

#[utoipa::path(
    get,
    path = "/api/admin/linkedin/projects",
    tag = "admin-linkedin",
    responses(
        (status = 200, description = "LinkedIn projects", body = Vec<LinkedInProject>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn list_linkedin_projects(
    State(db): State<Arc<Db>>,
) -> ApiResult<Json<Vec<LinkedInProject>>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, linkedin_id, title, description, url, start_date, end_date,
                    associated_position, mapped_challenge_id, sync_status, imported_at, reviewed_at
             FROM linkedin_projects ORDER BY start_date DESC",
        )
        .map_err(db_err)?;

    let projects = stmt
        .query_map([], row_to_project)
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(projects))
}

// ── Diff ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/linkedin/positions/{id}/diff",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn position ID")),
    responses(
        (status = 200, description = "Position diff", body = PositionDiff),
        (status = 404, description = "Position not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn position_diff(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
) -> ApiResult<Json<PositionDiff>> {
    let conn = db.conn.lock().unwrap();

    let position = conn
        .query_row(
            "SELECT id, linkedin_id, company, title, location, start_date, end_date,
                    description, mapped_job_id, sync_status, imported_at, reviewed_at
             FROM linkedin_positions WHERE id = ?1",
            rusqlite::params![id],
            row_to_position,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(format!("Position {} not found", id)),
            other => db_err(other),
        })?;

    let (job_title, job_company, fields) = if let Some(job_id) = position.mapped_job_id {
        let job = conn
            .query_row(
                "SELECT company, title, location, start_date, end_date, summary FROM jobs WHERE id = ?1",
                rusqlite::params![job_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .map_err(db_err)?;

        let fields = vec![
            compare_field("company", Some(&position.company), Some(&job.0)),
            compare_field("title", Some(&position.title), Some(&job.1)),
            compare_field("location", position.location.as_deref(), job.2.as_deref()),
            compare_field("start_date", Some(&position.start_date), Some(&job.3)),
            compare_field("end_date", position.end_date.as_deref(), job.4.as_deref()),
            compare_field(
                "description",
                position.description.as_deref(),
                job.5.as_deref(),
            ),
        ];

        (Some(job.1), Some(job.0), fields)
    } else {
        (None, None, vec![])
    };

    Ok(Json(PositionDiff {
        position,
        job_title,
        job_company,
        fields,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/linkedin/projects/{id}/diff",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn project ID")),
    responses(
        (status = 200, description = "Project diff", body = ProjectDiff),
        (status = 404, description = "Project not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn project_diff(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
) -> ApiResult<Json<ProjectDiff>> {
    let conn = db.conn.lock().unwrap();

    let project = conn
        .query_row(
            "SELECT id, linkedin_id, title, description, url, start_date, end_date,
                    associated_position, mapped_challenge_id, sync_status, imported_at, reviewed_at
             FROM linkedin_projects WHERE id = ?1",
            rusqlite::params![id],
            row_to_project,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(format!("Project {} not found", id)),
            other => db_err(other),
        })?;

    let (challenge_title, fields) = if let Some(ch_id) = project.mapped_challenge_id {
        let ch = conn
            .query_row(
                "SELECT title, description, url FROM challenges WHERE id = ?1",
                rusqlite::params![ch_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .map_err(db_err)?;

        let fields = vec![
            compare_field("title", Some(&project.title), Some(&ch.0)),
            compare_field("description", project.description.as_deref(), Some(&ch.1)),
            compare_field("url", project.url.as_deref(), ch.2.as_deref()),
        ];

        (Some(ch.0), fields)
    } else {
        (None, vec![])
    };

    Ok(Json(ProjectDiff {
        project,
        challenge_title,
        fields,
    }))
}

fn compare_field(name: &str, linkedin: Option<&str>, db: Option<&str>) -> SyncFieldComparison {
    let l = linkedin.map(|s| s.to_string());
    let d = db.map(|s| s.to_string());
    let differs = l != d;
    SyncFieldComparison {
        field: name.to_string(),
        linkedin_value: l,
        db_value: d,
        differs,
    }
}

// ── Map ─────────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/api/admin/linkedin/positions/{id}/map",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn position ID")),
    request_body = MapRequest,
    responses(
        (status = 200, description = "Mapped"),
        (status = 404, description = "Position not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn map_position_to_job(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(req): Json<MapRequest>,
) -> ApiResult<StatusCode> {
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "UPDATE linkedin_positions SET mapped_job_id = ?1 WHERE id = ?2",
            rusqlite::params![req.target_id, id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Position {} not found", id)));
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(
    put,
    path = "/api/admin/linkedin/projects/{id}/map",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn project ID")),
    request_body = MapRequest,
    responses(
        (status = 200, description = "Mapped"),
        (status = 404, description = "Project not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn map_project_to_challenge(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(req): Json<MapRequest>,
) -> ApiResult<StatusCode> {
    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "UPDATE linkedin_projects SET mapped_challenge_id = ?1 WHERE id = ?2",
            rusqlite::params![req.target_id, id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Project {} not found", id)));
    }
    Ok(StatusCode::OK)
}

// ── Status ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/api/admin/linkedin/positions/{id}/status",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn position ID")),
    request_body = StatusUpdateRequest,
    responses(
        (status = 200, description = "Status updated"),
        (status = 400, description = "Invalid status"),
        (status = 404, description = "Position not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn update_position_status(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(req): Json<StatusUpdateRequest>,
) -> ApiResult<StatusCode> {
    if !VALID_STATUSES.contains(&req.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid status '{}'. Valid: {:?}",
                req.status, VALID_STATUSES
            ),
        ));
    }

    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "UPDATE linkedin_positions SET sync_status = ?1, reviewed_at = datetime('now') WHERE id = ?2",
            rusqlite::params![req.status, id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Position {} not found", id)));
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(
    put,
    path = "/api/admin/linkedin/projects/{id}/status",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn project ID")),
    request_body = StatusUpdateRequest,
    responses(
        (status = 200, description = "Status updated"),
        (status = 400, description = "Invalid status"),
        (status = 404, description = "Project not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn update_project_status(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(req): Json<StatusUpdateRequest>,
) -> ApiResult<StatusCode> {
    if !VALID_STATUSES.contains(&req.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid status '{}'. Valid: {:?}",
                req.status, VALID_STATUSES
            ),
        ));
    }

    let conn = db.conn.lock().unwrap();
    let rows = conn
        .execute(
            "UPDATE linkedin_projects SET sync_status = ?1, reviewed_at = datetime('now') WHERE id = ?2",
            rusqlite::params![req.status, id],
        )
        .map_err(db_err)?;

    if rows == 0 {
        return Err(not_found(format!("Project {} not found", id)));
    }
    Ok(StatusCode::OK)
}

// ── Sync Log ────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/linkedin/sync-log",
    tag = "admin-linkedin",
    responses(
        (status = 200, description = "Sync log entries", body = Vec<LinkedInSyncLogEntry>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn list_sync_log(
    State(db): State<Arc<Db>>,
) -> ApiResult<Json<Vec<LinkedInSyncLogEntry>>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, source, positions_count, projects_count, imported_at
             FROM linkedin_sync_log ORDER BY imported_at DESC LIMIT 50",
        )
        .map_err(db_err)?;

    let entries = stmt
        .query_map([], |row| {
            Ok(LinkedInSyncLogEntry {
                id: row.get(0)?,
                source: row.get(1)?,
                positions_count: row.get(2)?,
                projects_count: row.get(3)?,
                imported_at: row.get(4)?,
            })
        })
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(entries))
}

// ── Router ──────────────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/import", post(import_linkedin_data))
        .route("/positions", get(list_linkedin_positions))
        .route("/projects", get(list_linkedin_projects))
        .route("/positions/:id/diff", get(position_diff))
        .route("/projects/:id/diff", get(project_diff))
        .route("/positions/:id/map", put(map_position_to_job))
        .route("/projects/:id/map", put(map_project_to_challenge))
        .route("/positions/:id/status", put(update_position_status))
        .route("/projects/:id/status", put(update_project_status))
        .route("/sync-log", get(list_sync_log))
}
