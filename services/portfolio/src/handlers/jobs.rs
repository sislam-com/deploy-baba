use rusqlite::Connection;
use service_protocol::ServiceResponse;
use std::sync::{Arc, Mutex};
use tracing::error;

pub async fn list_jobs(db: &Arc<Mutex<Connection>>) -> ServiceResponse {
    let db = db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order
         FROM jobs ORDER BY sort_order ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to prepare jobs query: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    let jobs: Vec<serde_json::Value> = match stmt.query_map([], |row| {
        let tech_raw: Option<String> = row.get(8)?;
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "slug": row.get::<_, String>(1)?,
            "company": row.get::<_, String>(2)?,
            "title": row.get::<_, String>(3)?,
            "location": row.get::<_, Option<String>>(4)?,
            "start_date": row.get::<_, String>(5)?,
            "end_date": row.get::<_, Option<String>>(6)?,
            "summary": row.get::<_, Option<String>>(7)?,
            "tech_stack": tech_raw,
            "sort_order": row.get::<_, i64>(9)?,
        }))
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            error!("Failed to query jobs: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    ServiceResponse::ok(jobs)
}

pub async fn get_job(db: &Arc<Mutex<Connection>>, slug: &str) -> ServiceResponse {
    let db = db.lock().unwrap();
    let job: Option<serde_json::Value> = match db.query_row(
        "SELECT id, slug, company, title, location, start_date, end_date, summary, tech_stack
         FROM jobs WHERE slug = ?1",
        [slug],
        |row| {
            let tech_raw: Option<String> = row.get(8)?;
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "slug": row.get::<_, String>(1)?,
                "company": row.get::<_, String>(2)?,
                "title": row.get::<_, String>(3)?,
                "location": row.get::<_, Option<String>>(4)?,
                "start_date": row.get::<_, String>(5)?,
                "end_date": row.get::<_, Option<String>>(6)?,
                "summary": row.get::<_, Option<String>>(7)?,
                "tech_stack": tech_raw,
            }))
        },
    ) {
        Ok(j) => Some(j),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => {
            error!("Failed to query job: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    match job {
        Some(j) => ServiceResponse::ok(j),
        None => ServiceResponse::error(404, "job not found"),
    }
}
