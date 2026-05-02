use regex::Regex;
use semver::Version;
use std::sync::LazyLock;

use super::git::CommitInfo;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpKind {
    None,
    Patch,
    Minor,
    Major,
}

pub fn compute_bump(commits: &[CommitInfo]) -> BumpKind {
    let mut best = BumpKind::None;
    for c in commits {
        let k = classify(&c.subject, &c.body);
        if k > best {
            best = k;
        }
        if best == BumpKind::Major {
            break;
        }
    }
    if best == BumpKind::None {
        BumpKind::Patch
    } else {
        best
    }
}

fn classify(subject: &str, body: &str) -> BumpKind {
    if RE_BREAKING_BANG.is_match(subject) {
        return BumpKind::Major;
    }
    if body.contains("BREAKING CHANGE:") || body.contains("BREAKING-CHANGE:") {
        return BumpKind::Major;
    }
    if RE_FEAT.is_match(subject) {
        return BumpKind::Minor;
    }
    if RE_PATCH.is_match(subject) {
        return BumpKind::Patch;
    }
    if RE_SKIP.is_match(subject) {
        return BumpKind::None;
    }
    BumpKind::Patch
}

static RE_BREAKING_BANG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^[a-z]+(\([^)]+\))?!:").unwrap());
static RE_FEAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^feat(\([^)]+\))?:").unwrap());
static RE_PATCH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(fix|refactor|perf)(\([^)]+\))?:").unwrap());
static RE_SKIP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(docs|chore|style|test|build|ci)(\([^)]+\))?:").unwrap());

pub fn next(base: &str, bump: BumpKind) -> anyhow::Result<String> {
    let mut v = Version::parse(base)?;
    match bump {
        BumpKind::Major => {
            v.major += 1;
            v.minor = 0;
            v.patch = 0;
        }
        BumpKind::Minor => {
            v.minor += 1;
            v.patch = 0;
        }
        BumpKind::Patch | BumpKind::None => {
            v.patch += 1;
        }
    }
    Ok(v.to_string())
}

pub fn floor_from_cargo() -> anyhow::Result<String> {
    // Try xtask/Cargo.toml for an explicit version first.
    let xtask_candidates = ["xtask/Cargo.toml", "../xtask/Cargo.toml"];
    let xtask_manifest = xtask_candidates
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .ok_or_else(|| anyhow::anyhow!("cannot find xtask/Cargo.toml"))?;

    let mut in_package = false;
    for line in xtask_manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("version") {
            // Skip workspace inheritance: `version.workspace = true`
            if rest.starts_with('.') {
                continue;
            }
            let ver = rest
                .trim_start_matches(|c: char| c.is_whitespace() || c == '=')
                .trim_matches('"');
            if Version::parse(ver).is_ok() {
                return Ok(ver.to_string());
            }
        }
    }

    // Fallback: xtask uses `version.workspace = true` — read from [workspace.package].
    let ws_candidates = ["Cargo.toml", "../Cargo.toml"];
    let ws_manifest = ws_candidates
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .ok_or_else(|| anyhow::anyhow!("cannot find workspace Cargo.toml"))?;

    let mut in_ws_package = false;
    for line in ws_manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_ws_package = line == "[workspace.package]";
            continue;
        }
        if !in_ws_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("version") {
            if rest.starts_with('.') {
                continue;
            }
            let ver = rest
                .trim_start_matches(|c: char| c.is_whitespace() || c == '=')
                .trim_matches('"');
            if Version::parse(ver).is_ok() {
                return Ok(ver.to_string());
            }
        }
    }

    anyhow::bail!("version not found in xtask/Cargo.toml or workspace [workspace.package]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release::git::CommitInfo;

    fn ci(subject: &str) -> CommitInfo {
        CommitInfo {
            subject: subject.to_string(),
            body: String::new(),
            sha: "abc1234".to_string(),
        }
    }

    fn ci_body(subject: &str, body: &str) -> CommitInfo {
        CommitInfo {
            subject: subject.to_string(),
            body: body.to_string(),
            sha: "abc1234".to_string(),
        }
    }

    #[test]
    fn feat_fix_docs_is_minor() {
        let bump = compute_bump(&[
            ci("feat: add thing"),
            ci("fix: correct thing"),
            ci("docs: update readme"),
        ]);
        assert_eq!(bump, BumpKind::Minor);
    }

    #[test]
    fn breaking_bang_is_major() {
        let bump = compute_bump(&[ci("fix!: remove old api")]);
        assert_eq!(bump, BumpKind::Major);
    }

    #[test]
    fn breaking_body_is_major() {
        let bump = compute_bump(&[ci_body(
            "feat: new thing",
            "BREAKING CHANGE: removed old param",
        )]);
        assert_eq!(bump, BumpKind::Major);
    }

    #[test]
    fn only_docs_chore_is_patch() {
        let bump = compute_bump(&[ci("docs: update readme"), ci("chore: bump deps")]);
        assert_eq!(bump, BumpKind::Patch);
    }

    #[test]
    fn no_commits_is_patch() {
        let bump = compute_bump(&[]);
        assert_eq!(bump, BumpKind::Patch);
    }

    #[test]
    fn next_minor_bump() {
        let result = next("0.1.3", BumpKind::Minor).unwrap();
        assert_eq!(result, "0.2.0");
    }

    #[test]
    fn next_major_bump() {
        let result = next("0.1.3", BumpKind::Major).unwrap();
        assert_eq!(result, "1.0.0");
    }

    #[test]
    fn next_patch_bump() {
        let result = next("0.1.3", BumpKind::Patch).unwrap();
        assert_eq!(result, "0.1.4");
    }

    #[test]
    fn refactor_is_patch() {
        let bump = compute_bump(&[ci("refactor: restructure module")]);
        assert_eq!(bump, BumpKind::Patch);
    }

    #[test]
    fn perf_is_patch() {
        let bump = compute_bump(&[ci("perf: speed up query")]);
        assert_eq!(bump, BumpKind::Patch);
    }

    #[test]
    fn feat_with_scope_is_minor() {
        let bump = compute_bump(&[ci("feat(ui): add dark mode")]);
        assert_eq!(bump, BumpKind::Minor);
    }

    #[test]
    fn breaking_bang_with_scope_is_major() {
        let bump = compute_bump(&[ci("fix(api)!: remove deprecated endpoint")]);
        assert_eq!(bump, BumpKind::Major);
    }

    #[test]
    fn breaking_hyphen_body_is_major() {
        let bump = compute_bump(&[ci_body(
            "feat: new thing",
            "BREAKING-CHANGE: old param removed",
        )]);
        assert_eq!(bump, BumpKind::Major);
    }

    #[test]
    fn unconventional_subject_defaults_to_patch() {
        let bump = compute_bump(&[ci("Update readme typo")]);
        assert_eq!(bump, BumpKind::Patch);
    }

    #[test]
    fn next_none_bump_acts_as_patch() {
        let result = next("1.2.3", BumpKind::None).unwrap();
        assert_eq!(result, "1.2.4");
    }
}
