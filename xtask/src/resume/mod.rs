//! Resume generation module
//!
//! Generates resume documents from SQLite data and uploads to S3.

use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod generate;
pub mod upload;

#[derive(ValueEnum, Clone, Debug)]
pub enum ResumeFormat {
    Functional,
    Chronological,
    All,
}

#[derive(Subcommand)]
pub enum ResumeAction {
    /// Generate resume markdown + convert to DOCX/PDF via pandoc
    Generate {
        /// Path to SQLite database
        #[arg(long, default_value = "deploy-baba.db")]
        db_path: PathBuf,
        /// Output directory for generated files
        #[arg(long, default_value = "target/resume")]
        output_dir: PathBuf,
        /// Resume format(s) to generate
        #[arg(long, default_value = "all")]
        format: ResumeFormat,
        /// Use LLM to polish the Professional Summary (reads ANTHROPIC_API_KEY or OPENAI_API_KEY env var)
        #[arg(long)]
        ai: bool,
        /// LLM provider to use (anthropic or openai)
        #[arg(long, default_value = "anthropic")]
        provider: String,
    },
    /// Upload generated resume files to S3
    Upload {
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
        /// Directory containing generated resume files
        #[arg(long, default_value = "target/resume")]
        output_dir: PathBuf,
    },
    /// Generate + upload (full pipeline)
    All {
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
        /// Path to SQLite database
        #[arg(long, default_value = "deploy-baba.db")]
        db_path: PathBuf,
        /// Output directory
        #[arg(long, default_value = "target/resume")]
        output_dir: PathBuf,
        /// Resume format(s) to generate
        #[arg(long, default_value = "all")]
        format: ResumeFormat,
        /// Use LLM to polish the Professional Summary (reads ANTHROPIC_API_KEY or OPENAI_API_KEY env var)
        #[arg(long)]
        ai: bool,
        /// LLM provider to use (anthropic or openai)
        #[arg(long, default_value = "anthropic")]
        provider: String,
    },
}

pub async fn execute(action: ResumeAction) -> anyhow::Result<()> {
    match action {
        ResumeAction::Generate {
            db_path,
            output_dir,
            format,
            ai,
            provider,
        } => {
            let (api_key, provider_id) = resolve_api_key(ai, &provider)?;
            generate::generate_resume(
                &db_path,
                &output_dir,
                &format,
                api_key,
                provider_id.as_deref(),
            )
            .await
        }
        ResumeAction::Upload {
            profile,
            output_dir,
        } => upload::upload_resume_files(&output_dir, profile).await,
        ResumeAction::All {
            profile,
            db_path,
            output_dir,
            format,
            ai,
            provider,
        } => {
            let (api_key, provider_id) = resolve_api_key(ai, &provider)?;
            generate::generate_resume(
                &db_path,
                &output_dir,
                &format,
                api_key,
                provider_id.as_deref(),
            )
            .await?;
            upload::upload_resume_files(&output_dir, profile).await
        }
    }
}

/// Reads ANTHROPIC_API_KEY or OPENAI_API_KEY from the environment when `--ai` is requested.
fn resolve_api_key(ai: bool, provider: &str) -> anyhow::Result<(Option<String>, Option<String>)> {
    if !ai {
        return Ok((None, None));
    }
    let key = match provider {
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "--ai with provider=anthropic requires ANTHROPIC_API_KEY to be set in the environment. \
                 Export the key or omit --ai to use the static summary."
            )
        })?,
        "openai" => std::env::var("OPENAI_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "--ai with provider=openai requires OPENAI_API_KEY to be set in the environment. \
                 Export the key or omit --ai to use the static summary."
            )
        })?,
        _ => return Err(anyhow::anyhow!("Unknown provider: {provider}")),
    };
    Ok((Some(key), Some(provider.to_string())))
}
