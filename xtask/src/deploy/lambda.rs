//! AWS Lambda deployment
//!
//! Builds the deploy-baba-ui binary for aarch64 (ARM64) using cargo-lambda,
//! packages it as lambda.zip, and uploads it to the Lambda function.

use aws_sdk_lambda::Client as LambdaClient;
use std::process::Command;

const DEFAULT_PACKAGE: &str = "deploy-baba-ui";
const TARGET: &str = "aarch64-unknown-linux-gnu";

pub async fn deploy(
    function: String,
    package: Option<String>,
    zip_path_override: Option<String>,
    profile: Option<String>,
) -> anyhow::Result<()> {
    println!("🚀 Deploying to AWS Lambda: {}", function);

    let zip_path = if let Some(ref path) = zip_path_override {
        println!("   Using pre-built zip: {}", path);
        if !std::path::Path::new(path).exists() {
            return Err(anyhow::anyhow!("Zip file not found: {}", path));
        }
        path.clone()
    } else {
        let pkg = package.unwrap_or_else(|| DEFAULT_PACKAGE.to_string());
        let zip = format!("infra/build/{}.zip", pkg);

        println!("   Building {} ({})...", pkg, TARGET);
        let status = Command::new("cargo")
            .args([
                "lambda",
                "build",
                "--release",
                "--package",
                &pkg,
                "--target",
                TARGET,
            ])
            .status()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to run cargo lambda: {} (is cargo-lambda installed?)",
                    e
                )
            })?;

        if !status.success() {
            return Err(anyhow::anyhow!("cargo lambda build failed"));
        }

        println!("   Packaging {}...", zip);
        std::fs::create_dir_all("infra/build")
            .map_err(|e| anyhow::anyhow!("Failed to create infra/build: {}", e))?;

        let _ = std::fs::remove_file(&zip);

        let bootstrap_path = format!("target/lambda/{}/bootstrap", pkg);
        let status = Command::new("zip")
            .args(["-j", &zip, &bootstrap_path])
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run zip: {}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to create deployment package"));
        }

        zip
    };

    println!("   Uploading to Lambda function: {}...", function);
    let config = crate::aws::create_aws_config(profile).await?;
    let client = LambdaClient::new(&config);

    let zip_data = std::fs::read(&zip_path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", zip_path, e))?;

    client
        .update_function_code()
        .function_name(&function)
        .zip_file(aws_sdk_lambda::primitives::Blob::new(zip_data))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update Lambda function: {}", e))?;

    println!("✅ Lambda function deployed: {}", function);
    Ok(())
}
