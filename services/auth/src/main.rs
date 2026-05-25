use lambda_http::{run, service_fn, Error, Request};
use std::sync::Arc;
use tower::ServiceExt;

mod cognito;
mod error;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .json()
        .init();

    let state = Arc::new(AppState::from_env().await);
    tracing::info!("auth service ready (dev_mode={})", state.config.dev_mode);

    let app = routes::router(state.clone());

    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        tracing::info!("starting as AWS Lambda function");
        run(service_fn(move |req: Request| {
            let app = app.clone();
            async move {
                app.oneshot(req)
                    .await
                    .map_err(|e| Error::from(e.to_string()))
            }
        }))
        .await?;
    } else {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await?;
        tracing::info!("http://localhost:3002");
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await?;
    }

    Ok(())
}
