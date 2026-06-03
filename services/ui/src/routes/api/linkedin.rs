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
    ApplyFieldsRequest, ApplyResult, AutoMatchResult, BulkStatusRequest, BulkStatusResult,
    LinkedInImportPayload, LinkedInImportResult, LinkedInOAuthStatus, LinkedInOAuthToken,
    LinkedInPosition, LinkedInProject, LinkedInSyncLogEntry, MapRequest, PositionDiff, ProjectDiff,
    ReconciliationItem, ReconciliationSummary, StatusUpdateRequest, SyncFieldComparison,
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

// ── OAuth Token Persistence ─────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/api/admin/linkedin/oauth-token",
    tag = "admin-linkedin",
    request_body = LinkedInOAuthToken,
    responses(
        (status = 200, description = "Token stored"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn upsert_oauth_token(
    State(db): State<Arc<Db>>,
    Json(token): Json<LinkedInOAuthToken>,
) -> ApiResult<StatusCode> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO linkedin_oauth_tokens (id, access_token, expires_at, name, email, picture_url, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET
           access_token = excluded.access_token,
           expires_at = excluded.expires_at,
           name = excluded.name,
           email = excluded.email,
           picture_url = excluded.picture_url,
           updated_at = datetime('now')",
        rusqlite::params![
            token.access_token,
            token.expires_at,
            token.name,
            token.email,
            token.picture_url,
        ],
    )
    .map_err(db_err)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/admin/linkedin/oauth-token",
    tag = "admin-linkedin",
    responses(
        (status = 200, description = "Current OAuth status", body = LinkedInOAuthStatus),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn get_oauth_token(State(db): State<Arc<Db>>) -> ApiResult<Json<LinkedInOAuthStatus>> {
    let conn = db.conn.lock().unwrap();
    let result = conn.query_row(
        "SELECT access_token, expires_at, name, email, picture_url FROM linkedin_oauth_tokens WHERE id = 1",
        [],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        },
    );

    match result {
        Ok((_token, expires_at, name, email, picture_url)) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let connected = expires_at > now;
            Ok(Json(LinkedInOAuthStatus {
                connected,
                name,
                email,
                picture_url,
                token_expires_at: Some(expires_at.to_string()),
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Json(LinkedInOAuthStatus {
            connected: false,
            name: None,
            email: None,
            picture_url: None,
            token_expires_at: None,
        })),
        Err(e) => Err(db_err(e)),
    }
}

#[utoipa::path(
    delete,
    path = "/api/admin/linkedin/oauth-token",
    tag = "admin-linkedin",
    responses(
        (status = 200, description = "Token cleared"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn delete_oauth_token(State(db): State<Arc<Db>>) -> ApiResult<StatusCode> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM linkedin_oauth_tokens WHERE id = 1", [])
        .map_err(db_err)?;
    Ok(StatusCode::OK)
}

// ── Bulk Operations ────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/api/admin/linkedin/positions/bulk-status",
    tag = "admin-linkedin",
    request_body = BulkStatusRequest,
    responses(
        (status = 200, description = "Bulk status updated", body = BulkStatusResult),
        (status = 400, description = "Invalid status"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn bulk_update_position_status(
    State(db): State<Arc<Db>>,
    Json(req): Json<BulkStatusRequest>,
) -> ApiResult<Json<BulkStatusResult>> {
    if !VALID_STATUSES.contains(&req.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid status '{}'. Valid: {:?}",
                req.status, VALID_STATUSES
            ),
        ));
    }
    if req.ids.is_empty() {
        return Ok(Json(BulkStatusResult { updated: 0 }));
    }

    let conn = db.conn.lock().unwrap();
    let placeholders: Vec<String> = req
        .ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 2))
        .collect();
    let sql = format!(
        "UPDATE linkedin_positions SET sync_status = ?1, reviewed_at = datetime('now') WHERE id IN ({})",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql).map_err(db_err)?;
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    params.push(Box::new(req.status.clone()));
    for id in &req.ids {
        params.push(Box::new(*id));
    }
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let updated = stmt.execute(refs.as_slice()).map_err(db_err)?;

    Ok(Json(BulkStatusResult {
        updated: updated as i64,
    }))
}

#[utoipa::path(
    put,
    path = "/api/admin/linkedin/projects/bulk-status",
    tag = "admin-linkedin",
    request_body = BulkStatusRequest,
    responses(
        (status = 200, description = "Bulk status updated", body = BulkStatusResult),
        (status = 400, description = "Invalid status"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn bulk_update_project_status(
    State(db): State<Arc<Db>>,
    Json(req): Json<BulkStatusRequest>,
) -> ApiResult<Json<BulkStatusResult>> {
    if !VALID_STATUSES.contains(&req.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid status '{}'. Valid: {:?}",
                req.status, VALID_STATUSES
            ),
        ));
    }
    if req.ids.is_empty() {
        return Ok(Json(BulkStatusResult { updated: 0 }));
    }

    let conn = db.conn.lock().unwrap();
    let placeholders: Vec<String> = req
        .ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 2))
        .collect();
    let sql = format!(
        "UPDATE linkedin_projects SET sync_status = ?1, reviewed_at = datetime('now') WHERE id IN ({})",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql).map_err(db_err)?;
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    params.push(Box::new(req.status.clone()));
    for id in &req.ids {
        params.push(Box::new(*id));
    }
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let updated = stmt.execute(refs.as_slice()).map_err(db_err)?;

    Ok(Json(BulkStatusResult {
        updated: updated as i64,
    }))
}

