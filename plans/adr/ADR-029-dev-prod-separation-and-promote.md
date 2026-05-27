# ADR-029: Dev/Prod Environment Separation with Artifact Promotion

**Status:** Proposed  
**Date:** 2026-05-21  
**Modules:** W-CI, W-OTF, W-XT, W-MCP  

## Context

Today, dev and prod deploy to **the same infrastructure**: same Lambda (`deploy-baba-prod`), same S3 SPA bucket, same CloudFront distribution. The `dev` OpenTofu workspace exists (W-CI.4.5) but was never populated with environment-specific resources. `deploy-dev.yml` reads `deploy-baba/prod/deploy-config` and deploys to prod-named resources. The CI deploy dev IAM role (`ci-oidc.tf:70`) even targets `deploy-baba-prod` Lambda.

This means:
- Every CI deploy to "dev" overwrites production
- No safe e2e testing environment exists
- `dev.sislam.com` routes to the same origin as `sislam.com`

We need true dev/prod separation with **artifact promotion** (build once on dev, copy to prod) to enable safe e2e testing before production.

## Decision

### 1. OpenTofu Workspaces for Environment Isolation

Use `default` workspace for prod, `dev` workspace for dev. `var.environment` is already used in ~70% of resource names — fix the remaining ~30%.

### 2. Singleton Resources (Managed in `default` Workspace Only)

These resources are per-account or per-VPC and must not be duplicated:

| Resource | File | Why singleton |
|----------|------|---------------|
| `aws_iam_openid_connect_provider.github` | `ci-oidc.tf` | Per-account |
| `aws_iam_role.ci_deploy_dev` + policy | `ci-oidc.tf` | Per-account CI roles |
| `aws_iam_role.ci_deploy_prod` + policy | `ci-oidc.tf` | Per-account CI roles |
| `aws_vpc_endpoint.lambda` | `vpc-endpoints.tf` | Per-VPC, $7.30/month |
| `aws_vpc_endpoint.secretsmanager` | `vpc-endpoints.tf` | Per-VPC, $7.30/month |
| `aws_vpc_endpoint.s3` | `vpc-endpoints.tf` | Per-VPC, free |
| `aws_acm_certificate.wildcard` | `acm.tf` | Shared across subdomains |

Guard these with `count = var.environment == "prod" ? 1 : 0`. Dev workspace data-sources or hardcoded ARNs reference the existing resources.

### 3. Resources That Need Environment Parameterization

| Resource | Current name | New name | File |
|----------|-------------|----------|------|
| Email Lambda | `${project}-email` | `${project}-${env}-email` | `email-lambda.tf` |
| LLM-proxy Lambda | `${project}-llm-proxy` | `${project}-${env}-llm-proxy` | `llm-proxy-lambda.tf` |
| MCP gateway Lambda | `${project}-mcp-gateway` | `${project}-${env}-mcp-gateway` | `mcp-gateway-lambda.tf` |
| Assets S3 bucket | `${project}-assets-${acct}` | `${project}-${env}-assets-${acct}` | `s3-assets.tf` |
| API Gateway | `${project}-contact-api` | `${project}-${env}-contact-api` | `apigateway.tf` |
| All helper IAM roles | `${project}-<name>-execution-role` | `${project}-${env}-<name>-execution-role` | respective `.tf` files |
| All helper log groups | `/aws/lambda/${project}-<name>` | `/aws/lambda/${project}-${env}-<name>` | respective `.tf` files |

### 4. CloudFront: Shared Distribution with Per-Environment Origins

Keep a single CloudFront distribution (both `sislam.com` and `dev.sislam.com` are aliases). Use a **CloudFront Function** (free, 2M invocations/month) at viewer-request to route by `Host` header:

- `dev.sislam.com` → dev origins (dev Lambda Function URL, dev S3 SPA/assets, dev API Gateway)
- `sislam.com` / `www.sislam.com` → prod origins

This avoids duplicating the distribution ($0) while providing clean separation. CloudFront is managed in the prod workspace only (singleton). The dev workspace outputs its origin URLs; the promote workflow or a manual step updates the CloudFront Function's routing table.

**Alternative considered:** Separate CloudFront distributions per environment. Rejected — adds complexity, a second ACM validation, and potential free-tier split. A single distribution with hostname routing is simpler and cheaper.

### 5. `just promote` — Artifact Promotion (Not Rebuild)

Promote copies artifacts from dev to prod without rebuilding. This guarantees prod runs the exact code that passed e2e testing.

```
just promote
  1. For each Lambda (ui, email, llm-proxy, mcp-gateway):
     - Download code zip from dev function
     - Upload to prod function
     - Wait for active
  2. S3 sync: dev SPA bucket → prod SPA bucket
  3. S3 sync: dev assets bucket → prod assets bucket (if assets differ)
  4. Invalidate prod CloudFront paths
  5. Smoke test prod /health
  6. Create vX.Y.Z release tag
```

### 6. Xtask Refactoring — Decouple Workspace from AWS Profile

Rename `--profile` to `--workspace` in xtask infra commands. Add separate `--aws-profile` for credentials. The justfile `PROFILE` parameter becomes `WORKSPACE`.

```rust
// xtask/src/infra/tofu.rs
fn make_cmd(dir: &str, aws_profile: Option<&str>) -> Command { ... }
fn select_workspace(dir: &str, workspace: &str, aws_profile: Option<&str>) -> Result<()> { ... }
```

### 7. CI/CD Workflow Updates

- `deploy-dev.yml`: read `deploy-baba/dev/deploy-config`, target dev-named Lambdas
- `deploy-prod.yml`: remove rebuild — instead, invoke `just promote` (or equivalent AWS CLI)
- `ci-oidc.tf`: fix dev role to target `${project}-dev` Lambda and dev S3 bucket

## Cost Impact

| Item | Current | After | Delta |
|------|---------|-------|-------|
| VPC endpoints | $14.60/month | $14.60/month (shared) | $0 |
| CloudFront | Free tier | Free tier (shared) | $0 |
| Dev Lambda | N/A | Free tier (1M req) | $0 |
| Dev EFS | N/A | ~$0.30/month | +$0.30 |
| Dev S3 buckets | N/A | Free tier (5GB) | $0 |
| Dev Cognito | N/A | Free tier | $0 |
| **Total** | ~$15.40/month | ~$15.70/month | **+$0.30** |

## Consequences

- Dev deployments no longer overwrite production
- E2e tests can safely target `dev.sislam.com`
- Promotion guarantees prod runs tested code
- VPC endpoints are shared — zero additional cost for the most expensive resources
- Helper Lambda renames require one-time `tofu import` for existing prod resources
- CI workflows need updated secret paths and function names
