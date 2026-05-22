# W-PROM: Environment Separation & Artifact Promotion

**Path:** `xtask/src/deploy/promote.rs`, `infra/*.tf`, `.github/workflows/`  
**Status:** TODO  
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

The `default` workspace has **129 resources fully tracked** (188KB state in S3). No imports needed.
The `dev` workspace has 11 resources pointing to the **same** physical AWS resources as default (verified: same Cognito pool ID, same secret ARNs). It needs to be cleaned and re-initialized.

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

### Phase 2: Infra Parameterization

| ID | Task | Status |
|---|---|---|
| W-PROM.4.6 | Parameterize email Lambda: `${project}-${env}-email` + log group + IAM role | TODO |
| W-PROM.4.7 | Parameterize llm-proxy Lambda: `${project}-${env}-llm-proxy` + log group + IAM role | TODO |
| W-PROM.4.8 | Parameterize mcp-gateway Lambda: `${project}-${env}-mcp-gateway` + log group + IAM role | TODO |
| W-PROM.4.9 | Parameterize assets S3 bucket: `${project}-${env}-assets-${acct}` | TODO |
| W-PROM.4.10 | Parameterize API Gateway: `${project}-${env}-contact-api` + add dev CORS origin | TODO |
| W-PROM.4.11 | Add `count = var.environment == "prod" ? 1 : 0` on singletons (OIDC, VPC endpoints, ACM, CloudFront, CI roles) | TODO |
| W-PROM.4.12 | Use OpenTofu `moved` blocks to migrate renamed prod resources in state (avoids destroy/recreate) | TODO |
| W-PROM.4.13 | Fix CI deploy dev role (`ci-oidc.tf:70`) to target `${project}-dev` Lambda + dev S3 bucket | TODO |

### Phase 3: Dev Workspace Initialization

| ID | Task | Status |
|---|---|---|
| W-PROM.4.14 | `just infra-plan dev` → review plan (should show only dev-specific resources) | TODO |
| W-PROM.4.15 | `just infra-apply dev` → create dev infrastructure | TODO |
| W-PROM.4.16 | Create `deploy-baba/dev/deploy-config` secret with dev resource names | TODO |
| W-PROM.4.17 | Verify: deploy to dev, smoke test `dev.sislam.com/health` | TODO |

### Phase 4: Promote Command

| ID | Task | Status |
|---|---|---|
| W-PROM.4.18 | Create `xtask/src/deploy/promote.rs`: download dev Lambda zips, upload to prod | TODO |
| W-PROM.4.19 | Add S3 server-side copy (dev SPA → prod SPA, preserving cache-control) | TODO |
| W-PROM.4.20 | Add CloudFront invalidation + smoke test to promote | TODO |
| W-PROM.4.21 | Add release tag creation (vX.Y.Z) on successful promote | TODO |
| W-PROM.4.22 | Wire `just promote` recipe in justfile | TODO |

### Phase 5: CI/CD Updates

| ID | Task | Status |
|---|---|---|
| W-PROM.4.23 | Update `deploy-dev.yml`: read `deploy-baba/dev/deploy-config`, target dev Lambdas | TODO |
| W-PROM.4.24 | Update `deploy-prod.yml`: replace full rebuild with promote (keep rebuild as fallback) | TODO |
| W-PROM.4.25 | Remove auto-promote tag from deploy-dev.yml (line 139); promotion is now explicit | TODO |
| W-PROM.4.26 | End-to-end: push to main → CI → dev deploy → `just promote` → prod live | TODO |

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