#[utoipa::path(
    post,
    path = "/api/admin/linkedin/auto-match",
    tag = "admin-linkedin",
    responses(
        (status = 200, description = "Auto-match completed", body = AutoMatchResult),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn run_auto_match(State(db): State<Arc<Db>>) -> ApiResult<Json<AutoMatchResult>> {
    let conn = db.conn.lock().unwrap();
    let positions_matched = auto_match_positions(&conn);
    let projects_matched = auto_match_projects(&conn);
    Ok(Json(AutoMatchResult {
        positions_matched,
        projects_matched,
    }))
}

// ── Reconciliation ─────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/linkedin/reconciliation",
    tag = "admin-linkedin",
    responses(
        (status = 200, description = "Reconciliation summary", body = ReconciliationSummary),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn get_reconciliation(
    State(db): State<Arc<Db>>,
) -> ApiResult<Json<ReconciliationSummary>> {
    let conn = db.conn.lock().unwrap();

    let mut needs_linkedin_update = Vec::new();
    let mut needs_db_import = Vec::new();
    let mut in_sync = Vec::new();

    // Positions
    let mut stmt = conn
        .prepare(
            "SELECT id, company, title, sync_status, mapped_job_id
             FROM linkedin_positions ORDER BY id",
        )
        .map_err(db_err)?;

    let pos_rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        })
        .map_err(db_err)?;

    for row in pos_rows.flatten() {
        let (id, _company, title, sync_status, mapped_job_id) = row;
        let has_mapping = mapped_job_id.is_some();

        let differing_fields = if let Some(job_id) = mapped_job_id {
            get_position_differing_fields(&conn, id, job_id)
        } else {
            vec![]
        };

        let item = ReconciliationItem {
            id,
            entity_type: "position".to_string(),
            title,
            sync_status: sync_status.clone(),
            has_mapping,
            differing_fields: differing_fields.clone(),
        };

        match sync_status.as_str() {
            "synced" => in_sync.push(item),
            "linkedin_only" | "unreviewed" if !has_mapping => needs_db_import.push(item),
            "diverged" if !differing_fields.is_empty() => needs_linkedin_update.push(item),
            _ => needs_db_import.push(item),
        }
    }

    // Projects
    let mut stmt = conn
        .prepare(
            "SELECT id, title, sync_status, mapped_challenge_id
             FROM linkedin_projects ORDER BY id",
        )
        .map_err(db_err)?;

    let proj_rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })
        .map_err(db_err)?;

    for row in proj_rows.flatten() {
        let (id, title, sync_status, mapped_challenge_id) = row;
        let has_mapping = mapped_challenge_id.is_some();

        let differing_fields = if let Some(ch_id) = mapped_challenge_id {
            get_project_differing_fields(&conn, id, ch_id)
        } else {
            vec![]
        };

        let item = ReconciliationItem {
            id,
            entity_type: "project".to_string(),
            title,
            sync_status: sync_status.clone(),
            has_mapping,
            differing_fields: differing_fields.clone(),
        };

        match sync_status.as_str() {
            "synced" => in_sync.push(item),
            "linkedin_only" | "unreviewed" if !has_mapping => needs_db_import.push(item),
            "diverged" if !differing_fields.is_empty() => needs_linkedin_update.push(item),
            _ => needs_db_import.push(item),
        }
    }

    Ok(Json(ReconciliationSummary {
        needs_linkedin_update,
        needs_db_import,
        in_sync,
    }))
}

