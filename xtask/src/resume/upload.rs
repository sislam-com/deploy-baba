//! Resume S3 upload

use anyhow::Context;
use aws_sdk_s3::primitives::ByteStream;
use std::fs;
use std::path::Path;

pub async fn upload_resume_files(output_dir: &Path, profile: Option<String>) -> anyhow::Result<()> {
    println!("Uploading resume files to S3...");

    let config = crate::aws::create_aws_config(profile.clone()).await?;

    // Resolve account ID to build the bucket name
    let sts = aws_sdk_sts::Client::new(&config);
    let identity = sts
        .get_caller_identity()
        .send()
        .await
        .context("Failed to get AWS caller identity")?;
    let account_id = identity
        .account()
        .ok_or_else(|| anyhow::anyhow!("No account ID in STS response"))?;

    let bucket = format!("deploy-baba-assets-{}", account_id);
    println!("  Bucket: {}", bucket);

    let s3 = aws_sdk_s3::Client::new(&config);

    let uploads = [
        (
            "sharful-islam-resume-chronological.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "attachment; filename=\"sharful-islam-resume-chronological.docx\"",
        ),
        (
            "sharful-islam-resume-chronological.pdf",
            "application/pdf",
            "attachment; filename=\"sharful-islam-resume-chronological.pdf\"",
        ),
        (
            "sharful-islam-resume-functional.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "attachment; filename=\"sharful-islam-resume-functional.docx\"",
        ),
        (
            "sharful-islam-resume-functional.pdf",
            "application/pdf",
            "attachment; filename=\"sharful-islam-resume-functional.pdf\"",
        ),
    ];

    for (filename, content_type, content_disposition) in &uploads {
        let local_path = output_dir.join(filename);
        if !local_path.exists() {
            println!("  Skipping (not found): {}", local_path.display());
            continue;
        }

        let data = fs::read(&local_path)
            .with_context(|| format!("Failed to read {}", local_path.display()))?;

        let s3_key = format!("resume/{}", filename);
        println!("  Uploading: {} → s3://{}/{}", filename, bucket, s3_key);

        s3.put_object()
            .bucket(&bucket)
            .key(&s3_key)
            .body(ByteStream::from(data))
            .content_type(*content_type)
            .content_disposition(*content_disposition)
            .cache_control("public, max-age=86400")
            .send()
            .await
            .with_context(|| format!("Failed to upload {} to S3", filename))?;

        println!("  Done: {}", s3_key);
    }

    println!("Resume files uploaded.");
    Ok(())
}
