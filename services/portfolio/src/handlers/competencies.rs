use rusqlite::Connection;
use service_protocol::ServiceResponse;
use std::sync::{Arc, Mutex};
use tracing::error;

pub async fn list_competencies(db: &Arc<Mutex<Connection>>) -> ServiceResponse {
    let db = db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, slug, name, description, icon, sort_order
         FROM competencies ORDER BY sort_order ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to prepare competencies query: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    let competencies: Vec<serde_json::Value> = match stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "slug": row.get::<_, String>(1)?,
            "name": row.get::<_, String>(2)?,
            "description": row.get::<_, Option<String>>(3)?,
            "icon": row.get::<_, Option<String>>(4)?,
            "sort_order": row.get::<_, i64>(5)?,
        }))
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            error!("Failed to query competencies: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    ServiceResponse::ok(competencies)
}
