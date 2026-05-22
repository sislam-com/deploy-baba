//! Infrastructure management module
//!
//! Wraps OpenTofu operations and bootstrap procedures.
//!
//! `--workspace` selects the OpenTofu workspace (default → prod, dev → dev).
//! `--aws-profile` sets AWS credentials (independent of workspace).

use clap::Subcommand;

pub mod bootstrap;
pub mod tofu;

#[derive(Subcommand)]
pub enum InfraAction {
    /// Initialize OpenTofu
    Init {
        #[arg(long)]
        dir: Option<String>,
        /// AWS profile for credentials
        #[arg(long)]
        aws_profile: Option<String>,
    },
    /// Plan infrastructure changes
    Plan {
        #[arg(long)]
        dir: Option<String>,
        /// Target workspace (default → prod, dev → dev-named resources)
        #[arg(long)]
        workspace: Option<String>,
        /// AWS profile for credentials
        #[arg(long)]
        aws_profile: Option<String>,
    },
    /// Apply infrastructure changes
    Apply {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        auto_approve: bool,
        /// Target workspace (default → prod, dev → dev-named resources)
        #[arg(long)]
        workspace: Option<String>,
        /// AWS profile for credentials
        #[arg(long)]
        aws_profile: Option<String>,
    },
    /// Destroy infrastructure
    Destroy {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        auto_approve: bool,
        /// Target workspace (default → prod, dev → dev-named resources)
        #[arg(long)]
        workspace: Option<String>,
        /// AWS profile for credentials
        #[arg(long)]
        aws_profile: Option<String>,
    },
    /// Get OpenTofu output values
    Output {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        dir: Option<String>,
        /// Target workspace (default → prod, dev → dev-named resources)
        #[arg(long)]
        workspace: Option<String>,
        /// AWS profile for credentials
        #[arg(long)]
        aws_profile: Option<String>,
    },
    /// Bootstrap AWS account (create state bucket + DynamoDB lock table, run tofu init)
    Bootstrap {
        /// AWS profile for credentials
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        region: Option<String>,
    },
}

pub async fn execute(action: InfraAction) -> anyhow::Result<()> {
    match action {
        InfraAction::Init { dir, aws_profile } => tofu::run_tofu_init(dir, aws_profile).await,
        InfraAction::Plan {
            dir,
            workspace,
            aws_profile,
        } => tofu::run_tofu_plan(dir, workspace, aws_profile).await,
        InfraAction::Apply {
            dir,
            auto_approve,
            workspace,
            aws_profile,
        } => tofu::run_tofu_apply(dir, auto_approve, workspace, aws_profile).await,
        InfraAction::Destroy {
            dir,
            auto_approve,
            workspace,
            aws_profile,
        } => tofu::run_tofu_destroy(dir, auto_approve, workspace, aws_profile).await,
        InfraAction::Output {
            name,
            dir,
            workspace,
            aws_profile,
        } => tofu::run_tofu_output(name, dir, workspace, aws_profile).await,
        InfraAction::Bootstrap { profile, region } => {
            bootstrap::bootstrap_account(profile, region).await
        }
    }
}
