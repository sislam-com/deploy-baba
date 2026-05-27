//! OpenTofu command wrappers
//!
//! All commands default to running in the "infra/" subdirectory via tofu's
//! `-chdir=<dir>` flag.
//!
//! `workspace` selects the OpenTofu workspace (default → prod, dev → dev).
//! `aws_profile` sets `AWS_PROFILE` for credentials (independent of workspace).

use std::process::Command;

pub fn check_tofu_binary() -> anyhow::Result<()> {
    let output = Command::new("tofu").arg("version").output();
    match output {
        Ok(o) if o.status.success() => Ok(()),
        _ => Err(anyhow::anyhow!(
            "tofu binary not found. Install with: brew install opentofu"
        )),
    }
}

fn make_cmd(dir: &str, aws_profile: Option<&str>) -> Command {
    let mut cmd = Command::new("tofu");
    cmd.arg(format!("-chdir={}", dir));
    if let Some(p) = aws_profile {
        cmd.env("AWS_PROFILE", p);
    }
    cmd
}

fn select_workspace(dir: &str, workspace: &str, aws_profile: Option<&str>) -> anyhow::Result<()> {
    let mut cmd = make_cmd(dir, aws_profile);
    cmd.args(["workspace", "select", workspace]);
    let status = cmd.status()?;
    if status.success() {
        return Ok(());
    }
    let mut cmd = make_cmd(dir, aws_profile);
    cmd.args(["workspace", "new", workspace]);
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to select or create workspace '{}'",
            workspace
        ));
    }
    Ok(())
}

fn resolve_dir(dir: Option<&str>) -> String {
    dir.unwrap_or("infra").to_string()
}

/// Select workspace and return extra `-var` args to append after the subcommand.
fn prepare_workspace(
    dir: &str,
    workspace: Option<&str>,
    aws_profile: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    if let Some(ws) = workspace {
        select_workspace(dir, ws, aws_profile)?;
        if ws != "default" {
            return Ok(vec!["-var".into(), format!("environment={}", ws)]);
        }
    }
    Ok(vec![])
}

pub async fn run_tofu_init(dir: Option<String>, aws_profile: Option<String>) -> anyhow::Result<()> {
    check_tofu_binary()?;
    let dir = resolve_dir(dir.as_deref());
    let mut cmd = make_cmd(&dir, aws_profile.as_deref());
    println!("Initializing OpenTofu ({})...", dir);
    cmd.arg("init");

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("tofu init failed"));
    }

    println!("OpenTofu initialized");
    Ok(())
}

pub async fn run_tofu_plan(
    dir: Option<String>,
    workspace: Option<String>,
    aws_profile: Option<String>,
) -> anyhow::Result<()> {
    check_tofu_binary()?;
    let dir = resolve_dir(dir.as_deref());
    let extra = prepare_workspace(&dir, workspace.as_deref(), aws_profile.as_deref())?;
    println!(
        "Planning OpenTofu ({}, workspace={})...",
        dir,
        workspace.as_deref().unwrap_or("default")
    );
    let mut cmd = make_cmd(&dir, aws_profile.as_deref());
    cmd.arg("plan");
    cmd.args(&extra);

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("tofu plan failed"));
    }

    println!("OpenTofu plan complete");
    Ok(())
}

pub async fn run_tofu_apply(
    dir: Option<String>,
    auto_approve: bool,
    workspace: Option<String>,
    aws_profile: Option<String>,
) -> anyhow::Result<()> {
    check_tofu_binary()?;
    let dir = resolve_dir(dir.as_deref());
    let extra = prepare_workspace(&dir, workspace.as_deref(), aws_profile.as_deref())?;
    println!(
        "Applying OpenTofu ({}, workspace={})...",
        dir,
        workspace.as_deref().unwrap_or("default")
    );
    let mut cmd = make_cmd(&dir, aws_profile.as_deref());
    cmd.arg("apply");
    cmd.args(&extra);

    if auto_approve {
        cmd.arg("-auto-approve");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("tofu apply failed"));
    }

    println!("OpenTofu applied successfully");
    Ok(())
}

pub async fn run_tofu_destroy(
    dir: Option<String>,
    auto_approve: bool,
    workspace: Option<String>,
    aws_profile: Option<String>,
) -> anyhow::Result<()> {
    check_tofu_binary()?;
    let dir = resolve_dir(dir.as_deref());
    let extra = prepare_workspace(&dir, workspace.as_deref(), aws_profile.as_deref())?;
    println!(
        "Destroying OpenTofu ({}, workspace={})...",
        dir,
        workspace.as_deref().unwrap_or("default")
    );
    let mut cmd = make_cmd(&dir, aws_profile.as_deref());
    cmd.arg("destroy");
    cmd.args(&extra);

    if auto_approve {
        cmd.arg("-auto-approve");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("tofu destroy failed"));
    }

    println!("OpenTofu destroyed");
    Ok(())
}

pub async fn run_tofu_output(
    name: Option<String>,
    dir: Option<String>,
    workspace: Option<String>,
    aws_profile: Option<String>,
) -> anyhow::Result<()> {
    check_tofu_binary()?;
    let dir = resolve_dir(dir.as_deref());
    let extra = prepare_workspace(&dir, workspace.as_deref(), aws_profile.as_deref())?;
    println!(
        "Getting OpenTofu output{} ({})...",
        name.as_ref()
            .map(|n| format!(": {}", n))
            .unwrap_or_default(),
        dir,
    );
    let mut cmd = make_cmd(&dir, aws_profile.as_deref());
    cmd.arg("output");
    cmd.arg("-json");
    cmd.args(&extra);

    if let Some(n) = name {
        cmd.arg(n);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("tofu output failed"));
    }

    println!("Output retrieved");
    Ok(())
}
