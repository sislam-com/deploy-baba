//! Artifact promotion: copy deployed code from dev → prod without rebuilding.
//!
//! Reads deploy-config from both environments via Secrets Manager,
//! copies Lambda code zips and S3 SPA assets, then smoke-tests prod.

use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_secretsmanager::Client as SmClient;
use std::time::Duration;

use super::spa::{invalidate_cloudfront, smoke_test, wait_lambda_active};

struct EnvConfig {
    spa_bucket: String,
    cloudfront_id: String,
    ui_fn_name: String,
    agent_fn_name: String,
    email_fn_name: String,
    llm_proxy_fn_name: String,
    auth_fn_name: String,
    portfolio_fn_name: String,
    admin_fn_name: String,
    contact_fn_name: String,
    rag_fn_name: String,
    mcp_gateway_fn_name: String,
    fn_url: String,
}

impl EnvConfig {
    async fn from_secrets_manager(
        config: &aws_config::SdkConfig,
        env: &str,
    ) -> anyhow::Result<Self> {
        let client = SmClient::new(config);
        let secret_name = format!("deploy-baba/{env}/deploy-config");
        let resp = client
            .get_secret_value()
            .secret_id(&secret_name)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read {secret_name}: {e}"))?;
        let raw = resp
            .secret_string()
            .ok_or_else(|| anyhow::anyhow!("{secret_name} has no string value"))?;
        let v: serde_json::Value = serde_json::from_str(raw)?;
        Ok(Self {
            spa_bucket: v["spa_bucket"].as_str().unwrap_or_default().to_owned(),
            cloudfront_id: v["cloudfront_id"].as_str().unwrap_or_default().to_owned(),
            ui_fn_name: v["ui_fn_name"].as_str().unwrap_or_default().to_owned(),
            agent_fn_name: v["agent_fn_name"].as_str().unwrap_or_default().to_owned(),
            email_fn_name: v["email_fn_name"].as_str().unwrap_or_default().to_owned(),
            llm_proxy_fn_name: v["llm_proxy_fn_name"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            auth_fn_name: v["auth_fn_name"].as_str().unwrap_or_default().to_owned(),
            portfolio_fn_name: v["portfolio_fn_name"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            admin_fn_name: v["admin_fn_name"].as_str().unwrap_or_default().to_owned(),
            contact_fn_name: v["contact_fn_name"].as_str().unwrap_or_default().to_owned(),
            rag_fn_name: v["rag_fn_name"].as_str().unwrap_or_default().to_owned(),
            mcp_gateway_fn_name: v["mcp_gateway_fn_name"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            fn_url: v["fn_url"].as_str().unwrap_or_default().to_owned(),
        })
    }
}

pub async fn promote(profile: Option<String>, skip_tag: bool) -> anyhow::Result<()> {
    println!("Promoting dev → prod (artifact copy, no rebuild)");

    let aws_config = crate::aws::create_aws_config(profile).await?;
    let dev = EnvConfig::from_secrets_manager(&aws_config, "dev").await?;
    let prod = EnvConfig::from_secrets_manager(&aws_config, "prod").await?;

    let lambda_client = LambdaClient::new(&aws_config);
    let s3_client = S3Client::new(&aws_config);

    // 1. Promote Lambda functions (zip-based services only — the PDF Lambda
    // is a Docker/ECR image and is promoted separately via `just pdf-deploy`)
    promote_lambda(&lambda_client, &dev.ui_fn_name, &prod.ui_fn_name).await?;
    promote_lambda(&lambda_client, &dev.agent_fn_name, &prod.agent_fn_name).await?;
    promote_lambda(&lambda_client, &dev.email_fn_name, &prod.email_fn_name).await?;
    promote_lambda(
        &lambda_client,
        &dev.llm_proxy_fn_name,
        &prod.llm_proxy_fn_name,
    )
    .await?;
    promote_lambda(&lambda_client, &dev.auth_fn_name, &prod.auth_fn_name).await?;
    promote_lambda(
        &lambda_client,
        &dev.portfolio_fn_name,
        &prod.portfolio_fn_name,
    )
    .await?;
    promote_lambda(&lambda_client, &dev.admin_fn_name, &prod.admin_fn_name).await?;
    promote_lambda(&lambda_client, &dev.contact_fn_name, &prod.contact_fn_name).await?;
    promote_lambda(&lambda_client, &dev.rag_fn_name, &prod.rag_fn_name).await?;
    promote_lambda(
        &lambda_client,
        &dev.mcp_gateway_fn_name,
        &prod.mcp_gateway_fn_name,
    )
    .await?;

    // 2. Promote SPA (S3 server-side copy)
    promote_spa(&s3_client, &dev.spa_bucket, &prod.spa_bucket).await?;

    // 3. CloudFront invalidation
    if !prod.cloudfront_id.is_empty() {
        invalidate_cloudfront(&prod.cloudfront_id)?;
    }

    // 4. Smoke test prod
    smoke_test(&prod.fn_url).await?;

    // 5. Create release tag
    if !skip_tag {
        create_release_tag()?;
    }

    println!("Promote complete: prod is now running dev artifacts");
    Ok(())
}

async fn promote_lambda(client: &LambdaClient, dev_fn: &str, prod_fn: &str) -> anyhow::Result<()> {
    println!("   Promoting Lambda: {} → {}", dev_fn, prod_fn);

    let resp = client
        .get_function()
        .function_name(dev_fn)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get dev function {dev_fn}: {e}"))?;

    let code_location = resp
        .code()
        .and_then(|c| c.location())
        .ok_or_else(|| anyhow::anyhow!("No code location for {dev_fn}"))?;

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    let zip_bytes = http_client
        .get(code_location)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to download {dev_fn} code: {e}"))?
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read {dev_fn} code bytes: {e}"))?;

    println!("   Downloaded {} bytes from {}", zip_bytes.len(), dev_fn);

    client
        .update_function_code()
        .function_name(prod_fn)
        .zip_file(aws_sdk_lambda::primitives::Blob::new(zip_bytes.to_vec()))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update {prod_fn}: {e}"))?;

    wait_lambda_active(client, prod_fn).await?;
    println!("   {} updated and active", prod_fn);
    Ok(())
}

async fn promote_spa(client: &S3Client, dev_bucket: &str, prod_bucket: &str) -> anyhow::Result<()> {
    println!(
        "   Promoting SPA: s3://{} → s3://{}",
        dev_bucket, prod_bucket
    );

    // List all objects in dev bucket
    let mut continuation_token: Option<String> = None;
    let mut dev_keys: Vec<String> = Vec::new();

    loop {
        let mut req = client.list_objects_v2().bucket(dev_bucket);
        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list {dev_bucket}: {e}"))?;

        for obj in resp.contents() {
            if let Some(key) = obj.key() {
                dev_keys.push(key.to_owned());
            }
        }

        if resp.is_truncated() == Some(true) {
            continuation_token = resp.next_continuation_token().map(|s| s.to_owned());
        } else {
            break;
        }
    }

    println!("   Copying {} objects", dev_keys.len());

    for key in &dev_keys {
        let copy_source = format!("{}/{}", dev_bucket, key);

        // Preserve cache-control: index.html gets no-cache, everything else immutable
        let cache_control = if key == "index.html" {
            "no-cache,no-store,must-revalidate"
        } else {
            "public,max-age=31536000,immutable"
        };

        client
            .copy_object()
            .bucket(prod_bucket)
            .key(key)
            .copy_source(&copy_source)
            .cache_control(cache_control)
            .metadata_directive(aws_sdk_s3::types::MetadataDirective::Replace)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to copy {key}: {e}"))?;
    }

    // Delete objects in prod that don't exist in dev
    let mut prod_continuation: Option<String> = None;
    let mut prod_keys: Vec<String> = Vec::new();

    loop {
        let mut req = client.list_objects_v2().bucket(prod_bucket);
        if let Some(token) = &prod_continuation {
            req = req.continuation_token(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list {prod_bucket}: {e}"))?;

        for obj in resp.contents() {
            if let Some(key) = obj.key() {
                prod_keys.push(key.to_owned());
            }
        }

        if resp.is_truncated() == Some(true) {
            prod_continuation = resp.next_continuation_token().map(|s| s.to_owned());
        } else {
            break;
        }
    }

    let dev_set: std::collections::HashSet<&str> = dev_keys.iter().map(|s| s.as_str()).collect();
    let stale: Vec<&String> = prod_keys
        .iter()
        .filter(|k| !dev_set.contains(k.as_str()))
        .collect();

    if !stale.is_empty() {
        println!("   Deleting {} stale objects from prod", stale.len());
        for key in stale {
            client
                .delete_object()
                .bucket(prod_bucket)
                .key(key)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to delete {key} from {prod_bucket}: {e}"))?;
        }
    }

    println!("   SPA promotion complete");
    Ok(())
}

fn create_release_tag() -> anyhow::Result<()> {
    use crate::release::git;

    let latest_dev = git::last_dev_tag()?.ok_or_else(|| {
        anyhow::anyhow!("no dev-v* tag found — run `release tag --kind dev` first")
    })?;

    let ver = latest_dev
        .strip_prefix("dev-v")
        .ok_or_else(|| anyhow::anyhow!("unexpected dev tag format: {latest_dev}"))?;

    let prod_tag = format!("v{ver}");

    if git::tag_exists(&prod_tag)? {
        println!("   Tag {prod_tag} already exists — skipping");
        return Ok(());
    }

    let dev_sha = git::tag_sha(&latest_dev)?;
    let body = format!("Promote {latest_dev} → {prod_tag} (artifact copy)");
    git::create_annotated_tag_at(&prod_tag, &body, &dev_sha)?;
    git::push_tag(&prod_tag)?;
    println!("   Created and pushed tag {prod_tag}");
    Ok(())
}
