//! Agent cache management subcommand
//!
//! Provides status/refresh/clear operations for .agent-cache/index.json.
//! Replaces the inline Python heredoc in the justfile (which `just` cannot parse).

use clap::Subcommand;
use std::fs;
use std::process::Command;

#[derive(Subcommand)]
pub enum CacheAction {
    /// Show cache age and staleness vs current HEAD
    Status,
    /// Re-scan the codebase and rewrite .agent-cache/index.json
    Refresh,
    /// Delete cache to force full re-scan
    Clear,
}

pub async fn execute(action: CacheAction) -> anyhow::Result<()> {
    match action {
        CacheAction::Status => status(),
        CacheAction::Refresh => refresh(),
        CacheAction::Clear => clear(),
    }
}

fn git(args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("git").args(args).output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn read_cache() -> anyhow::Result<serde_json::Value> {
    let raw = fs::read_to_string(".agent-cache/index.json")
        .map_err(|_| anyhow::anyhow!("No cache found — run: just cache-refresh"))?;
    Ok(serde_json::from_str(&raw)?)
}

fn status() -> anyhow::Result<()> {
    let cache = match read_cache() {
        Ok(c) => c,
        Err(e) => {
            println!("❌ {}", e);
            return Ok(());
        }
    };

    let cached_sha = cache["git"]["sha"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let last_updated = cache["_meta"]["last_updated"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let head_sha = git(&["rev-parse", "HEAD"])?;

    if cached_sha == head_sha {
        println!("✅ Cache is FRESH (last updated: {})", last_updated);
        println!("   SHA: {}", head_sha);
    } else {
        println!(
            "⚠️  Cache is STALE (cached: {}, HEAD: {})",
            &cached_sha[..7.min(cached_sha.len())],
            &head_sha[..7.min(head_sha.len())]
        );
        println!("   Changed files since cache:");
        let diff = Command::new("git")
            .args(["diff", "--name-only", &cached_sha, "HEAD"])
            .output();
        if let Ok(out) = diff {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                println!("   - {}", line);
            }
        }
        println!("   Run: just cache-refresh");
    }

    Ok(())
}

fn refresh() -> anyhow::Result<()> {
    println!("🔄 Refreshing agent cache...");

    let mut cache = read_cache()?;

    let head_sha = git(&["rev-parse", "HEAD"])?;
    let short_sha = git(&["rev-parse", "--short", "HEAD"])?;
    let branch = git(&["branch", "--show-current"])?;
    let last_commit = git(&["log", "-1", "--pretty=%s"])?;
    let today = {
        let out = Command::new("date").arg("+%Y-%m-%d").output()?;
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    cache["git"]["sha"] = serde_json::Value::String(head_sha.clone());
    cache["git"]["short_sha"] = serde_json::Value::String(short_sha.clone());
    cache["git"]["branch"] = serde_json::Value::String(branch.clone());
    cache["git"]["last_commit"] = serde_json::Value::String(last_commit);
    cache["_meta"]["last_updated"] = serde_json::Value::String(today);
    cache["_meta"]["generated_by"] = serde_json::Value::String("just cache-refresh".to_string());

    // Mark all component SHAs as current (shallow refresh — SHA-based staleness only)
    if let Some(crates) = cache["crates"].as_object_mut() {
        for crate_obj in crates.values_mut() {
            crate_obj["git_sha_at_scan"] = serde_json::Value::String(head_sha.clone());
        }
    }
    if let Some(services) = cache["services"].as_object_mut() {
        for svc in services.values_mut() {
            svc["git_sha_at_scan"] = serde_json::Value::String(head_sha.clone());
        }
    }
    for key in &["infra", "xtask", "database"] {
        if cache[key].is_object() {
            cache[*key]["git_sha_at_scan"] = serde_json::Value::String(head_sha.clone());
        }
    }

    let json = serde_json::to_string_pretty(&cache)?;
    fs::write(".agent-cache/index.json", json)?;

    println!("✅ Cache refreshed → SHA: {} ({})", short_sha, branch);
    Ok(())
}

fn clear() -> anyhow::Result<()> {
    let path = ".agent-cache/index.json";
    if fs::remove_file(path).is_ok() {
        println!("🗑️  Cache cleared — Claude will do a full re-scan next session");
    } else {
        println!("ℹ️  No cache file found (already clear)");
    }
    Ok(())
}
