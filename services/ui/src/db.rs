use anyhow::Result;
use async_trait::async_trait;
use rag_core::{PortfolioDataProvider, RagError};
use rusqlite::Connection;
use std::sync::Mutex;

/// Compile-time embedded migrations — order matters.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_create_jobs",
        include_str!("../migrations/001_create_jobs.sql"),
    ),
    (
        "002_create_competencies",
        include_str!("../migrations/002_create_competencies.sql"),
    ),
    (
        "003_seed_jobs",
        include_str!("../migrations/003_seed_jobs.sql"),
    ),
    (
        "004_seed_job_details",
        include_str!("../migrations/004_seed_job_details.sql"),
    ),
    (
        "005_seed_competencies",
        include_str!("../migrations/005_seed_competencies.sql"),
    ),
    (
        "006_seed_competency_evidence",
        include_str!("../migrations/006_seed_competency_evidence.sql"),
    ),
    (
        "007_seed_personal_projects",
        include_str!("../migrations/007_seed_personal_projects.sql"),
    ),
    (
        "008_create_about_sections",
        include_str!("../migrations/008_create_about_sections.sql"),
    ),
    (
        "009_seed_about_sections",
        include_str!("../migrations/009_seed_about_sections.sql"),
    ),
    (
        "010_create_social_links",
        include_str!("../migrations/010_create_social_links.sql"),
    ),
    (
        "011_seed_social_links",
        include_str!("../migrations/011_seed_social_links.sql"),
    ),
    (
        "012_update_about_sections.sql",
        include_str!("../migrations/012_update_about_sections.sql"),
    ),
    (
        "013_align_linkedin_profile",
        include_str!("../migrations/013_align_linkedin_profile.sql"),
    ),
    (
        "014_add_sync_unique_indexes",
        include_str!("../migrations/014_add_sync_unique_indexes.sql"),
    ),
    (
        "015_sync_dashboard_2026-04-09",
        include_str!("../migrations/015_sync_dashboard_2026-04-09.sql"),
    ),
    (
        "016_sync_dashboard_2026-04-10",
        include_str!("../migrations/016_sync_dashboard_2026-04-10.sql"),
    ),
    (
        "017_rag_index",
        include_str!("../migrations/017_rag_index.sql"),
    ),
    (
        "018_resume_ai_positioning",
        include_str!("../migrations/018_resume_ai_positioning.sql"),
    ),
    (
        "019_add_me_summary",
        include_str!("../migrations/019_add_me_summary.sql"),
    ),
    (
        "020_fix_me_summary_content",
        include_str!("../migrations/020_fix_me_summary_content.sql"),
    ),
    (
        "021_update_competencies",
        include_str!("../migrations/021_update_competencies.sql"),
    ),
    (
        "022_create_challenges",
        include_str!("../migrations/022_create_challenges.sql"),
    ),
    (
        "023_rag_eval",
        include_str!("../migrations/023_rag_eval.sql"),
    ),
    (
        "024_resume_cleanup",
        include_str!("../migrations/024_resume_cleanup.sql"),
    ),
    (
        "025_resume_content_polish",
        include_str!("../migrations/025_resume_content_polish.sql"),
    ),
    (
        "026_metrics_tables",
        include_str!("../migrations/026_metrics_tables.sql"),
    ),
    (
        "027_outcome_focused_descriptions",
        include_str!("../migrations/027_outcome_focused_descriptions.sql"),
    ),
];

/// Re-exported from `api_openapi::models::social` — the canonical SSOT.
/// Only `url` and `label` are used for nav rendering (visible links only).
pub use api_openapi::models::SocialLink;

