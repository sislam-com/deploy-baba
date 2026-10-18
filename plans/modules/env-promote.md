# W-PROM: Environment Separation & Artifact Promotion

**Path:** `xtask/src/deploy/promote.rs`, `infra/*.tf`, `.github/workflows/`  
**Status:** WIP — Phases 0–5 DONE (auto-promote retired); needs `just infra-apply` + e2e verification  
**Depends on:** W-CI, W-OTF, W-XT, W-MCP  
**Depended on by:** (e2e testing, safe production deploys)

## W-PROM.1 Purpose

Establish true dev/prod infrastructure separation and an artifact promotion pipeline that copies tested code from dev to prod without rebuilding. Enables safe e2e testing at `dev.sislam.com` before production.

## W-PROM.2 Public Surface

| Command | Purpose |
|---------|---------|
| `just promote` | Copy all artifacts from dev → prod (Lambdas + SPA + assets) |
| `just infra-plan dev` | OpenTofu plan for dev workspace |
| `just infra-apply dev` | Provision/update dev infrastructure |
| `just infra-plan` | OpenTofu plan for prod workspace (default) |

## W-PROM.3 Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ Push to main → CI → deploy-dev.yml                               │
│   → builds all Lambdas + SPA                                     │
│   → deploys to dev infrastructure (deploy-baba-dev-*)            │
│   → smoke tests dev.sislam.com                                   │
│   → tags dev-vX.Y.Z                                              │
└──────────────────────────────────────┬───────────────────────────┘
                                       │
                                       ↓ (e2e tests pass)
