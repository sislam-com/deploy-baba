---
name: deploy
description: Build and deploy Lambda functions and the React SPA to AWS. Covers full 7-step pipeline (Lambda + SPA), quality gate, email Lambda, infra changes, and secrets rotation. Full pipeline mirrors deploy-dev.yml / deploy-prod.yml exactly.
argument-hint: "<profile> [--full [--env=dev|prod] [--tag]] [--fast|--email|--infra]"
disable-model-invocation: true
---

Deploy the portfolio Lambda(s) and/or SPA to AWS. All commands go through `just` per ADR-001 — never call `cargo xtask` directly.

## Arguments

- `$1` — AWS profile name (e.g. `personal`, `default`). Required.
- `--full` — run the complete 7-step pipeline (Lambda + SPA + smoke). Default path.
- `--env=dev|prod` — target environment for `--full`. Default: `prod`.
- `--tag` — also create `dev-vX.Y.Z` git tag after deploy (opt-in; CI is the canonical tagger).
- `--fast` — skip quality gate (hotfixes only; confirm with user).
- `--email` — deploy the email Lambda instead of the UI Lambda.
- `--infra` — run `infra-apply` before the Lambda deploy.

## Decision Tree

Parse `$ARGUMENTS` and follow the matching path:

### Full deploy: Lambda + SPA + smoke (`--full`, default)

**Preconditions — check these before running `just`:**

1. AWS auth: run `aws sts get-caller-identity --profile <PROFILE>`. If it fails, tell the user to run `aws sso login --profile <PROFILE>` and stop.
2. Tools: verify `cargo lambda --version` and `pnpm --version` resolve. If either is missing, tell the user what to install and stop.
3. **Env vars required for `cargo xtask deploy spa`:**
   - `SPA_BUCKET` — from `tofu -chdir=infra output -raw spa_bucket_name`
   - `UI_FN_NAME` — from `tofu -chdir=infra output -raw lambda_function_name`
   - `FN_URL` — from `tofu -chdir=infra output -raw function_url`
   - Export all three before calling `just spa-deploy` (or `just deploy-full`).
4. Worktree cleanliness: check `git status --porcelain`. If dirty, **note it** (don't block unless `--tag` is also passed — tag requires a clean tree).
5. **Prod gate:** if `--env=prod`, require the user to type **"yes, deploy to prod"** in the conversation before proceeding. This mirrors the GitHub `production` environment manual-approval gate. Do not skip this.

**Run:**
```
# Export infra config first
export SPA_BUCKET=$(tofu -chdir=infra output -raw spa_bucket_name)
export UI_FN_NAME=$(tofu -chdir=infra output -raw lambda_function_name)
export FN_URL=$(tofu -chdir=infra output -raw function_url)

# Full pipeline (no tag by default)
just deploy-full <PROFILE>

# Full pipeline + create dev-vX.Y.Z tag
just deploy-full <PROFILE> TAG=1
```

**What the pipeline does (each step, in order):**
1. `just quality` — fmt + clippy + tests + coverage floors. Stop on any failure.
2. `just lambda-deploy <PROFILE>` — cargo-lambda build → zip → upload to Lambda.
3. `just lambda-wait <PROFILE>` — polls `GetFunction` until `LastUpdateStatus == Successful` (120s timeout).
4. `pnpm --dir web run build` — Vite build emits to `web/dist/`.
5. Walk `web/dist/`, upload to `s3://${SPA_BUCKET}/${SHA}/` — hashed assets get `Cache-Control: public,max-age=31536000,immutable`; `index.html` gets `no-cache`.
6. Lambda invoke `{action:"sync-spa", sha}` — EFS atomic symlink swap. Assert `"status":"ok"`.
7. GET `${FN_URL}/health` → assert 200.

**Report after completion:**
- Lambda function name + whether step 2 succeeded.
- SPA: file count + bytes synced + S3 prefix.
- sync-spa response JSON.
- /health status code + latency.
- If `--tag`: tag name pushed.

### Standard Lambda-only deploy (no `--full`)
```
just quality
just lambda-build
just lambda-deploy <PROFILE>
```

### Fast deploy (`--fast`)
```
just deploy-fast <PROFILE>         # skips quality gate — confirm with user first
```

### SPA-only (`--spa`)
```
# Use when Lambda code is unchanged; only SPA assets need refreshing
export SPA_BUCKET=... UI_FN_NAME=... FN_URL=...
just spa-deploy <PROFILE>
```

### Email Lambda deploy (`--email`)
```
just email-build
just email-deploy <PROFILE>
```

### With infra changes (`--infra`)
```
just infra-plan <PROFILE>          # show OpenTofu plan — review before applying
just infra-apply <PROFILE>         # apply infra — confirm with user first
just lambda-build && just lambda-deploy <PROFILE>
```

### After adding a secret (post W-SEC)
```
just secret-put <NAME> <VALUE> <PROFILE>
just lambda-deploy <PROFILE>
```

## Failure Handling

- **Quality gate fails** → stop. Show the specific failure (fmt, clippy, test, or coverage). Do NOT use `--fast` to bypass unless the user explicitly asks.
- **`cargo lambda` not found** → `cargo install cargo-lambda`.
- **AWS credentials expire** → `aws sso login --profile <PROFILE>`.
- **Lambda wait timeout (120s)** → check CloudWatch logs: `just ui-logs <PROFILE>`. The Lambda update may still be in progress or rolled back.
- **S3 upload fails** → check IAM role has `s3:PutObject` on the SPA bucket. The CI role in `infra/ci-oidc.tf` has it; local profiles need equivalent perms.
- **sync-spa returns non-ok** → read the full JSON response. Common causes: EFS not mounted (`/mnt/spa` missing), S3 bucket mismatch (`SPA_BUCKET` wrong env).
- **/health returns non-200** → Lambda may have restarted cold; retry once after 5s. If still failing, check CloudWatch for panics.
- **Dirty worktree + `--tag`** → `xtask release tag` enforces a clean tree. Commit or stash changes first.

## Key Files

- `justfile` — `deploy-full`, `spa-deploy`, `lambda-wait`, `lambda-deploy`, `release-tag`
- `xtask/src/deploy/lambda.rs` — Lambda build/upload
- `xtask/src/deploy/spa.rs` — wait Lambda active, build SPA, S3 sync, invoke sync-spa, smoke /health
- `xtask/src/release/` — conventional-commits versioning + tag creation
- `infra/outputs.tf`, `infra/s3-spa.tf` — `spa_bucket_name`, `lambda_function_name`, `function_url`
- `services/ui/src/sync.rs` — server-side sync-spa handler (EFS atomic swap)
- `.github/workflows/deploy-dev.yml` — the canonical CI pipeline this mirrors locally