fn get_position_differing_fields(
    conn: &rusqlite::Connection,
    pos_id: i64,
    job_id: i64,
) -> Vec<String> {
    let result = conn.query_row(
        "SELECT lp.company, lp.title, lp.location, lp.start_date, lp.end_date, lp.description,
                j.company, j.title, j.location, j.start_date, j.end_date, j.summary
         FROM linkedin_positions lp, jobs j
         WHERE lp.id = ?1 AND j.id = ?2",
        rusqlite::params![pos_id, job_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
            ))
        },
    );

    match result {
        Ok((
            lp_company,
            lp_title,
            lp_location,
            lp_start,
            lp_end,
            lp_desc,
            j_company,
            j_title,
            j_location,
            j_start,
            j_end,
            j_desc,
        )) => {
            let mut diffs = Vec::new();
            if lp_company != j_company {
                diffs.push("company".to_string());
            }
            if lp_title != j_title {
                diffs.push("title".to_string());
            }
            if lp_location != j_location {
                diffs.push("location".to_string());
            }
            if lp_start != j_start {
                diffs.push("start_date".to_string());
            }
            if lp_end != j_end {
                diffs.push("end_date".to_string());
            }
            if lp_desc != j_desc {
                diffs.push("description".to_string());
            }
            diffs
        }
        Err(_) => vec![],
    }
}

fn get_project_differing_fields(
    conn: &rusqlite::Connection,
    proj_id: i64,
    ch_id: i64,
) -> Vec<String> {
    let result = conn.query_row(
        "SELECT lp.title, lp.description, lp.url,
                c.title, c.description, c.url
         FROM linkedin_projects lp, challenges c
         WHERE lp.id = ?1 AND c.id = ?2",
        rusqlite::params![proj_id, ch_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        },
    );

    match result {
        Ok((lp_title, lp_desc, lp_url, c_title, c_desc, c_url)) => {
            let mut diffs = Vec::new();
            if lp_title != c_title {
                diffs.push("title".to_string());
            }
            if lp_desc != c_desc {
                diffs.push("description".to_string());
            }
            if lp_url != c_url {
                diffs.push("url".to_string());
            }
            diffs
        }
        Err(_) => vec![],
    }
}

