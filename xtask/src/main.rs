//! Xtask - Build automation for deploy-baba portfolio project
//!
//! This crate provides comprehensive task automation for building, testing,
//! and deploying the deploy-baba portfolio application.

use clap::{Parser, Subcommand};
use std::process::exit;

mod aws;
mod build;
mod cache;
mod coverage;
mod database;
mod deploy;
mod infra;
mod quality;
mod rag;
mod release;
mod resume;
mod secret;
mod test;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation for deploy-baba portfolio project")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build tasks (fmt, lint, compile)
    Build {
        #[command(subcommand)]
        action: build::BuildAction,
    },
    /// Test execution
    Test {
        #[command(subcommand)]
        action: test::TestAction,
    },
    /// Coverage analysis with per-crate floors
    Coverage {
        #[command(subcommand)]
        action: coverage::CoverageAction,
    },
    /// Quality gate enforcement
    Quality {
        #[command(subcommand)]
        action: quality::QualityAction,
    },
    /// AWS operations (profile validation, SSM parameters)
    Aws {
        #[command(subcommand)]
        action: aws::AwsAction,
    },
    /// Infrastructure management (Terraform)
    Infra {
        #[command(subcommand)]
        action: infra::InfraAction,
    },
    /// Deployment operations
    Deploy {
        #[command(subcommand)]
        action: deploy::DeployAction,
    },
    /// Agent cache management (status/refresh/clear)
    Cache {
        #[command(subcommand)]
        action: cache::CacheAction,
    },
    /// Database operations (backup/restore)
    Database {
        #[command(subcommand)]
        action: database::DatabaseAction,
    },
    /// RAG index management (ingest / query)
    Rag {
        #[command(subcommand)]
        action: rag::RagAction,
    },
    /// Resume generation and upload
    Resume {
        #[command(subcommand)]
        action: resume::ResumeAction,
    },
    /// Secrets Manager operations (put/get/list)
    Secret {
        #[command(subcommand)]
        action: secret::SecretAction,
    },
    /// Publish/release operations
    Publish {
        /// Target environment (dev, staging, prod)
        environment: String,
        /// Dry run - validate only
        #[arg(long)]
        dry_run: bool,
    },
    /// Release management (next version, tag, promote dev→prod)
    Release(release::ReleaseArgs),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Err(e) = run().await {
        eprintln!("error: {}", e);
        exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { action } => build::execute(action).await,
        Commands::Test { action } => test::execute(action).await,
        Commands::Coverage { action } => coverage::execute(action).await,
        Commands::Quality { action } => quality::execute(action).await,
        Commands::Aws { action } => aws::execute(action).await,
        Commands::Infra { action } => infra::execute(action).await,
        Commands::Deploy { action } => deploy::execute(action).await,
        Commands::Cache { action } => cache::execute(action).await,
        Commands::Database { action } => database::execute(action).await,
        Commands::Rag { action } => rag::execute(action).await,
        Commands::Resume { action } => resume::execute(action).await,
        Commands::Secret { action } => secret::execute(action).await,
        Commands::Publish {
            environment,
            dry_run,
        } => publish(environment, dry_run).await,
        Commands::Release(args) => tokio::task::spawn_blocking(|| release::run(args))
            .await
            .map_err(|e| anyhow::anyhow!("release task panicked: {e}"))?,
    }
}

async fn publish(environment: String, dry_run: bool) -> anyhow::Result<()> {
    println!(
        "📦 Publishing to {}{}",
        environment,
        if dry_run { " (dry run)" } else { "" }
    );

    // Validate quality gates
    quality::execute(quality::QualityAction::All).await?;

    // Build release artifacts
    build::execute(build::BuildAction::Compile {
        release: true,
        features: None,
    })
    .await?;

    // Deploy
    if !dry_run {
        deploy::execute(deploy::DeployAction::Lambda {
            function: None,
            profile: None,
        })
        .await?;
        println!("✅ Published successfully");
    } else {
        println!("✅ Publish validated (dry run)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_build() {
        let cli = Cli::try_parse_from(["xtask", "build", "compile"]).unwrap();
        matches!(cli.command, Commands::Build { .. });
    }

    #[test]
    fn test_cli_parse_test() {
        let cli = Cli::try_parse_from(["xtask", "test", "all"]).unwrap();
        matches!(cli.command, Commands::Test { .. });
    }

    #[test]
    fn test_cli_parse_quality() {
        let cli = Cli::try_parse_from(["xtask", "quality", "all"]).unwrap();
        matches!(cli.command, Commands::Quality { .. });
    }

    #[test]
    fn test_cli_parse_cache() {
        let cli = Cli::try_parse_from(["xtask", "cache", "status"]).unwrap();
        matches!(cli.command, Commands::Cache { .. });
    }

    #[test]
    fn test_cli_parse_publish_dry() {
        let cli = Cli::try_parse_from(["xtask", "publish", "dev", "--dry-run"]).unwrap();
        matches!(cli.command, Commands::Publish { .. });
    }

    #[test]
    fn test_cli_parse_publish_no_dry() {
        let cli = Cli::try_parse_from(["xtask", "publish", "prod"]).unwrap();
        matches!(cli.command, Commands::Publish { .. });
    }
}
