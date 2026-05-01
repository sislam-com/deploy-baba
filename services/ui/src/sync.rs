use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use serde::{Deserialize, Serialize};
use std::{
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};
use tracing::{info, warn};

#[derive(Deserialize)]
pub struct SyncPayload {
    pub sha: String,
}

#[derive(Serialize)]
pub struct SyncResponse {
    pub status: &'static str,
    pub active_sha: String,
    pub files: usize,
    pub bytes: u64,
    pub duration_ms: u128,
}

pub async fn handle(payload: SyncPayload, s3: &Client, bucket: &str) -> Result<SyncResponse> {
    let sha = &payload.sha;
    validate_sha(sha)?;

    let started = std::time::Instant::now();
    let spa_base = Path::new("/mnt/spa");
    let dest_dir = spa_base.join(sha);

    let (files, bytes) = sync_from_s3(s3, bucket, sha, &dest_dir).await?;
    atomic_swap(sha, spa_base)?;

    let duration_ms = started.elapsed().as_millis();
    info!(sha, files, bytes, duration_ms, "spa sync complete");

    Ok(SyncResponse {
        status: "ok",
        active_sha: sha.clone(),
        files,
        bytes,
        duration_ms,
    })
}

fn validate_sha(sha: &str) -> Result<()> {
    let len = sha.len();
    anyhow::ensure!(
        (7..=40).contains(&len) && sha.chars().all(|c| c.is_ascii_hexdigit()),
        "invalid sha: must be 7-40 hex chars, got {:?}",
        sha
    );
    Ok(())
}

async fn sync_from_s3(s3: &Client, bucket: &str, sha: &str, dest: &Path) -> Result<(usize, u64)> {
    tokio::fs::create_dir_all(dest)
        .await
        .with_context(|| format!("create dir {dest:?}"))?;

    let prefix = format!("{sha}/");
    let mut continuation = None;
    let mut files = 0usize;
    let mut bytes = 0u64;

    loop {
        let mut req = s3.list_objects_v2().bucket(bucket).prefix(&prefix);
        if let Some(tok) = continuation.take() {
            req = req.continuation_token(tok);
        }
        let resp = req.send().await.context("list_objects_v2")?;

        for obj in resp.contents() {
            let key = obj.key().unwrap_or_default();
            let rel = key.trim_start_matches(&prefix);
            if rel.is_empty() {
                continue;
            }
            let local_path = dest.join(rel);
            if let Some(parent) = local_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let get = s3
                .get_object()
                .bucket(bucket)
                .key(key)
                .send()
                .await
                .with_context(|| format!("get_object {key}"))?;

            let size = get.content_length().unwrap_or(0) as u64;
            let body = get.body.collect().await?.into_bytes();
            tokio::fs::write(&local_path, &body)
                .await
                .with_context(|| format!("write {local_path:?}"))?;

            files += 1;
            bytes += size;
        }

        match resp.next_continuation_token() {
            Some(tok) => continuation = Some(tok.to_owned()),
            None => break,
        }
    }

    Ok((files, bytes))
}

fn atomic_swap(sha: &str, spa_base: &Path) -> Result<()> {
    let next = spa_base.join(".active.next");
    let active = spa_base.join("active");

    if next.exists() {
        std::fs::remove_file(&next).ok();
    }

    let sha_path = PathBuf::from(sha);
    symlink(&sha_path, &next).with_context(|| format!("symlink {next:?} → {sha}"))?;
    std::fs::rename(&next, &active).with_context(|| format!("rename {next:?} → {active:?}"))?;

    info!(sha, "active symlink swapped");
    Ok(())
}

pub async fn prune(keep: usize) -> Result<usize> {
    let spa_base = Path::new("/mnt/spa");
    let active_target = std::fs::read_link(spa_base.join("active"))
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));

    let mut dirs: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    let mut entries = tokio::fs::read_dir(spa_base).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(meta) = entry.metadata().await {
                if let Ok(modified) = meta.modified() {
                    dirs.push((modified, path));
                }
            }
        }
    }

    dirs.sort_by_key(|(t, _)| *t);
    dirs.reverse();

    let mut removed = 0;
    for (_, dir) in dirs.iter().skip(keep) {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if Some(&name) == active_target.as_ref() {
            warn!(sha = name, "skipping active sha during prune");
            continue;
        }
        tokio::fs::remove_dir_all(dir).await?;
        removed += 1;
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn validate_sha_accepts_valid() {
        assert!(validate_sha("abc1234").is_ok());
        assert!(validate_sha("deadbeef1234567890abcdef").is_ok());
    }

    #[test]
    fn validate_sha_rejects_invalid() {
        assert!(validate_sha("").is_err());
        assert!(validate_sha("abc123z").is_err());
        assert!(validate_sha("abc").is_err());
    }

    #[test]
    fn atomic_swap_creates_symlink() {
        let base = std::env::temp_dir().join(format!("spa-test-{}", std::process::id()));
        fs::create_dir_all(&base).unwrap();
        let sha = "deadbeef1234567";
        fs::create_dir(base.join(sha)).unwrap();
        atomic_swap(sha, &base).unwrap();
        let target = std::fs::read_link(base.join("active")).unwrap();
        assert_eq!(target, PathBuf::from(sha));
        fs::remove_dir_all(&base).ok();
    }
}
