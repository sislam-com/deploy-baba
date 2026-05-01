use anyhow::{Context, Result};
use std::process::Command;

pub struct CommitInfo {
    pub subject: String,
    pub body: String,
    pub sha: String,
}

pub fn last_dev_tag() -> Result<Option<String>> {
    latest_matching_tag("dev-v*")
}

pub fn last_prod_tag() -> Result<Option<String>> {
    latest_matching_tag("v[0-9]*")
}

fn latest_matching_tag(pattern: &str) -> Result<Option<String>> {
    let out = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0", "--match", pattern])
        .output()
        .context("git describe")?;
    if out.status.success() {
        Ok(Some(String::from_utf8(out.stdout)?.trim().to_string()))
    } else {
        Ok(None)
    }
}

pub fn commits_since(since_tag: Option<&str>) -> Result<Vec<CommitInfo>> {
    let range = match since_tag {
        Some(t) => format!("{t}..HEAD"),
        None => "HEAD".to_string(),
    };
    let out = Command::new("git")
        .args(["log", &range, "--format=%s%x00%b%x00%H%x1e"])
        .output()
        .context("git log")?;
    let raw = String::from_utf8(out.stdout)?;
    let mut result = Vec::new();
    for record in raw.split('\x1e') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let parts: Vec<&str> = record.splitn(3, '\x00').collect();
        let subject = parts.first().copied().unwrap_or("").trim().to_string();
        let body = parts.get(1).copied().unwrap_or("").trim().to_string();
        let sha: String = parts
            .get(2)
            .copied()
            .unwrap_or("")
            .trim()
            .chars()
            .take(7)
            .collect();
        if !subject.is_empty() {
            result.push(CommitInfo { subject, body, sha });
        }
    }
    Ok(result)
}

pub fn ensure_clean() -> Result<()> {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("git status")?;
    let stdout = String::from_utf8(out.stdout)?;
    if !stdout.trim().is_empty() {
        anyhow::bail!("working tree is dirty — commit or stash changes first");
    }
    Ok(())
}

pub fn tag_exists(tag: &str) -> Result<bool> {
    let out = Command::new("git")
        .args(["tag", "--list", tag])
        .output()
        .context("git tag --list")?;
    Ok(!String::from_utf8(out.stdout)?.trim().is_empty())
}

pub fn tag_exists_at_head(tag: &str) -> Result<bool> {
    if !tag_exists(tag)? {
        return Ok(false);
    }
    let tag_sha = Command::new("git")
        .args(["rev-list", "-n1", tag])
        .output()?;
    let head_sha = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    Ok(String::from_utf8(tag_sha.stdout)?.trim() == String::from_utf8(head_sha.stdout)?.trim())
}

pub fn tag_sha(tag: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["rev-list", "-n1", tag])
        .output()
        .context("git rev-list")?;
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

pub fn create_annotated_tag(tag: &str, body: &str) -> Result<()> {
    let tmp = write_tag_body(body)?;
    let status = Command::new("git")
        .args(["tag", "-a", tag, "-F", &tmp])
        .status()
        .context("git tag")?;
    if !status.success() {
        anyhow::bail!("git tag failed for {tag}");
    }
    Ok(())
}

pub fn create_annotated_tag_at(tag: &str, body: &str, sha: &str) -> Result<()> {
    let tmp = write_tag_body(body)?;
    let status = Command::new("git")
        .args(["tag", "-a", tag, sha, "-F", &tmp])
        .status()
        .context("git tag")?;
    if !status.success() {
        anyhow::bail!("git tag failed for {tag}");
    }
    Ok(())
}

pub fn push_tag(tag: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["push", "origin", tag])
        .status()
        .context("git push")?;
    if !status.success() {
        anyhow::bail!("git push failed for tag {tag}");
    }
    Ok(())
}

fn write_tag_body(body: &str) -> Result<String> {
    use std::io::Write;
    let path = std::env::temp_dir().join("xtask-tag-body.txt");
    let mut f = std::fs::File::create(&path).context("create tag body tempfile")?;
    f.write_all(body.as_bytes())?;
    Ok(path.to_string_lossy().into_owned())
}
