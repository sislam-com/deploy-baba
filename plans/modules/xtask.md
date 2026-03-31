# W-XT: xtask
**Crate:** `xtask/` | **Status:** WIP
**Coverage floor:** N/A (internal tooling) | **Depends on:** (none — reads stack.toml via file I/O, no crate deps)
**Depended on by:** W-DX (justfile calls xtask)

---

## W-XT.1 Purpose

Internal automation tooling. Invoked exclusively by the justfile — never documented
as a user-facing API. Wraps `cargo`, `tofu`, AWS SDK, and Docker operations.

→ ADR-001 (justfile is the only interface)

Developer flow:
```
just <command> → justfile recipe → cargo xtask <subcommand> → actual tooling
```

---

## W-XT.2 Module Map

```
xtask/src/
├── main.rs          clap dispatcher: build | test | coverage | quality |
│                    aws | infra | deploy | cache | database | publish
├── build.rs         fmt [--check], lint, compile
├── test.rs          unit, all, --crate, quarantine isolation
├── coverage.rs      per-crate floors, HTML report, --open flag
├── quality.rs       orchestrates fmt-check + lint + test + coverage + audit
├── aws/
│   ├── mod.rs       AWS SDK client setup, profile resolution from ~/.aws/config
│   ├── validate.rs  reads /deploy-baba/sentinel from SSM, verifies expected value
│   └── ssm.rs       get_parameter, put_parameter helpers
├── deploy/
│   ├── mod.rs       deploy mode selection (lambda vs ecs from stack.toml)
│   ├── docker.rs    docker build --platform linux/arm64, tag
│   ├── ecr.rs       ECR Public auth + push
│   ├── lambda.rs    cargo lambda build + zip + aws lambda update-function-code
│   └── ecs.rs       register new task definition, update service
├── infra/
│   ├── mod.rs       tofu wrapper, reads AWS profile + region from stack.toml
│   ├── tofu.rs      init, plan, apply, destroy, output (-json)
│   └── bootstrap.rs S3 state bucket + DynamoDB lock + SSM sentinel + tofu init
├── cache.rs         agent cache management: status | refresh | clear
│                    reads/writes .agent-cache/index.json; updates git SHA fields
└── database/
    ├── mod.rs       SQLite + S3 config from stack.toml
    ├── backup.rs    VACUUM INTO + gzip + S3 upload + retention pruning
    └── restore.rs   list S3 objects, download latest or --version, decompress
```

---

## W-XT.3 Implementation Notes

### `aws/validate.rs`
```rust
pub async fn validate_profile(profile: &str) -> Result<CallerInfo, AwsError> {
    let config = load_aws_config_for_profile(profile).await?;
    let sts = aws_sdk_sts::Client::new(&config);
    let ssm = aws_sdk_ssm::Client::new(&config);
    let identity = sts.get_caller_identity().send().await?;
    let param = ssm.get_parameter().name("/deploy-baba/sentinel").send().await?;
    if param.parameter().and_then(|p| p.value()) != Some("deploy-baba-configured") {
        return Err(AwsError::SentinelMismatch);
    }
    Ok(CallerInfo { account: identity.account, arn: identity.arn, profile: profile.to_string() })
}
```

### `infra/bootstrap.rs`
First-run setup (called by `just infra-bootstrap`):
1. Create S3 bucket `deploy-baba-tfstate` (versioning + AES256 + block public access)
2. Create DynamoDB table `terraform-lock` (PAY_PER_REQUEST, LockID hash key)
3. Write SSM params: `/deploy-baba/sentinel`, `/deploy-baba/region`, `/deploy-baba/account`
4. Run `tofu -chdir=infra init` with S3 backend config

### `deploy/lambda.rs`
```
1. cargo lambda build --release --package deploy-baba-ui --target aarch64-unknown-linux-gnu
2. Copy target/lambda/deploy-baba-ui/bootstrap to infra/build/lambda.zip
3. aws lambda update-function-code --function-name deploy-baba-prod \
       --zip-file fileb://infra/build/lambda.zip
4. aws lambda publish-version
5. Print: ✓ Lambda updated, new version: <n>
```

### `database/backup.rs`
```
1. Read SqliteConfig from stack.toml
2. Run VACUUM INTO '/tmp/backup.db' (clean copy without WAL)
3. gzip /tmp/backup.db → /tmp/backup-<timestamp>.db.gz
4. Upload to s3://<bucket>/<prefix><timestamp>.db.gz
5. List all backups, delete oldest if count > retain_versions
```

### OpenTofu subprocess pattern
```rust
cmd.arg(format!("-chdir={}", dir))  // BEFORE subcommand
   .arg(subcommand)
   .env("AWS_PROFILE", profile);
```
Note: `-chdir=<dir>` must come before the subcommand name.

---

## W-XT.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-XT.4.1 | Fix CLI naming mismatch | FIXED | 3 justfile mismatches corrected: `fmt`→`format` (build), `--crate`→`crate` subcommand (test), `gate`→`all` (quality) |
| W-XT.4.2 | Remove or wire EnvironmentInterpolator | OPEN | Dead code warning; either use in build.rs or delete |
| W-XT.4.3 | Fully implement bootstrap.rs | OPEN | Fixed in DRL but needs `just infra-bootstrap` to be tested end-to-end |
| W-XT.4.4 | cache.rs subcommand | DONE | Replaces inline Python heredoc in justfile (which `just` could not parse); implements status/refresh/clear via serde_json |

---

## W-XT.5 Test Strategy

- No coverage floor (internal tooling)
- Tested via justfile integration: `just dev`, `just quality`, `just infra-plan` etc.
- Unit tests for pure functions (e.g., zip path construction, retention pruning logic)
- `cargo build --package xtask` must compile clean ✅

---

## W-XT.6 Cross-References
- → ADR-001 (justfile-only interface)
- → ADR-002 (SQLite database backup)
- → ADR-006 (EFS + S3 topology)
- → W-INFR (StackConfig/SqliteConfig types)
- → W-DX (justfile recipes that call xtask)
- → `plans/drift/DRL-2026-03-18-xtask.md` — all fixed items and open issues
- → `plans/cross-cutting/aws-setup-spec.md` — IAM policy for AWS SDK calls