┌──────────────────────────────────────────────────────────────────┐
│ just promote (or CI auto-promote)                                │
│   1. Download code zip from each dev Lambda function             │
│   2. Upload to corresponding prod Lambda function                │
│   3. aws lambda wait function-updated (each)                     │
│   4. S3 sync dev SPA bucket → prod SPA bucket                   │
│   5. S3 sync dev assets bucket → prod assets bucket              │
│   6. CloudFront invalidation on prod paths                       │
│   7. Smoke test sislam.com/health                                │
│   8. Create vX.Y.Z release tag                                   │
└──────────────────────────────────────────────────────────────────┘
```

### Resource Mapping (dev → prod)

| Resource | Dev Name | Prod Name |
|----------|----------|-----------|
| UI Lambda | `deploy-baba-dev` | `deploy-baba-prod` |
| Email Lambda | `deploy-baba-dev-email` | `deploy-baba-prod-email` |
| LLM-proxy Lambda | `deploy-baba-dev-llm-proxy` | `deploy-baba-prod-llm-proxy` |
| MCP gateway Lambda | `deploy-baba-dev-mcp-gateway` | `deploy-baba-prod-mcp-gateway` |
| SPA bucket | `deploy-baba-dev-spa-{acct}` | `deploy-baba-prod-spa-{acct}` |
| Assets bucket | `deploy-baba-dev-assets-{acct}` | `deploy-baba-prod-assets-{acct}` |
| Deploy config secret | `deploy-baba/dev/deploy-config` | `deploy-baba/prod/deploy-config` |

### Singleton Resources (Prod Workspace Only)

These use `count = var.environment == "prod" ? 1 : 0`:
- `aws_iam_openid_connect_provider.github`
- `aws_iam_role.ci_deploy_dev` + `aws_iam_role.ci_deploy_prod`
- `aws_vpc_endpoint.lambda`, `.secretsmanager`, `.s3`
- `aws_acm_certificate.wildcard` + validation
- `aws_cloudfront_distribution.main` + Route53 records
- CloudFront Function (hostname routing)

### CloudFront Hostname Routing

Single distribution serves both environments. A CloudFront Function rewrites the origin:

```javascript
function handler(event) {
  var request = event.request;
  var host = request.headers.host.value;
  if (host === 'dev.sislam.com') {
    // Route to dev origins — set custom header for origin selection
    request.headers['x-env-origin'] = { value: 'dev' };
  }
  return request;
}
```

Origin groups with failover or cache behaviors per environment path (TBD — depends on CloudFront origin routing capabilities). Alternative: use separate origin IDs per environment in cache behaviors conditioned on host.

**Simplest approach:** Two CloudFront distributions after all. The free tier is generous (1TB + 10M requests). Both distributions share the same ACM cert. This avoids complex hostname routing logic.

## W-PROM.4 Work Items

### Phase 0: Immediate State Fix

Resolved: `default` workspace is prod, `dev` workspace is properly isolated.

| ID | Task | Status |
|---|---|---|
| W-PROM.4.0a | Switch `.terraform/environment` to `default` | DONE |
| W-PROM.4.0b | Force-delete stale `dev` workspace (`tofu workspace delete -force dev`) | DONE |
| W-PROM.4.0c | Verify: `just infra-plan` shows only 5 new MCP resources (0 changes, 0 destroy) | DONE |

### Phase 1: Xtask Workspace Refactoring (from ultraplan)

Decouple "which workspace/environment to target" from "which AWS credentials to use."

| ID | Task | Status |
|---|---|---|
| W-PROM.4.1 | Refactor `xtask/src/infra/tofu.rs`: split `profile` into `workspace` + `aws_profile` | DONE |
| W-PROM.4.2 | Add `select_workspace(dir, workspace, aws_profile)` helper with auto-create | DONE |
| W-PROM.4.3 | Pass `-var environment=<ws>` when workspace is not `"default"` | DONE |
| W-PROM.4.4 | Update `xtask/src/infra/mod.rs` enum: rename `profile` → `workspace` + add `aws_profile` | DONE |
| W-PROM.4.5 | Update justfile: `infra-plan WORKSPACE="default"`, always `aws-check deploy-baba` | DONE |

Key mapping: `workspace=default` → `environment=prod` (variable default); `workspace=dev` → `-var environment=dev`.

### Phase 1.5: Deploy Recipe Alignment

Align all justfile deploy recipes with the workspace convention established in Phase 1.

| ID | Task | Status |
|---|---|---|
| W-PROM.4.5a | Generalize xtask `deploy lambda`: `--package` flag, parameterized `--function` | DONE |
| W-PROM.4.5b | All justfile `*-deploy` recipes use `cargo xtask deploy lambda` with `ENV` param | DONE |
| W-PROM.4.5c | All Lambda function names follow `deploy-baba-{env}-{service}` convention | DONE |
| W-PROM.4.5d | `lambda-deploy-all` and per-service recipes accept and pass `ENV` (default `prod`) | DONE |

Key mapping: `ENV=prod` → function `deploy-baba-prod[-service]`; `ENV=dev` → function `deploy-baba-dev[-service]`. Global `PROFILE` (default `deploy-baba`) handles AWS credentials independently.

### Phase 2: Infra Parameterization

All infra `.tf` files use `${local.lambda_function_name}` (= `${project}-${env}`) for naming.

| ID | Task | Status |
|---|---|---|
| W-PROM.4.6 | Parameterize email Lambda: `${local.lambda_function_name}-email` + log group + IAM role | DONE |
| W-PROM.4.7 | Parameterize llm-proxy Lambda: `${local.lambda_function_name}-llm-proxy` + log group + IAM role | DONE |
| W-PROM.4.8 | Parameterize mcp-gateway Lambda: `${local.lambda_function_name}-mcp-gateway` + log group + IAM role | DONE |
| W-PROM.4.9 | Parameterize S3 buckets: SPA = `deploy-baba-${env}-spa-${acct}`, assets = `${assets_bucket_prefix}-assets-${acct}` | DONE |
| W-PROM.4.10 | Parameterize API Gateway: `${local.lambda_function_name}-contact-api` + dev CORS origin | DONE |
| W-PROM.4.11 | Singletons gated: `is_prod_acm`, `is_prod_cdn`, `is_prod_vpc`, `is_prod` (ci-oidc) | DONE |
| W-PROM.4.12 | OpenTofu `moved` blocks for renamed prod resources | N/A — resources created with parameterized names from the start |
| W-PROM.4.13 | CI deploy dev role targets `${project}-dev*` Lambda + `${project}-dev-spa-*` S3 | DONE |

### Phase 3: Dev Workspace Initialization

Dev workspace exists (`tofu workspace list` shows `dev`). CI deploys to dev on every push to main.

| ID | Task | Status |
|---|---|---|
| W-PROM.4.14 | `just infra-plan dev` → review plan | DONE |
| W-PROM.4.15 | `just infra-apply dev` → create dev infrastructure | DONE |
| W-PROM.4.16 | Create `deploy-baba/dev/deploy-config` secret with dev resource names | DONE — CI reads it on every deploy |
| W-PROM.4.17 | Verify: deploy to dev, smoke test | DONE — deploy-dev.yml runs /health check |

### Phase 4: Promote Command

| ID | Task | Status |
|---|---|---|
| W-PROM.4.18 | Create `xtask/src/deploy/promote.rs`: download dev Lambda zips, upload to prod | DONE — covers all 10 zip-based services (ui, agent, email, llm-proxy, auth, portfolio, admin, contact, rag, mcp-gateway); pdf is Docker/ECR-image based and out of scope (2026-09-10) |
| W-PROM.4.19 | S3 server-side copy (dev SPA → prod SPA, preserving cache-control + stale deletion) | DONE |
| W-PROM.4.20 | CloudFront invalidation + smoke test in promote flow | DONE |
| W-PROM.4.21 | Release tag creation (vX.Y.Z) on successful promote (`--skip-tag` to opt out) | DONE |
| W-PROM.4.22 | `just promote` recipe in justfile | DONE |

### Phase 5: CI/CD Updates

| ID | Task | Status |
|---|---|---|
| W-PROM.4.23 | `deploy-dev.yml` reads `deploy-baba/dev/deploy-config`, targets dev Lambdas | DONE — now builds+deploys all 10 zip-based services, not just ui/agent (2026-09-10) |
| W-PROM.4.24 | `promote.yml` workflow_dispatch: artifact promote (rebuild fallback kept in `deploy-prod.yml`) | DONE — `deploy-prod.yml` rebuild fallback extended to the same 10-service set for parity (2026-09-10) |
| W-PROM.4.25 | Remove auto-promote tag from deploy-dev.yml (line 186-187); promotion is now explicit | DONE — auto-tag-promote step removed from `deploy-dev.yml`; `just promote` / `promote.yml` workflow_dispatch is now the sole path to prod (2026-09-10) |
| W-PROM.4.26 | End-to-end: push to main → CI → dev deploy → `just promote` → prod live | TODO — `ci-oidc.tf` IAM already wildcards `${project}-{env}*`, no policy change needed; still requires `just infra-apply` (to land the new `deploy-config` secret fields) and a manual end-to-end test |

## W-PROM.4b CDN Ownership Separation (2026-09-12)

Previously `aws_cloudfront_distribution.dev` and the `dev.sislam.com` Route53
records were gated `is_prod_cdn` — i.e. owned by the **prod** workspace — while
their origins were **dev** workspace resources, bridged by hand-pasted
`terraform.tfvars` values (`dev_ui_lambda_url`, `dev_auth_lambda_url`,
`dev_apigw_endpoint`, `dev_cloudfront_id`). Those were empty, so every
`just infra-apply` failed with `origin.N.domain_name must not be empty`: a prod
apply broken by dev resources.

Now each workspace owns its own distribution and resolves origins from its own
state (`cdn.tf`: `is_dev_cdn`, `manages_cdn`, `cdn_name_suffix`). The shared
wildcard cert is read via `data.aws_acm_certificate.wildcard` in non-prod
(`acm.tf`, `local.cdn_certificate_arn`); the hosted zone via the existing
`data.aws_route53_zone.main`. All four bootstrap variables are deleted. Prod
keeps unsuffixed OAC/policy names so its live distribution is undisturbed.

**Apply order matters for the one-time transition** — `dev_a`/`dev_aaaa` are
currently in *prod* state:

1. `just infra-apply` — drops the `dev.sislam.com` alias from `main` and
   releases the dev records from prod state (dev.sislam.com goes dark).
2. `just infra-apply dev` — creates the dev distribution + records
   (CloudFront propagation ~5–15 min).

### ⚠️ Open hazard: prod-named resource in dev state

`aws_cloudwatch_event_rule.backup_schedule` in the **dev** workspace state
points at the live **prod** rule (`deploy-baba-prod-backup-schedule`, confirmed
by ARN). A `just infra-apply dev` would replace it — destroying prod's backup
schedule — before creating the dev-named one. Prod state legitimately owns that
same rule, so the fix is to drop the stale dev pointer (no AWS change):

```
tofu -chdir=infra workspace select dev
tofu -chdir=infra state rm aws_cloudwatch_event_rule.backup_schedule
```

Do this **before** the first `just infra-apply dev`. Every other resource in dev
state is correctly `deploy-baba-dev-*` named.

## W-PROM.5 Implementation Notes

### Lambda Promotion (No Rebuild)

```rust
// xtask/src/deploy/promote.rs
async fn promote_lambda(client: &LambdaClient, dev_fn: &str, prod_fn: &str) -> Result<()> {
    // Get dev function code location
    let dev = client.get_function().function_name(dev_fn).send().await?;
    let code_url = dev.code().unwrap().location().unwrap();
    
    // Download the zip
    let zip_bytes = reqwest::get(code_url).await?.bytes().await?;
    
    // Upload to prod
    client.update_function_code()
        .function_name(prod_fn)
        .zip_file(Blob::new(zip_bytes))
        .architectures(Architecture::Arm64)
        .send().await?;
    
    // Wait for update
    client.get_function_configuration()
        .function_name(prod_fn)
        .send().await?;
    // ... poll LastUpdateStatus == Successful
    Ok(())
}
```

### S3 Promotion

```rust
async fn promote_spa(s3: &S3Client, dev_bucket: &str, prod_bucket: &str) -> Result<()> {
    // List dev objects, copy each to prod
    // Use CopySource to avoid downloading/uploading (server-side copy)
    // Then delete objects in prod that don't exist in dev
}
```

### Dev Workspace EFS Consideration

Dev needs its own EFS filesystem for database isolation. The dev Lambda connects to dev EFS via the **same VPC** and **shared VPC endpoints**. Security groups are per-environment (created per workspace). Cost: ~$0.30/month for dev EFS provisioning.

## W-PROM.6 Test Strategy

1. `just infra-plan dev` → shows only dev resources to create (no prod resources, no singletons)
2. `just infra-plan` → shows 0 changes (prod unchanged)
3. `just promote` on fresh dev deploy → prod /health returns 200 with same version as dev
4. S3 sync integrity: diff dev and prod SPA bucket contents after promote (should be identical)
5. Lambda code hash: compare `CodeSha256` between dev and prod functions post-promote

## W-PROM.7 Cross-References

- → ADR-029 (this decision record)
- → ADR-020 (GitHub OIDC — CI roles need updating)
- → ADR-021 (release tagging — promote creates tags)
- → W-CI (CI pipeline changes)
- → W-OTF (OpenTofu workspace management)
- → W-XT (xtask promote subcommand)
- → W-MCP (MCP gateway Lambda parameterization)
