use rusqlite::Connection;
use service_protocol::ServiceResponse;
use std::sync::{Arc, Mutex};
use tracing::error;

pub async fn get_resume(db: &Arc<Mutex<Connection>>) -> ServiceResponse {
    let db = db.lock().unwrap();
    let about = match db.prepare(
        "SELECT page, slug, heading, body FROM about_sections WHERE page = 'resume' ORDER BY sort_order ASC",
    ) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "slug": row.get::<_, String>(1)?,
                    "heading": row.get::<_, String>(2)?,
                    "body": row.get::<_, String>(3)?,
                }))
            }) {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect::<Vec<_>>(),
                Err(e) => {
                    error!("Failed to query resume about: {}", e);
                    return ServiceResponse::error(500, "database error");
                }
            }
        }
        Err(e) => {
            error!("Failed to prepare resume about: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    let jobs = match db.prepare(
        "SELECT slug, company, title, location, start_date, end_date, summary, tech_stack
         FROM jobs ORDER BY sort_order ASC",
    ) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                let tech_raw: Option<String> = row.get(7)?;
                Ok(serde_json::json!({
                    "slug": row.get::<_, String>(0)?,
                    "company": row.get::<_, String>(1)?,
                    "title": row.get::<_, String>(2)?,
                    "location": row.get::<_, Option<String>>(3)?,
                    "start_date": row.get::<_, String>(4)?,
                    "end_date": row.get::<_, Option<String>>(5)?,
                    "summary": row.get::<_, Option<String>>(6)?,
                    "tech_stack": tech_raw,
                }))
            }) {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect::<Vec<_>>(),
                Err(e) => {
                    error!("Failed to query resume jobs: {}", e);
                    return ServiceResponse::error(500, "database error");
                }
            }
        }
        Err(e) => {
            error!("Failed to prepare resume jobs: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    let data = serde_json::json!({
        "about": about,
        "jobs": jobs,
    });

    ServiceResponse::ok(data)
}
