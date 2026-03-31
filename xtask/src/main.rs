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
mod resume;
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
    /// Resume generation and upload
    Resume {
        #[command(subcommand)]
        action: resume::ResumeAction,
    },
    /// Publish/release operations
    Publish {
        /// Target environment (dev, staging, prod)
        environment: String,
        /// Dry run - validate only
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() {
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
        Commands::Resume { action } => resume::execute(action).await,
        Commands::Publish {
            environment,
            dry_run,
        } => publish(environment, dry_run).await,
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