// ── Apply Actions ──────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/linkedin/positions/{id}/apply",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn position ID")),
    request_body = ApplyFieldsRequest,
    responses(
        (status = 200, description = "Fields applied", body = ApplyResult),
        (status = 404, description = "Position not found or not mapped"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn apply_position_to_job(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(req): Json<ApplyFieldsRequest>,
) -> ApiResult<Json<ApplyResult>> {
    let conn = db.conn.lock().unwrap();

    let (mapped_job_id, lp_company, lp_title, lp_location, lp_start, lp_end, lp_desc) = conn
        .query_row(
            "SELECT mapped_job_id, company, title, location, start_date, end_date, description
             FROM linkedin_positions WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(format!("Position {} not found", id)),
            other => db_err(other),
        })?;

    let job_id =
        mapped_job_id.ok_or_else(|| not_found(format!("Position {} has no mapped job", id)))?;

    let valid_fields = [
        "company",
        "title",
        "location",
        "start_date",
        "end_date",
        "description",
    ];
    let mut applied = Vec::new();

    for field in &req.fields {
        if !valid_fields.contains(&field.as_str()) {
            continue;
        }
        let (col, val): (&str, &dyn rusqlite::types::ToSql) = match field.as_str() {
            "company" => ("company", &lp_company),
            "title" => ("title", &lp_title),
            "location" => ("location", &lp_location),
            "start_date" => ("start_date", &lp_start),
            "end_date" => ("end_date", &lp_end),
            "description" => ("summary", &lp_desc),
            _ => continue,
        };
        let sql = format!("UPDATE jobs SET {} = ?1 WHERE id = ?2", col);
        conn.execute(&sql, rusqlite::params![val, job_id])
            .map_err(db_err)?;
        applied.push(field.clone());
    }

    if !applied.is_empty() {
        conn.execute(
            "UPDATE linkedin_positions SET sync_status = 'synced', reviewed_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(db_err)?;
    }

    Ok(Json(ApplyResult {
        fields_applied: applied,
    }))
}

#[utoipa::path(
    post,
    path = "/api/admin/linkedin/projects/{id}/apply",
    tag = "admin-linkedin",
    params(("id" = i64, Path, description = "LinkedIn project ID")),
    request_body = ApplyFieldsRequest,
    responses(
        (status = 200, description = "Fields applied", body = ApplyResult),
        (status = 404, description = "Project not found or not mapped"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []), ("bearerAuth" = [])),
)]
pub async fn apply_project_to_challenge(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(req): Json<ApplyFieldsRequest>,
) -> ApiResult<Json<ApplyResult>> {
    let conn = db.conn.lock().unwrap();

    let (mapped_challenge_id, lp_title, lp_desc, lp_url) = conn
        .query_row(
            "SELECT mapped_challenge_id, title, description, url
             FROM linkedin_projects WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(format!("Project {} not found", id)),
            other => db_err(other),
        })?;

    let ch_id = mapped_challenge_id
        .ok_or_else(|| not_found(format!("Project {} has no mapped challenge", id)))?;

    let valid_fields = ["title", "description", "url"];
    let mut applied = Vec::new();

    for field in &req.fields {
        if !valid_fields.contains(&field.as_str()) {
            continue;
        }
        let (col, val): (&str, &dyn rusqlite::types::ToSql) = match field.as_str() {
            "title" => ("title", &lp_title),
            "description" => ("description", &lp_desc),
            "url" => ("url", &lp_url),
            _ => continue,
        };
        let sql = format!("UPDATE challenges SET {} = ?1 WHERE id = ?2", col);
        conn.execute(&sql, rusqlite::params![val, ch_id])
            .map_err(db_err)?;
        applied.push(field.clone());
    }

    if !applied.is_empty() {
        conn.execute(
            "UPDATE linkedin_projects SET sync_status = 'synced', reviewed_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(db_err)?;
    }

    Ok(Json(ApplyResult {
        fields_applied: applied,
    }))
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
        .route(
            "/oauth-token",
            get(get_oauth_token)
                .put(upsert_oauth_token)
                .delete(delete_oauth_token),
        )
        .route("/positions/bulk-status", put(bulk_update_position_status))
        .route("/projects/bulk-status", put(bulk_update_project_status))
        .route("/auto-match", post(run_auto_match))
        .route("/reconciliation", get(get_reconciliation))
        .route("/positions/:id/apply", post(apply_position_to_job))
        .route("/projects/:id/apply", post(apply_project_to_challenge))
}
