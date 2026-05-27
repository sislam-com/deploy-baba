//! Database restore operations

use aws_sdk_s3::Client as S3Client;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Write;

pub async fn restore_database(
    version: Option<String>,
    path: Option<String>,
    profile: Option<String>,
) -> anyhow::Result<()> {
    let db_path = path.unwrap_or_else(|| "deploy-baba.db".to_string());
    println!("♻️  Restoring database to: {}", db_path);

    let config = crate::aws::create_aws_config(profile.clone()).await?;
    let client = S3Client::new(&config);
    let bucket = super::resolve_bucket(&profile).await;

    // Determine backup key
    let backup_key = if let Some(v) = version {
        format!("db-backups/app-{}.db.gz", v)
    } else {
        // Get latest backup
        println!("   Finding latest backup...");
        let response = client
            .list_objects_v2()
            .bucket(&bucket)
            .prefix("db-backups/")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list backups: {}", e))?;

        let latest = response
            .contents
            .and_then(|mut objects| {
                objects.sort_by(|a, b| {
                    let empty = String::new();
                    let a_key = a.key.as_ref().unwrap_or(&empty);
                    let b_key = b.key.as_ref().unwrap_or(&empty);
                    b_key.cmp(a_key)
                });
                objects.into_iter().next().and_then(|o| o.key)
            })
            .ok_or_else(|| anyhow::anyhow!("No backups found"))?;

        latest
    };

    println!("   Downloading backup: {}", backup_key);

    // Download from S3
    let response = client
        .get_object()
        .bucket(&bucket)
        .key(&backup_key)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to download backup: {}", e))?;

    let compressed_data = response
        .body
        .collect()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read backup data: {}", e))?
        .into_bytes()
        .to_vec();

    // Decompress
    println!("   Decompressing...");
    let decompressed = decompress_data(&compressed_data)?;

    // Write to database file
    let mut output_file = File::create(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to create database file: {}", e))?;

    output_file
        .write_all(&decompressed)
        .map_err(|e| anyhow::anyhow!("Failed to write database file: {}", e))?;

    println!("✅ Database restored: {}", db_path);
    Ok(())
}

fn decompress_data(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();

    std::io::Read::read_to_end(&mut decoder, &mut decompressed)
        .map_err(|e| anyhow::anyhow!("Decompression failed: {}", e))?;

    Ok(decompressed)
}
