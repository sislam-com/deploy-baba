//! Deployment module
//!
//! Orchestrates deployments to Lambda, ECS, and other targets

use crate::rag;
use clap::Subcommand;

pub mod docker;
pub mod ecr;
pub mod ecs;
pub mod lambda;
pub mod spa;

#[derive(Subcommand)]
pub enum DeployAction {
    /// Deploy to AWS Lambda
    Lambda {
        /// Function name (default: deploy-baba-prod)
        #[arg(long)]
        function: Option<String>,
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
    },
    /// Deploy to Amazon ECS
    Ecs {
        /// Cluster name
        #[arg(long)]
        cluster: Option<String>,
        /// Service name
        #[arg(long)]
        service: Option<String>,
    },
    /// Build Docker image
    Docker {
        /// Platform to build for (e.g., linux/arm64)
        #[arg(long, default_value = "linux/arm64")]
        platform: String,
        /// Image tag
        #[arg(long)]
        tag: Option<String>,
    },
    /// Push to Amazon ECR
    Push {
        /// Full ECR image URI (e.g. 123456789012.dkr.ecr.us-east-1.amazonaws.com/repo:tag).
        /// Defaults to deploy-baba-ui:latest (local tag built by `just build-image`).
        #[arg(long, default_value = "deploy-baba-ui:latest")]
        image: String,
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
    },
    /// Wait for Lambda function to become active after a code update
    Wait {
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
        /// Lambda function name (reads UI_FN_NAME env var if omitted)
        #[arg(long)]
        function: Option<String>,
    },
    /// Build SPA, sync to S3, invalidate CloudFront, smoke /health
    Spa {
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
        /// Git SHA to use as the S3 key prefix (defaults to HEAD)
        #[arg(long)]
        sha: Option<String>,
        /// Skip the Lambda wait step (useful when Lambda is already settled)
        #[arg(long)]
        skip_wait: bool,
        /// Environment to read deploy-config from (default: prod)
        #[arg(long, default_value = "prod")]
        env: String,
    },
}

pub async fn execute(action: DeployAction) -> anyhow::Result<()> {
    let result = match action {
        DeployAction::Lambda { function, profile } => lambda::deploy(function, profile).await,
        DeployAction::Ecs { cluster, service } => ecs::deploy(cluster, service).await,
        DeployAction::Docker { platform, tag } => docker::build(&platform, tag).await,
        DeployAction::Push { image, profile } => ecr::push(&image, profile).await,
        DeployAction::Wait { profile, function } => {
            let fn_name = function
                .or_else(|| std::env::var("UI_FN_NAME").ok())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Lambda function name required: pass --function or set UI_FN_NAME"
                    )
                })?;
            let aws_config = crate::aws::create_aws_config(profile).await?;
            let client = aws_sdk_lambda::Client::new(&aws_config);
            spa::wait_lambda_active(&client, &fn_name).await
        }
        DeployAction::Spa {
            profile,
            sha,
            skip_wait,
            env,
        } => {
            let sha = sha.or_else(|| git_head_sha().ok()).ok_or_else(|| {
                anyhow::anyhow!("Could not determine git SHA; pass --sha explicitly")
            })?;
            // Try Secrets Manager first (CI + standard deploys); fall back to env vars for local dev.
            let env_cfg =
                match spa::SpaEnvConfig::from_secrets_manager(profile.as_deref(), &env).await {
                    Ok(cfg) => cfg,
                    Err(sm_err) => {
                        println!("   SM read failed ({sm_err}); falling back to env vars");
                        spa::SpaEnvConfig::from_env()?
                    }
                };
            spa::deploy_spa(profile, env_cfg, &sha, skip_wait).await
        }
    };

    if let Err(ref e) = result {
        println!("❌ Deployment failed: {}", e);
        println!("🔍 Querying RAG for diagnosis...");
        match rag::diagnose_failure(&e.to_string()).await {
            Ok(suggestions) => {
                println!("💡 RAG diagnosis suggestions:");
                for suggestion in suggestions {
                    println!("  • {}", suggestion);
                }
            }
            Err(diag_err) => {
                println!("⚠️  RAG diagnosis failed: {}", diag_err);
            }
        }
    }

    result
}

fn git_head_sha() -> anyhow::Result<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !out.status.success() {
        return Err(anyhow::anyhow!("git rev-parse HEAD failed"));
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}
