use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const WAIT_TIMEOUT: Duration = Duration::from_secs(120);

/// Env-var or infra-output config for one environment.
pub struct SpaEnvConfig {
    pub spa_bucket: String,
    pub fn_name: String,
    pub fn_url: String,
}

impl SpaEnvConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let spa_bucket = std::env::var("SPA_BUCKET")
            .map_err(|_| anyhow::anyhow!("SPA_BUCKET env var not set (run: just infra-output or set manually)"))?;
        let fn_name = std::env::var("UI_FN_NAME")
            .map_err(|_| anyhow::anyhow!("UI_FN_NAME env var not set"))?;
        let fn_url = std::env::var("FN_URL")
            .map_err(|_| anyhow::anyhow!("FN_URL env var not set (include trailing slash)"))?;
        Ok(Self { spa_bucket, fn_name, fn_url })
    }
}

/// Step 2: poll until Lambda LastUpdateStatus == Successful.
pub async fn wait_lambda_active(
    client: &LambdaClient,
    fn_name: &str,
) -> anyhow::Result<()> {
    println!("   Waiting for Lambda '{}' to become active...", fn_name);
    let deadline = Instant::now() + WAIT_TIMEOUT;
    loop {
        let resp = client
            .get_function()
            .function_name(fn_name)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("get_function failed: {e}"))?;

        if let Some(config) = resp.configuration() {
            use aws_sdk_lambda::types::LastUpdateStatus;
            match config.last_update_status() {
                Some(LastUpdateStatus::Successful) => {
                    println!("   Lambda active.");
                    return Ok(());
                }
                Some(LastUpdateStatus::Failed) => {
                    return Err(anyhow::anyhow!("Lambda update failed — check CloudWatch logs"));
                }
                _ => {}
            }
        }
        if Instant::now() >= deadline {
            return Err(anyhow::anyhow!("Timed out waiting for Lambda to become active"));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Step 3: run `pnpm --dir web run build`, streaming output.
pub fn build_spa() -> anyhow::Result<()> {
    println!("   Building SPA (pnpm --dir web run build)...");
    let status = Command::new("pnpm")
        .args(["--dir", "web", "run", "build"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run pnpm: {e} (is pnpm installed?)"))?;
    if !status.success() {
        return Err(anyhow::anyhow!("pnpm build failed"));
    }
    println!("   SPA build complete.");
    Ok(())
}

/// Step 4: walk web/dist/, upload to s3://<bucket>/<sha>/ with correct Cache-Control.
pub async fn sync_to_s3(
    client: &S3Client,
    bucket: &str,
    sha: &str,
) -> anyhow::Result<(usize, u64)> {
    let dist = Path::new("web/dist");
    if !dist.exists() {
        return Err(anyhow::anyhow!("web/dist/ not found — run `just web-build` first"));
    }
    println!("   Syncing web/dist/ → s3://{}/{}/", bucket, sha);

    let mut file_count = 0usize;
    let mut total_bytes = 0u64;

    for entry in walkdir::WalkDir::new(dist).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(dist)
            .map_err(|_| anyhow::anyhow!("path strip failed"))?;
        let key = format!("{}/{}", sha, rel.to_string_lossy().replace('\\', "/"));

        let is_index = rel.to_string_lossy() == "index.html";
        let cache_control = if is_index {
            "no-cache"
        } else {
            "public,max-age=31536000,immutable"
        };

        let bytes = std::fs::read(path)?;
        let content_type = mime_guess(path);
        let len = bytes.len() as u64;

        client
            .put_object()
            .bucket(bucket)
            .key(&key)
            .body(ByteStream::from(bytes))
            .cache_control(cache_control)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("S3 upload failed for {key}: {e}"))?;

        file_count += 1;
        total_bytes += len;
    }

    println!("   Synced {} files ({} bytes)", file_count, total_bytes);
    Ok((file_count, total_bytes))
}

/// Step 5: invoke Lambda sync-spa action and assert status == "ok".
pub async fn invoke_sync_handler(
    client: &LambdaClient,
    fn_name: &str,
    sha: &str,
) -> anyhow::Result<serde_json::Value> {
    println!("   Invoking sync-spa handler (sha={})...", sha);
    let payload = serde_json::json!({ "action": "sync-spa", "sha": sha });
    let resp = client
        .invoke()
        .function_name(fn_name)
        .payload(Blob::new(serde_json::to_vec(&payload)?))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Lambda invoke failed: {e}"))?;

    if let Some(err) = resp.function_error() {
        return Err(anyhow::anyhow!("Lambda returned function error: {}", err));
    }

    let body: serde_json::Value = resp
        .payload()
        .map(|b| serde_json::from_slice(b.as_ref()))
        .transpose()
        .map_err(|e| anyhow::anyhow!("Failed to parse sync-spa response: {e}"))?
        .unwrap_or(serde_json::Value::Null);

    if body.get("status").and_then(|v| v.as_str()) != Some("ok") {
        return Err(anyhow::anyhow!("sync-spa returned non-ok status: {}", body));
    }
    println!("   sync-spa: {}", body);
    Ok(body)
}

/// Step 6: curl <fn_url>/health and assert HTTP 200.
pub async fn smoke_test(fn_url: &str) -> anyhow::Result<()> {
    let url = format!("{}health", fn_url.trim_end_matches('/'));
    println!("   Smoke testing {}...", url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Health check request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow::anyhow!("/health returned {}", status));
    }
    println!("   /health → {}", status);
    Ok(())
}

fn mime_guess(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain",
        Some("map") => "application/json",
        _ => "application/octet-stream",
    }
}

/// Full SPA deploy: steps 2–6.
pub async fn deploy_spa(
    profile: Option<String>,
    env_cfg: SpaEnvConfig,
    sha: &str,
    skip_wait: bool,
) -> anyhow::Result<()> {
    let aws_config = crate::aws::create_aws_config(profile).await?;
    let lambda_client = LambdaClient::new(&aws_config);
    let s3_client = S3Client::new(&aws_config);

    if !skip_wait {
        wait_lambda_active(&lambda_client, &env_cfg.fn_name).await?;
    }
    build_spa()?;
    sync_to_s3(&s3_client, &env_cfg.spa_bucket, sha).await?;
    invoke_sync_handler(&lambda_client, &env_cfg.fn_name, sha).await?;
    smoke_test(&env_cfg.fn_url).await?;

    println!("SPA deploy complete (sha={})", sha);
    Ok(())
}
