use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use std::sync::Arc;

use crate::db::{load_social_links, Db, SocialLink};

pub struct JobSummary {
    pub slug: String,
    pub company: String,
    pub title: String,
    pub start_date: String,
    pub end_date: String,
    pub summary: String,
    pub tech_stack: Vec<String>,
}

pub struct CompetencySummary {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}

#[derive(Template)]
#[template(path = "resume.html")]
struct ResumeTemplate {
    jobs: Vec<JobSummary>,
    competencies: Vec<CompetencySummary>,
    social_links: Vec<SocialLink>,
}

pub async fn handler(State(db): State<Arc<Db>>) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT slug, company, title, start_date, end_date, summary, tech_stack
             FROM jobs ORDER BY sort_order ASC",
        )
        .expect("prepare jobs");

    let jobs: Vec<JobSummary> = stmt
        .query_map([], |row| {
            let tech_raw: Option<String> = row.get(6)?;
            Ok(JobSummary {
                slug: row.get(0)?,
                company: row.get(1)?,
                title: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row
                    .get::<_, Option<String>>(4)?
                    .unwrap_or_else(|| "Present".into()),
                summary: row.get(5)?,
                tech_stack: tech_raw
                    .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                    .unwrap_or_default(),
            })
        })
        .expect("query jobs")
        .filter_map(|r| r.ok())
        .collect();

    let mut cstmt = conn
        .prepare("SELECT slug, name, description, icon FROM competencies ORDER BY sort_order ASC")
        .expect("prepare competencies");

    let competencies: Vec<CompetencySummary> = cstmt
        .query_map([], |row| {
            Ok(CompetencySummary {
                slug: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            })
        })
        .expect("query competencies")
        .filter_map(|r| r.ok())
        .collect();

    let social_links = load_social_links(&conn);

    ResumeTemplate {
        jobs,
        competencies,
        social_links,
    }
}
