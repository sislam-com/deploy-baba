use super::{git::CommitInfo, version::BumpKind};

pub fn render(tag: &str, bump: &BumpKind, range: &str, commits: &[CommitInfo]) -> String {
    let bump_label = match bump {
        BumpKind::Major => "major",
        BumpKind::Minor => "minor",
        BumpKind::Patch | BumpKind::None => "patch",
    };

    let mut features: Vec<String> = Vec::new();
    let mut fixes: Vec<String> = Vec::new();
    let mut other: Vec<String> = Vec::new();

    for c in commits {
        let line = format!("- {} ({})", c.subject, c.sha);
        if c.subject.starts_with("feat") {
            features.push(line);
        } else if c.subject.starts_with("fix") || c.subject.starts_with("perf") {
            fixes.push(line);
        } else if !is_skip_prefix(&c.subject) {
            other.push(line);
        }
    }

    let mut out = format!(
        "Release {tag}\n\nBump kind: {bump_label} (auto-detected)\nRange: {range} ({n} commits)\n",
        n = commits.len()
    );

    if !features.is_empty() {
        out.push_str("\nFeatures\n");
        for l in &features {
            out.push_str(l);
            out.push('\n');
        }
    }
    if !fixes.is_empty() {
        out.push_str("\nFixes\n");
        for l in &fixes {
            out.push_str(l);
            out.push('\n');
        }
    }
    if !other.is_empty() {
        out.push_str("\nOther\n");
        for l in &other {
            out.push_str(l);
            out.push('\n');
        }
    }

    out
}

fn is_skip_prefix(subject: &str) -> bool {
    for prefix in &["docs", "chore", "style", "test", "build", "ci"] {
        if let Some(rest) = subject.strip_prefix(prefix) {
            if rest.starts_with(':') || rest.starts_with('(') {
                return true;
            }
        }
    }
    false
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

    #[test]
    fn has_features_and_fixes_no_docs() {
        let commits = vec![
            ci("feat: add thing"),
            ci("fix: correct thing"),
            ci("docs: update readme"),
        ];
        let body = render("dev-v0.2.0", &BumpKind::Minor, "dev-v0.1.0..HEAD", &commits);
        assert!(body.contains("Features"));
        assert!(body.contains("Fixes"));
        assert!(!body.contains("docs: update readme"));
    }

    #[test]
    fn omits_empty_sections() {
        let commits = vec![ci("fix: only fixes")];
        let body = render("dev-v0.1.1", &BumpKind::Patch, "dev-v0.1.0..HEAD", &commits);
        assert!(!body.contains("Features"));
        assert!(body.contains("Fixes"));
        assert!(!body.contains("Other"));
    }

    #[test]
    fn refactor_goes_to_other() {
        let commits = vec![ci("refactor: restructure module")];
        let body = render("dev-v0.1.1", &BumpKind::Patch, "dev-v0.1.0..HEAD", &commits);
        assert!(body.contains("Other"));
        assert!(body.contains("refactor: restructure module"));
    }

    #[test]
    fn perf_goes_to_fixes() {
        let commits = vec![ci("perf: speed up db query")];
        let body = render("dev-v0.1.1", &BumpKind::Patch, "dev-v0.1.0..HEAD", &commits);
        assert!(body.contains("Fixes"));
        assert!(body.contains("perf: speed up db query"));
        assert!(!body.contains("Other"));
    }

    #[test]
    fn skip_prefixes_are_omitted() {
        for prefix in &["chore", "style", "test", "build", "ci"] {
            let commits = vec![ci(&format!("{prefix}: routine update"))];
            let body = render("dev-v0.1.1", &BumpKind::Patch, "dev-v0.1.0..HEAD", &commits);
            assert!(
                !body.contains(&format!("{prefix}: routine update")),
                "'{prefix}:' commit should be omitted from changelog"
            );
        }
    }

    #[test]
    fn major_bump_label_in_header() {
        let commits = vec![ci("feat!: breaking change")];
        let body = render("v1.0.0", &BumpKind::Major, "v0.9.0..HEAD", &commits);
        assert!(body.contains("major"));
    }

    #[test]
    fn header_contains_range_and_commit_count() {
        let commits = vec![ci("feat: a"), ci("fix: b"), ci("chore: c")];
        let body = render("dev-v0.3.0", &BumpKind::Minor, "dev-v0.2.0..HEAD", &commits);
        assert!(body.contains("dev-v0.2.0..HEAD"));
        assert!(body.contains("3 commits"));
    }

    #[test]
    fn sha_appears_in_commit_lines() {
        let commits = vec![ci("feat: shiny new thing")];
        let body = render("dev-v0.2.0", &BumpKind::Minor, "dev-v0.1.0..HEAD", &commits);
        assert!(body.contains("abc1234"));
    }
}
