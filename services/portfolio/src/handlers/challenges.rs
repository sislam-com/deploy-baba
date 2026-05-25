use rusqlite::Connection;
use service_protocol::ServiceResponse;
use std::sync::{Arc, Mutex};
use tracing::error;

pub async fn list_challenges(db: &Arc<Mutex<Connection>>) -> ServiceResponse {
    let db = db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT slug, title, description, short_description, tech_stack, category, url,
                problem, constraints, decisions, implementation, outcomes, metrics,
                related_job_slug, related_plan_module, related_adr, featured, sort_order
         FROM challenges ORDER BY sort_order ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to prepare challenges query: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    let challenges: Vec<serde_json::Value> = match stmt.query_map([], |row| {
        let featured: i64 = row.get(16)?;
        Ok(serde_json::json!({
            "slug": row.get::<_, String>(0)?,
            "title": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2)?,
            "short_description": row.get::<_, Option<String>>(3)?,
            "tech_stack": row.get::<_, Option<String>>(4)?,
            "category": row.get::<_, Option<String>>(5)?,
            "url": row.get::<_, Option<String>>(6)?,
            "problem": row.get::<_, Option<String>>(7)?,
            "constraints": row.get::<_, Option<String>>(8)?,
            "decisions": row.get::<_, Option<String>>(9)?,
            "implementation": row.get::<_, Option<String>>(10)?,
            "outcomes": row.get::<_, Option<String>>(11)?,
            "metrics": row.get::<_, Option<String>>(12)?,
            "related_job_slug": row.get::<_, Option<String>>(13)?,
            "related_plan_module": row.get::<_, Option<String>>(14)?,
            "related_adr": row.get::<_, Option<String>>(15)?,
            "featured": featured != 0,
            "sort_order": row.get::<_, i64>(17)?,
        }))
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            error!("Failed to query challenges: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    ServiceResponse::ok(challenges)
}

pub async fn get_challenge(db: &Arc<Mutex<Connection>>, slug: &str) -> ServiceResponse {
    let db = db.lock().unwrap();
    let challenge: Option<serde_json::Value> = match db.query_row(
        "SELECT slug, title, description, short_description, tech_stack, category, url,
                problem, constraints, decisions, implementation, outcomes, metrics,
                related_job_slug, related_plan_module, related_adr, featured
         FROM challenges WHERE slug = ?1",
        [slug],
        |row| {
            let featured: i64 = row.get(16)?;
            Ok(serde_json::json!({
                "slug": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "short_description": row.get::<_, Option<String>>(3)?,
                "tech_stack": row.get::<_, Option<String>>(4)?,
                "category": row.get::<_, Option<String>>(5)?,
                "url": row.get::<_, Option<String>>(6)?,
                "problem": row.get::<_, Option<String>>(7)?,
                "constraints": row.get::<_, Option<String>>(8)?,
                "decisions": row.get::<_, Option<String>>(9)?,
                "implementation": row.get::<_, Option<String>>(10)?,
                "outcomes": row.get::<_, Option<String>>(11)?,
                "metrics": row.get::<_, Option<String>>(12)?,
                "related_job_slug": row.get::<_, Option<String>>(13)?,
                "related_plan_module": row.get::<_, Option<String>>(14)?,
                "related_adr": row.get::<_, Option<String>>(15)?,
                "featured": featured != 0,
            }))
        },
    ) {
        Ok(c) => Some(c),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => {
            error!("Failed to query challenge: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    match challenge {
        Some(c) => ServiceResponse::ok(c),
        None => ServiceResponse::error(404, "challenge not found"),
    }
}
