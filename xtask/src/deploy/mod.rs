//! Deployment module
//!
//! Orchestrates deployments to Lambda, ECS, and other targets

use clap::Subcommand;

pub mod docker;
pub mod ecr;
pub mod ecs;
pub mod lambda;

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
}

pub async fn execute(action: DeployAction) -> anyhow::Result<()> {
    match action {
        DeployAction::Lambda { function, profile } => lambda::deploy(function, profile).await,
        DeployAction::Ecs { cluster, service } => ecs::deploy(cluster, service).await,
        DeployAction::Docker { platform, tag } => docker::build(&platform, tag).await,
        DeployAction::Push { image, profile } => ecr::push(&image, profile).await,
    }
}
