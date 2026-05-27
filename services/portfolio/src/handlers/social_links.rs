use rusqlite::Connection;
use service_protocol::ServiceResponse;
use std::sync::{Arc, Mutex};
use tracing::error;

pub async fn list_social_links(db: &Arc<Mutex<Connection>>) -> ServiceResponse {
    let db = db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT url, label, icon, visible, sort_order
         FROM social_links WHERE visible = 1 ORDER BY sort_order ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to prepare social links query: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    let links: Vec<serde_json::Value> = match stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "url": row.get::<_, String>(0)?,
            "label": row.get::<_, String>(1)?,
            "icon": row.get::<_, Option<String>>(2)?,
            "visible": row.get::<_, i64>(3)? != 0,
            "sort_order": row.get::<_, i64>(4)?,
        }))
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            error!("Failed to query social links: {}", e);
            return ServiceResponse::error(500, "database error");
        }
    };

    ServiceResponse::ok(links)
}
