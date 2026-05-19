# CI/CD Pipeline

Last updated: 2026-05-19

GitHub Actions workflows with OIDC-based AWS deployment. No long-lived access keys ([ADR-020](../plans/adr/ADR-020-github-actions-ci-oidc.md)).

## Workflows

### ci.yml — Quality Gate

**Trigger:** Push or PR to `main`.

Five parallel jobs must all pass:

| Job | What it checks |
|-----|---------------|
| **Format & Lint** | `cargo fmt --check` + `cargo clippy -D warnings` |
| **Test** | `cargo test --workspace` on ubuntu + macOS |
| **Documentation** | `cargo doc --no-deps --workspace` builds without warnings |
| **OpenTofu** | `tofu fmt -check` + `tofu init -backend=false` + `tofu validate` |
| **Web / SPA** | TypeScript types from OpenAPI spec → typecheck → test → coverage → build |

The Web job generates TypeScript types from the Rust OpenAPI spec as part of CI:
```bash
cargo run -p api-openapi --bin emit-spec > web/openapi.json
pnpm exec openapi-typescript openapi.json -o src/api/types.gen.ts
```

This ensures the SPA's type-safe API client stays in sync with the backend.

### deploy-dev.yml — Dev Deployment

**Trigger:** `workflow_run` — fires when CI succeeds on `main`.

Steps:
1. Assume the dev OIDC role (`CI_DEPLOY_DEV_ROLE_ARN` from GitHub Variables)
2. Load deploy config from Secrets Manager (`deploy-baba/prod/deploy-config`)
3. Build Lambda binary via `cargo-lambda` (aarch64)
4. Update Lambda function code
5. Build SPA (`pnpm --dir web run build`)
6. Sync `web/dist/` to S3 SPA bucket
7. Invalidate CloudFront cache
8. Tag release (`dev-vX.Y.Z`)

**Deploys to:** dev.sislam.com

### deploy-prod.yml — Production Deployment

**Trigger:** Tag push matching `v*`.

Same build steps as dev, but:
- Uses a separate prod OIDC role (`CI_DEPLOY_PROD_ROLE_ARN`)
- Requires the `production` GitHub environment (approval gate)

**Deploys to:** sislam.com

## OIDC Authentication

Instead of storing AWS access keys in GitHub Secrets, the workflows use OpenID Connect federation:

1. GitHub generates a short-lived OIDC token for each workflow run
2. AWS IAM validates the token against the GitHub OIDC provider
3. The workflow assumes an IAM role with scoped permissions

Two IAM roles:
- `deploy-baba-ci-dev-role` — can update dev Lambda + S3 + CloudFront
- `deploy-baba-ci-prod-role` — same permissions scoped to prod resources

Configured in `infra/ci-oidc.tf`. Only the bootstrap role ARN is stored as a GitHub Variable — everything else comes from Secrets Manager at runtime.

## Deploy Configuration

Non-secret deployment identifiers are stored in a single Secrets Manager secret (`deploy-baba/prod/deploy-config`):

| Key | Value |
|-----|-------|
| `spa_bucket` | S3 bucket name for SPA assets |
| `cloudfront_id` | CloudFront distribution ID |
| `ui_fn_name` | Lambda function name |
| `fn_url` | Lambda Function URL |

This avoids scattering identifiers across GitHub Variables and keeps them alongside the infrastructure that creates them. See [DRL-2026-05-04-sislam-outage](../plans/drift/DRL-2026-05-04-sislam-outage.md) for context on why this design was chosen.

## Release Tagging

- **Dev:** automatic `dev-vX.Y.Z` tags on successful deploy
- **Prod:** push a `v*` tag to trigger production deployment

## Cross-References

- [ADR-020](../plans/adr/ADR-020-github-actions-ci-oidc.md) — GitHub Actions CI with OIDC
- [ADR-021](../plans/adr/ADR-021-release-tagging.md) — Automated release tagging
- [plans/modules/ci.md](../plans/modules/ci.md) — CI module plan and work items
- `infra/ci-oidc.tf` — IAM OIDC provider + roles
