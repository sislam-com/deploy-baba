use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
};
use std::sync::Arc;

use crate::auth::Claims;
use crate::db::Db;

// ── Template data structs ─────────────────────────────────────────────────────

pub struct JobListItem {
    pub slug: String,
    pub company: String,
    pub title: String,
    pub start_date: String,
    pub end_date: String,
}

pub struct JobForDetail {
    pub id: i64,
    pub slug: String,
    pub company: String,
    pub title: String,
    pub location: String,
    pub start_date: String,
    pub end_date: String,
    pub summary: String,
    pub tech_stack: String,
    pub sort_order: i64,
}

pub struct JobDetailItem {
    pub id: i64,
    pub detail_text: String,
    pub category: String,
    pub sort_order: i64,
}

pub struct EvidenceForJob {
    pub id: i64,
    pub competency_id: i64,
    pub detail_id: String,
    pub highlight_text: String,
    pub sort_order: i64,
}

pub struct EvidenceForCompetency {
    pub id: i64,
    pub job_id: i64,
    pub job_slug: String,
    pub detail_id: String,
    pub highlight_text: String,
    pub detail_text: String,
    pub sort_order: i64,
}

pub struct JobNavItem {
    pub slug: String,
    pub label: String,
}

pub struct CompetencyListItem {
    pub slug: String,
    pub name: String,
}

pub struct CompetencyForDetail {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub sort_order: i64,
}

pub struct CompetencySelectItem {
    pub id: i64,
    pub name: String,
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard_home.html")]
pub struct DashboardHomeTemplate {
    pub username: String,
    pub jobs_count: i64,
    pub job_details_count: i64,
    pub competencies_count: i64,
    pub evidence_count: i64,
}

#[derive(Template)]
#[template(path = "dashboard_jobs_list.html")]
pub struct DashboardJobsListTemplate {
    pub username: String,
    pub jobs: Vec<JobListItem>,
}

#[derive(Template)]
#[template(path = "dashboard_job_detail.html")]
pub struct DashboardJobDetailTemplate {
    pub username: String,
    pub job: JobForDetail,
    pub details: Vec<JobDetailItem>,
    pub evidence: Vec<EvidenceForJob>,
    pub all_jobs: Vec<JobNavItem>,
    pub all_competencies: Vec<CompetencySelectItem>,
    pub is_new: bool,
}

#[derive(Template)]
#[template(path = "dashboard_competencies_list.html")]
pub struct DashboardCompetenciesListTemplate {
    pub username: String,
    pub competencies: Vec<CompetencyListItem>,
}

#[derive(Template)]
#[template(path = "dashboard_competency_detail.html")]
pub struct DashboardCompetencyDetailTemplate {
    pub username: String,
    pub competency: CompetencyForDetail,
    pub evidence: Vec<EvidenceForCompetency>,
    pub all_jobs: Vec<JobNavItem>,
}

// ── Error helper ──────────────────────────────────────────────────────────────

fn db_err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn dashboard_home(
    Extension(claims): Extension<Claims>,
    State(db): State<Arc<Db>>,
) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();
    let jobs_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0))
        .unwrap_or(0);
    let job_details_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM job_details", [], |r| r.get(0))
        .unwrap_or(0);
    let competencies_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM competencies", [], |r| r.get(0))
        .unwrap_or(0);
    let evidence_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM competency_evidence", [], |r| r.get(0))
        .unwrap_or(0);

    DashboardHomeTemplate {
        username: claims.username,
        jobs_count,
        job_details_count,
        competencies_count,
        evidence_count,
    }
}

