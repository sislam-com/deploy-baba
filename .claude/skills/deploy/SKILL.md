---
name: deploy
description: Build and deploy Lambda functions to AWS using just commands. Covers full quality gate, Lambda zip upload, email Lambda, infra changes, and secrets rotation.
argument-hint: "[profile] [--fast|--email|--infra]"
disable-model-invocation: true
---

Deploy the portfolio Lambda(s) to AWS. All commands go through `just` per ADR-001 — never call `cargo xtask` directly.

## Arguments

- `$1` — AWS profile name (e.g. `personal`, `default`). Required.
- `--fast` — skip quality gate (use for hotfixes only)
- `--email` — deploy the email Lambda instead of the UI Lambda
- `--infra` — run `infra-apply` before the Lambda deploy

## Decision Tree

Parse `$ARGUMENTS` and follow the matching path:

### Standard deploy (default)
```
just quality                       # fmt + lint + test + coverage floors + audit
just lambda-build                  # cargo-lambda → aarch64 zip
just lambda-deploy <PROFILE>       # upload zip to Lambda
```

### Fast deploy (--fast)
```
just deploy-fast <PROFILE>         # skips quality gate — confirm with user first
```

### Email Lambda deploy (--email)
```
just email-build                   # build email Lambda zip
just email-deploy <PROFILE>        # upload to email Lambda function
```

### With infra changes (--infra)
```
just infra-plan <PROFILE>          # show OpenTofu plan — review before applying
just infra-apply <PROFILE>         # apply infra — confirm with user first
just lambda-build && just lambda-deploy <PROFILE>
```

### After adding a secret (post W-SEC)
```
just secret-put <NAME> <VALUE> <PROFILE>   # write to Secrets Manager
just lambda-deploy <PROFILE>               # redeploy to pick up new ARN
```

## Failure Handling

- If `just quality` fails → stop. Show the specific failure (fmt, clippy, test, or coverage). Do NOT use `--fast` to bypass.
- If `just lambda-build` fails → check `cargo-lambda` is installed: `cargo lambda --version`. If missing: `cargo install cargo-lambda`.
- If AWS credentials expire → `aws sso login --profile <PROFILE>`.

## Key Files

- `justfile` — all commands defined here
- `xtask/src/deploy/lambda.rs` — Lambda build/upload logic
- `infra/` — OpenTofu HCL (edit before `infra-apply`)
- `xtask/src/secret.rs` — secrets commands
