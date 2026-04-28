//! AWS Secrets Manager operations — put/get/list managed secrets

use aws_sdk_secretsmanager::Client as SmClient;
use clap::Subcommand;

const SECRET_PREFIX: &str = "deploy-baba/prod/";

/// Known secret short-names (enforced on Put to prevent typos).
const KNOWN_SECRETS: &[&str] = &["pow-secret", "cognito-temp-password", "anthropic-api-key"];

fn full_id(name: &str) -> String {
    format!("{}{}", SECRET_PREFIX, name)
}

#[derive(Subcommand)]
pub enum SecretAction {
    /// Write (or rotate) a secret value in AWS Secrets Manager
    Put {
        /// Short name, e.g. pow-secret
        name: String,
        /// Secret value
        value: String,
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
    },
    /// Read a secret value from AWS Secrets Manager
    Get {
        /// Short name, e.g. pow-secret
        name: String,
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
    },
    /// List all managed secrets under the deploy-baba/prod/ prefix
    List {
        /// AWS profile
        #[arg(long)]
        profile: Option<String>,
    },
}

pub async fn execute(action: SecretAction) -> anyhow::Result<()> {
    match action {
        SecretAction::Put {
            name,
            value,
            profile,
        } => put(&name, &value, profile).await,
        SecretAction::Get { name, profile } => get(&name, profile).await,
        SecretAction::List { profile } => list(profile).await,
    }
}

async fn put(name: &str, value: &str, profile: Option<String>) -> anyhow::Result<()> {
    if !KNOWN_SECRETS.contains(&name) {
        anyhow::bail!(
            "Unknown secret '{}'. Known secrets: {}",
            name,
            KNOWN_SECRETS.join(", ")
        );
    }

    let id = full_id(name);
    println!("Writing secret: {}", id);

    let config = crate::aws::create_aws_config(profile).await?;
    let client = SmClient::new(&config);

    client
        .put_secret_value()
        .secret_id(&id)
        .secret_string(value)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to put secret '{}': {}", id, e))?;

    println!("Secret stored: {}", id);
    Ok(())
}

async fn get(name: &str, profile: Option<String>) -> anyhow::Result<()> {
    let id = full_id(name);
    println!("Reading secret: {}", id);

    let config = crate::aws::create_aws_config(profile).await?;
    let client = SmClient::new(&config);

    let resp = client
        .get_secret_value()
        .secret_id(&id)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get secret '{}': {}", id, e))?;

    let secret = resp
        .secret_string()
        .ok_or_else(|| anyhow::anyhow!("Secret '{}' has no string value", id))?;

    println!("{}", secret);
    Ok(())
}

async fn list(profile: Option<String>) -> anyhow::Result<()> {
    println!("Listing secrets under prefix: {}", SECRET_PREFIX);

    let config = crate::aws::create_aws_config(profile).await?;
    let client = SmClient::new(&config);

    let filter = aws_sdk_secretsmanager::types::Filter::builder()
        .key(aws_sdk_secretsmanager::types::FilterNameStringType::Name)
        .values(SECRET_PREFIX)
        .build();

    let resp = client
        .list_secrets()
        .filters(filter)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list secrets: {}", e))?;

    let secrets = resp.secret_list();
    if secrets.is_empty() {
        println!("No secrets found.");
    } else {
        for s in secrets {
            println!("  {}", s.name().unwrap_or("<unnamed>"));
        }
    }

    Ok(())
}
