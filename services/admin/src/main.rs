use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use service_protocol::{ServiceRequest, ServiceResponse};
use std::sync::{Arc, Mutex};
use tracing::info;

mod handlers;
mod router;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .json()
        .init();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "deploy-baba.db".to_string());
    info!(db = %db_path, "admin service starting");

    let conn = rusqlite::Connection::open(&db_path)?;
    let migration_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
        .unwrap_or(0);
    if migration_count == 0 {
        tracing::warn!(
            "_migrations table empty or missing — UI service may not have initialized the DB yet"
        );
    } else {
        info!(migrations = migration_count, "DB schema verified");
    }
    let db = Arc::new(Mutex::new(conn));

    run(service_fn(move |event: LambdaEvent<ServiceRequest>| {
        let db = db.clone();
        async move {
            let req = event.payload;
            info!(method = %req.method, path = %req.path, "handling request");
            let resp = router::route(&db, req).await;
            Ok::<ServiceResponse, Error>(resp)
        }
    }))
    .await
}
