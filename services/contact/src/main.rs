use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use service_protocol::{ServiceRequest, ServiceResponse};
use std::sync::Arc;
use tracing::info;

mod handlers;
mod router;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .json()
        .init();

    info!("contact service starting");

    let config = aws_config::load_from_env().await;
    let lambda_client = Arc::new(aws_sdk_lambda::Client::new(&config));

    run(service_fn(move |event: LambdaEvent<ServiceRequest>| {
        let lambda_client = lambda_client.clone();
        async move {
            let req = event.payload;
            info!(method = %req.method, path = %req.path, "handling request");
            let resp = router::route(&lambda_client, req).await;
            Ok::<ServiceResponse, Error>(resp)
        }
    }))
    .await
}
