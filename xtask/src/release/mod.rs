use clap::{Args, Subcommand};

mod changelog;
pub mod git;
mod version;

#[derive(Args)]
pub struct ReleaseArgs {
    #[command(subcommand)]
    cmd: ReleaseCmd,
}

#[derive(Subcommand)]
enum ReleaseCmd {
    /// Print the next version computed from commits since the last dev-v* tag (dry run).
    Next,
    /// Create an annotated tag at HEAD.
    Tag(TagArgs),
    /// Promote the latest dev-v* tag to a v* tag (triggers deploy-prod when pushed).
    Promote(PromoteArgs),
}

#[derive(Args)]
struct TagArgs {
    /// Tag kind: "dev" creates dev-vX.Y.Z; "prod" creates vX.Y.Z (emergency use only).
    #[arg(long, default_value = "dev")]
    kind: TagKind,
    /// Push the tag to origin after creating it.
    #[arg(long)]
    push: bool,
}

#[derive(Args)]
struct PromoteArgs {
    /// Push the v* tag to origin after creating it (triggers deploy-prod.yml).
    #[arg(long)]
    push: bool,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum TagKind {
    Dev,
    Prod,
}

pub fn run(args: ReleaseArgs) -> anyhow::Result<()> {
    match args.cmd {
        ReleaseCmd::Next => cmd_next(),
        ReleaseCmd::Tag(a) => cmd_tag(a),
        ReleaseCmd::Promote(a) => cmd_promote(a),
    }
}

fn cmd_next() -> anyhow::Result<()> {
    let floor = version::floor_from_cargo()?;
    let last_tag = git::last_dev_tag()?;
    let Some(ref last) = last_tag else {
        println!("{floor}");
        return Ok(());
    };
    let commits = git::commits_since(Some(last.as_str()))?;
    let bump = version::compute_bump(&commits);
    let base = last.strip_prefix("dev-v").unwrap_or(&floor);
    let next_ver = version::next(base, bump)?;
    println!("{next_ver}");
    Ok(())
}

fn cmd_tag(args: TagArgs) -> anyhow::Result<()> {
    git::ensure_clean()?;

    let floor = version::floor_from_cargo()?;
    let (prefix, last_tag) = match args.kind {
        TagKind::Dev => ("dev-v", git::last_dev_tag()?),
        TagKind::Prod => ("v", git::last_prod_tag()?),
    };

    let (next_ver, bump, commits, range_desc) = if let Some(ref last) = last_tag {
        let commits = git::commits_since(Some(last.as_str()))?;
        let bump = version::compute_bump(&commits);
        let base = last.strip_prefix(prefix).unwrap_or(&floor);
        let next_ver = version::next(base, bump.clone())?;
        let range_desc = format!("{last}..HEAD");
        (next_ver, bump, commits, range_desc)
    } else {
        let commits = git::commits_since(None)?;
        (
            floor.clone(),
            version::BumpKind::Patch,
            commits,
            "(initial)..HEAD".to_string(),
        )
    };
    let tag_name = format!("{prefix}{next_ver}");

    if git::tag_exists_at_head(&tag_name)? {
        println!("{tag_name} already exists at HEAD — nothing to do");
        return Ok(());
    }
    if git::tag_exists(&tag_name)? {
        anyhow::bail!("{tag_name} already exists at a different commit");
    }

    let body = changelog::render(&tag_name, &bump, &range_desc, &commits);
    git::create_annotated_tag(&tag_name, &body)?;
    println!("created {tag_name}");

    if args.push {
        git::push_tag(&tag_name)?;
        println!("pushed {tag_name}");
    }
    Ok(())
}

fn cmd_promote(args: PromoteArgs) -> anyhow::Result<()> {
    let latest_dev = git::last_dev_tag()?.ok_or_else(|| {
        anyhow::anyhow!("no dev-v* tag found — run `release tag --kind dev` first")
    })?;

    let ver = latest_dev
        .strip_prefix("dev-v")
        .ok_or_else(|| anyhow::anyhow!("unexpected dev tag format: {latest_dev}"))?;

    let prod_tag = format!("v{ver}");

    if git::tag_exists(&prod_tag)? {
        anyhow::bail!("{prod_tag} already exists");
    }

    let dev_sha = git::tag_sha(&latest_dev)?;
    let body = format!("Promote {latest_dev} → {prod_tag}");
    git::create_annotated_tag_at(&prod_tag, &body, &dev_sha)?;
    println!("created {prod_tag} at {dev_sha}");

    if args.push {
        git::push_tag(&prod_tag)?;
        println!("pushed {prod_tag}");
    }
    Ok(())
}