pub async fn dashboard_jobs_list(
    Extension(claims): Extension<Claims>,
    State(db): State<Arc<Db>>,
) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT slug, company, title, start_date, end_date
             FROM jobs ORDER BY sort_order ASC",
        )
        .unwrap();
    let jobs = stmt
        .query_map([], |row| {
            let end_date: Option<String> = row.get(4)?;
            Ok(JobListItem {
                slug: row.get(0)?,
                company: row.get(1)?,
                title: row.get(2)?,
                start_date: row.get(3)?,
                end_date: end_date.unwrap_or_default(),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    DashboardJobsListTemplate {
        username: claims.username,
        jobs,
    }
}

pub async fn dashboard_job_new(
    Extension(claims): Extension<Claims>,
    State(db): State<Arc<Db>>,
) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();
    let all_jobs = load_job_nav_items(&conn);
    let all_competencies = load_competency_select_items(&conn);

    DashboardJobDetailTemplate {
        username: claims.username,
        job: JobForDetail {
            id: 0,
            slug: String::new(),
            company: String::new(),
            title: String::new(),
            location: String::new(),
            start_date: String::new(),
            end_date: String::new(),
            summary: String::new(),
            tech_stack: String::new(),
            sort_order: 0,
        },
        details: Vec::new(),
        evidence: Vec::new(),
        all_jobs,
        all_competencies,
        is_new: true,
    }
}

pub async fn dashboard_job_detail(
    Extension(claims): Extension<Claims>,
    State(db): State<Arc<Db>>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let job = conn
        .query_row(
            "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order
             FROM jobs WHERE slug = ?1",
            rusqlite::params![slug],
            |row| {
                let location: Option<String> = row.get(4)?;
                let end_date: Option<String> = row.get(6)?;
                let tech_stack: Option<String> = row.get(8)?;
                Ok(JobForDetail {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    company: row.get(2)?,
                    title: row.get(3)?,
                    location: location.unwrap_or_default(),
                    start_date: row.get(5)?,
                    end_date: end_date.unwrap_or_default(),
                    summary: row.get(7)?,
                    tech_stack: tech_stack.unwrap_or_default(),
                    sort_order: row.get(9)?,
                })
            },
        )
        .map_err(|_| (StatusCode::NOT_FOUND, format!("Job '{}' not found", slug)))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, detail_text, category, sort_order
             FROM job_details WHERE job_id = ?1 ORDER BY sort_order ASC",
        )
        .map_err(db_err)?;
    let details = stmt
        .query_map(rusqlite::params![job.id], |row| {
            let category: Option<String> = row.get(2)?;
            Ok(JobDetailItem {
                id: row.get(0)?,
                detail_text: row.get(1)?,
                category: category.unwrap_or_default(),
                sort_order: row.get(3)?,
            })
        })
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt = conn
        .prepare(
            "SELECT ce.id, ce.competency_id, ce.detail_id, ce.highlight_text, ce.sort_order
             FROM competency_evidence ce
             WHERE ce.job_id = ?1
             ORDER BY ce.sort_order ASC",
        )
        .map_err(db_err)?;
    let evidence = stmt
        .query_map(rusqlite::params![job.id], |row| {
            let detail_id: Option<i64> = row.get(2)?;
            let highlight_text: Option<String> = row.get(3)?;
            Ok(EvidenceForJob {
                id: row.get(0)?,
                competency_id: row.get(1)?,
                detail_id: detail_id.map(|d| d.to_string()).unwrap_or_default(),
                highlight_text: highlight_text.unwrap_or_default(),
                sort_order: row.get(4)?,
            })
        })
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    let all_jobs = load_job_nav_items(&conn);
    let all_competencies = load_competency_select_items(&conn);

    Ok(DashboardJobDetailTemplate {
        username: claims.username,
        job,
        details,
        evidence,
        all_jobs,
        all_competencies,
        is_new: false,
    })
}

pub async fn dashboard_competencies_list(
    Extension(claims): Extension<Claims>,
    State(db): State<Arc<Db>>,
) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT slug, name FROM competencies ORDER BY sort_order ASC")
        .unwrap();
    let competencies = stmt
        .query_map([], |row| {
            Ok(CompetencyListItem {
                slug: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    DashboardCompetenciesListTemplate {
        username: claims.username,
        competencies,
    }
}

pub async fn dashboard_competency_detail(
    Extension(claims): Extension<Claims>,
    State(db): State<Arc<Db>>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = db.conn.lock().unwrap();

    let competency = conn
        .query_row(
            "SELECT id, slug, name, description, icon, sort_order
             FROM competencies WHERE slug = ?1",
            rusqlite::params![slug],
            |row| {
                let icon: Option<String> = row.get(4)?;
                Ok(CompetencyForDetail {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    icon: icon.unwrap_or_default(),
                    sort_order: row.get(5)?,
                })
            },
        )
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
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
        .map_err(db_err)?;
    let evidence = stmt
        .query_map(rusqlite::params![competency.id], |row| {
            let detail_id: Option<i64> = row.get(4)?;
            let highlight_text: Option<String> = row.get(5)?;
            let detail_text: Option<String> = row.get(6)?;
            Ok(EvidenceForCompetency {
                id: row.get(0)?,
                job_id: row.get(1)?,
                job_slug: row.get(2)?,
                detail_id: detail_id.map(|d| d.to_string()).unwrap_or_default(),
                highlight_text: highlight_text.unwrap_or_default(),
                detail_text: detail_text.unwrap_or_default(),
                sort_order: row.get(7)?,
            })
        })
        .map_err(db_err)?
        .filter_map(|r| r.ok())
        .collect();

    let all_jobs = load_job_nav_items(&conn);

    Ok(DashboardCompetencyDetailTemplate {
        username: claims.username,
        competency,
        evidence,
        all_jobs,
    })
}

// ── Private helpers ────────────────────────────────────────────────────────────

fn load_job_nav_items(conn: &rusqlite::Connection) -> Vec<JobNavItem> {
    let mut stmt = conn
        .prepare("SELECT slug, company, title FROM jobs ORDER BY sort_order ASC")
        .unwrap();
    stmt.query_map([], |row| {
        let company: String = row.get(1)?;
        let title: String = row.get(2)?;
        Ok(JobNavItem {
            slug: row.get(0)?,
            label: format!("{} — {}", company, title),
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

fn load_competency_select_items(conn: &rusqlite::Connection) -> Vec<CompetencySelectItem> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM competencies ORDER BY sort_order ASC")
        .unwrap();
    stmt.query_map([], |row| {
        Ok(CompetencySelectItem {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}
