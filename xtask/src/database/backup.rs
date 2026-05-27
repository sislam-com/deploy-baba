//! Database backup operations

use aws_sdk_s3::Client as S3Client;
use flate2::Compression;
use rusqlite::Connection;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn backup_database(path: Option<String>, profile: Option<String>) -> anyhow::Result<()> {
    let db_path = path.unwrap_or_else(|| "deploy-baba.db".to_string());
    println!("💾 Backing up database: {}", db_path);

    if !Path::new(&db_path).exists() {
        return Err(anyhow::anyhow!("Database file not found: {}", db_path));
    }

    // The database runs in WAL mode (PRAGMA journal_mode=WAL), so the main
    // .db file alone is insufficient — committed data may live in the -wal file.
    // Use SQLite's online backup API to produce a consistent snapshot.
    println!("   Creating consistent snapshot...");
    let temp_path = format!("{}.backup-{}", db_path, std::process::id());

    let src_conn = Connection::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open database for backup: {}", e))?;
    src_conn
        .backup(rusqlite::DatabaseName::Main, &temp_path, None)
        .map_err(|e| anyhow::anyhow!("SQLite backup failed: {}", e))?;

    let mut snapshot_file = File::open(&temp_path)
        .map_err(|e| anyhow::anyhow!("Failed to open snapshot file: {}", e))?;
    let mut db_data = Vec::new();
    snapshot_file
        .read_to_end(&mut db_data)
        .map_err(|e| anyhow::anyhow!("Failed to read snapshot file: {}", e))?;

    std::fs::remove_file(&temp_path)
        .unwrap_or_else(|_| eprintln!("   ⚠️  Failed to clean up temp snapshot: {}", temp_path));

    // Compress with gzip
    println!("   Compressing...");
    let compressed = compress_data(&db_data)?;

    // Upload to S3
    println!("   Uploading to S3...");
    let bucket = super::resolve_bucket(&profile).await;
    let config = crate::aws::create_aws_config(profile).await?;
    let client = S3Client::new(&config);

    // Generate timestamp using SystemTime
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("Failed to get current time: {}", e))?
        .as_secs();

    let backup_key = format!("db-backups/app-{}.db.gz", timestamp);

    client
        .put_object()
        .bucket(&bucket)
        .key(&backup_key)
        .body(aws_sdk_s3::primitives::ByteStream::from(compressed))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload backup to S3: {}", e))?;

    println!("✅ Database backed up: {}", backup_key);
    Ok(())
}

fn compress_data(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| anyhow::anyhow!("Compression failed: {}", e))?;

    encoder
        .finish()
        .map_err(|e| anyhow::anyhow!("Compression finish failed: {}", e))
}
