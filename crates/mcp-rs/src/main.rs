use mcp_rs::{build_server, config, initialize_workspace};

fn main() {
    let runtime = RuntimeOptions::from_args(std::env::args().skip(1));

    let config = config::load_config_from(runtime.config_path.clone());
    let workspace_root = initialize_workspace(&config);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| config.get_tracing_filter().into());

    match config.logging.format.to_lowercase().as_str() {
        "json" => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(filter)
                .with_target(config.logging.include_source)
                .init();
        }
        "pretty" => {
            tracing_subscriber::fmt()
                .pretty()
                .with_env_filter(filter)
                .with_target(config.logging.include_source)
                .init();
        }
        _ => {
            tracing_subscriber::fmt()
                .compact()
                .with_env_filter(filter)
                .with_target(config.logging.include_source)
                .init();
        }
    }

    tracing::info!("MCP-RS starting with configuration loaded");
    tracing::info!(workspace_root = %workspace_root.display(), "Workspace root initialized");

    let transport = runtime
        .transport
        .or_else(|| config.server.transport.clone())
        .unwrap_or_else(|| "stdio".to_string());
    let server = build_server(&config);

    match transport.as_str() {
        "http" => {
            let addr = runtime
                .http_addr
                .or_else(|| config.server.http_addr.clone())
                .unwrap_or_else(|| "127.0.0.1:8080".to_string());
            if let Err(e) = server.run_http(&addr) {
                tracing::error!(error = %e, addr = addr, "HTTP transport failed");
                std::process::exit(1);
            }
        }
        _ => server.run_stdio(),
    }
}

#[derive(Debug, Default)]
struct RuntimeOptions {
    config_path: Option<std::path::PathBuf>,
    transport: Option<String>,
    http_addr: Option<String>,
}

impl RuntimeOptions {
    fn from_args<I>(args: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut opts = RuntimeOptions::default();
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--config" => {
                    if let Some(value) = iter.next() {
                        opts.config_path = Some(std::path::PathBuf::from(value));
                    }
                }
                "--transport" => {
                    if let Some(value) = iter.next() {
                        opts.transport = Some(value);
                    }
                }
                "--http-addr" => {
                    if let Some(value) = iter.next() {
                        opts.http_addr = Some(value);
                    }
                }
                "--help" | "-h" => {
                    println!(
                        "Usage: mcp-rs [--config PATH] [--transport stdio|http] [--http-addr HOST:PORT]"
                    );
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        opts
    }
}
