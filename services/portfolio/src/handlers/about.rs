use rusqlite::Connection;
use service_protocol::ServiceResponse;
use std::sync::{Arc, Mutex};
use tracing::error;

pub async fn list_about(db: &Arc<Mutex<Connection>>) -> ServiceResponse {
    let db = db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT page, slug, heading, body, sort_order
         FROM about_sections ORDER BY sort_order ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to prepare about query: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    let sections: Vec<serde_json::Value> = match stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "page": row.get::<_, String>(0)?,
            "slug": row.get::<_, String>(1)?,
            "heading": row.get::<_, String>(2)?,
            "body": row.get::<_, String>(3)?,
            "sort_order": row.get::<_, i64>(4)?,
        }))
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            error!("Failed to query about sections: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    ServiceResponse::ok(sections)
}