/// Load visible social links using an already-locked connection.
pub fn load_social_links(conn: &rusqlite::Connection) -> Vec<SocialLink> {
    let mut stmt = match conn
        .prepare("SELECT url, label FROM social_links WHERE visible = 1 ORDER BY sort_order ASC")
    {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map([], |row| {
        Ok(SocialLink {
            url: row.get(0)?,
            label: row.get(1)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

pub struct Db {
    pub conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT    NOT NULL UNIQUE,
                applied_at TEXT    NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        for (name, sql) in MIGRATIONS {
            let already_applied: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
                    rusqlite::params![name],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if already_applied {
                continue;
            }

            tracing::info!("Applying migration: {}", name);
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                rusqlite::params![name],
            )?;
        }

        Ok(())
    }
}

#[async_trait]
impl PortfolioDataProvider for Db {
    async fn get_jobs_summary(&self) -> std::result::Result<Vec<serde_json::Value>, RagError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT j.slug, j.company, j.title, j.location, j.start_date, j.end_date, j.summary, j.tech_stack
                 FROM jobs j ORDER BY j.sort_order ASC",
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        let jobs_json: Vec<(String, serde_json::Value)> = stmt
            .query_map([], |row| {
                let tech_raw: Option<String> = row.get(7)?;
                let slug = row.get::<_, String>(0)?;
                Ok((
                    slug.clone(),
                    serde_json::json!({
                        "slug": slug,
                        "company": row.get::<_, String>(1)?,
                        "title": row.get::<_, String>(2)?,
                        "location": row.get::<_, Option<String>>(3)?,
                        "start_date": row.get::<_, String>(4)?,
                        "end_date": row.get::<_, Option<String>>(5)?,
                        "summary": row.get::<_, Option<String>>(6)?,
                        "tech_stack": tech_raw,
                    }),
                ))
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        // Fetch details for each job
        let mut jobs_with_details = Vec::new();
        for (slug, mut job_val) in jobs_json {
            let job_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM jobs WHERE slug = ?1",
                    rusqlite::params![&slug],
                    |row| row.get(0),
                )
                .ok();

            if let Some(jid) = job_id {
                let mut detail_stmt = conn
                    .prepare(
                        "SELECT detail_text, category FROM job_details WHERE job_id = ?1 ORDER BY sort_order ASC",
                    )
                    .map_err(|e| RagError::Database(e.to_string()))?;

                let details: Vec<serde_json::Value> = detail_stmt
                    .query_map(rusqlite::params![jid], |row| {
                        Ok(serde_json::json!({
                            "text": row.get::<_, String>(0)?,
                            "category": row.get::<_, Option<String>>(1)?,
                        }))
                    })
                    .map_err(|e| RagError::Database(e.to_string()))?
                    .filter_map(|r| r.ok())
                    .collect();

                job_val["details"] = serde_json::Value::Array(details);
            }

            jobs_with_details.push(job_val);
        }

        Ok(jobs_with_details)
    }

    async fn get_job_details(
        &self,
        slug: &str,
    ) -> std::result::Result<Option<serde_json::Value>, RagError> {
        let conn = self.conn.lock().unwrap();

        let job = conn
            .query_row(
                "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack
                 FROM jobs WHERE slug = ?1",
                rusqlite::params![slug],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        serde_json::json!({
                            "slug": row.get::<_, String>(1)?,
                            "company": row.get::<_, String>(2)?,
                            "title": row.get::<_, String>(3)?,
                            "location": row.get::<_, Option<String>>(4)?,
                            "start_date": row.get::<_, String>(5)?,
                            "end_date": row.get::<_, Option<String>>(6)?,
                            "summary": row.get::<_, Option<String>>(7)?,
                            "tech_stack": row.get::<_, Option<String>>(8)?,
                        }),
                    ))
                },
            )
            .ok();

        let Some((job_id, mut job_val)) = job else {
            return Ok(None);
        };

        let mut stmt = conn
            .prepare(
                "SELECT detail_text, category
                 FROM job_details WHERE job_id = ?1 ORDER BY sort_order ASC",
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        let details: Vec<serde_json::Value> = stmt
            .query_map(rusqlite::params![job_id], |row| {
                Ok(serde_json::json!({
                    "detail_text": row.get::<_, String>(0)?,
                    "category": row.get::<_, Option<String>>(1)?,
                }))
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        job_val["details"] = serde_json::Value::Array(details);
        Ok(Some(job_val))
    }

    async fn get_competencies_summary(
        &self,
    ) -> std::result::Result<Vec<serde_json::Value>, RagError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT c.slug, c.name, c.description, c.icon
                 FROM competencies c ORDER BY c.sort_order ASC",
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        let competencies_json: Vec<(String, serde_json::Value)> = stmt
            .query_map([], |row| {
                let slug = row.get::<_, String>(0)?;
                Ok((
                    slug.clone(),
                    serde_json::json!({
                        "slug": slug,
                        "name": row.get::<_, String>(1)?,
                        "description": row.get::<_, Option<String>>(2)?,
                        "icon": row.get::<_, Option<String>>(3)?,
                    }),
                ))
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        // Fetch evidence for each competency
        let mut competencies_with_evidence = Vec::new();
        for (slug, mut comp_val) in competencies_json {
            let comp_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM competencies WHERE slug = ?1",
                    rusqlite::params![&slug],
                    |row| row.get(0),
                )
                .ok();

            if let Some(cid) = comp_id {
                let mut evidence_stmt = conn
                    .prepare(
                        "SELECT jd.detail_text, j.company
                         FROM competency_evidence ce
                         LEFT JOIN jobs j ON ce.job_id = j.id
                         LEFT JOIN job_details jd ON ce.detail_id = jd.id
                         WHERE ce.competency_id = ?1
                         ORDER BY ce.sort_order ASC",
                    )
                    .map_err(|e| RagError::Database(e.to_string()))?;

                let evidence: Vec<serde_json::Value> = evidence_stmt
                    .query_map(rusqlite::params![cid], |row| {
                        Ok(serde_json::json!({
                            "text": row.get::<_, Option<String>>(0)?,
                            "company": row.get::<_, Option<String>>(1)?,
                        }))
                    })
                    .map_err(|e| RagError::Database(e.to_string()))?
                    .filter_map(|r| r.ok())
                    .collect();

                comp_val["evidence"] = serde_json::Value::Array(evidence);
            }

            competencies_with_evidence.push(comp_val);
        }

        Ok(competencies_with_evidence)
    }

    async fn get_about_sections(&self) -> std::result::Result<Vec<serde_json::Value>, RagError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT page, slug, heading, body
                 FROM about_sections ORDER BY sort_order ASC",
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        let sections = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "page": row.get::<_, String>(0)?,
                    "slug": row.get::<_, String>(1)?,
                    "heading": row.get::<_, String>(2)?,
                    "body": row.get::<_, String>(3)?,
                }))
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(sections)
    }

    async fn get_challenges_summary(
        &self,
    ) -> std::result::Result<Vec<serde_json::Value>, RagError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT slug, title, description, short_description, tech_stack, category, url, featured
                 FROM challenges ORDER BY sort_order ASC",
            )
            .map_err(|e| RagError::Database(e.to_string()))?;

        let challenges = stmt
            .query_map([], |row| {
                let featured: i64 = row.get(7)?;
                Ok(serde_json::json!({
                    "entity_type": "challenge",
                    "slug": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "short_description": row.get::<_, Option<String>>(3)?,
                    "tech_stack": row.get::<_, Option<String>>(4)?,
                    "category": row.get::<_, Option<String>>(5)?,
                    "url": row.get::<_, Option<String>>(6)?,
                    "featured": featured != 0,
                }))
            })
            .map_err(|e| RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(challenges)
    }
}
