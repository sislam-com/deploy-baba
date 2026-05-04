# DRL-2026-05-04-sislam-outage

**Date:** 2026-05-04
**Severity:** high
**Affected modules:** W-WEB, W-OTF, W-CI, W-SEC

## Summary

Both `sislam.com` and `dev.sislam.com` returned HTTP 404 for all page requests after the D.4/D.5 architecture flip (commit `7b20f3f`). The new Lambda binary expected to serve SPA assets from EFS at `/mnt/spa/active`, but the second `file_system_config` block for the SPA EFS access point was never applied (removed in DRL-2026-05-01-infra-plan-blockers.md as "matches actual AWS state"). The SPA bucket existed but was empty. Every request 404'd via `ServeFile::new("/mnt/spa/active/index.html")`. The incident was discovered 2026-05-04 when the user ran `just spa-deploy` and got `SPA_BUCKET env var not set` from the xtask (the xtask read from local env, not from SM, which hadn't yet been wired up).

Resolution: switched to CloudFront→S3 direct serving (no Lambda in the SPA asset path), populated a self-managed `deploy-baba/prod/deploy-config` SM secret, deployed the new Lambda binary (SPA serving code removed), and ran the SPA sync. Both domains restored within ~30 minutes.

## Entries

| ID | Finding | Status | Resolution |
|----|---------|--------|-----------|
| DRL-OUTAGE-1 | Lambda served SPA from EFS `/mnt/spa/active` but the EFS mount was never applied; every page 404'd | RESOLVED | Dropped EFS sync; CloudFront→S3 direct serving added in `infra/cdn.tf`; Lambda no longer handles static assets |
| DRL-OUTAGE-2 | `SPA_BUCKET env var not set` error when running `just spa-deploy` — deploy identifiers had no source of truth | RESOLVED | Created `deploy-baba/prod/deploy-config` SM secret (self-populated by `tofu apply`); `SpaEnvConfig::from_secrets_manager()` added to `xtask/src/deploy/spa.rs`; env-var fallback kept for local dev |
| DRL-OUTAGE-3 | GitHub Variables `DEV_SPA_BUCKET`, `DEV_UI_FN_NAME`, `DEV_FN_URL`, `PROD_*` variants were TODO (W-CI.4.9) but the design required them to be set manually | RESOLVED | Variables removed from CI design entirely; CI workflow fetches config from SM after OIDC auth; only `CI_DEPLOY_DEV_ROLE_ARN` / `CI_DEPLOY_PROD_ROLE_ARN` remain as GitHub Variables (bootstrap) |
| DRL-OUTAGE-4 | `dev.sislam.com` Cognito callback was not registered; login from dev subdomain would fail | RESOLVED | Added `https://dev.sislam.com/auth/callback` and logout URL to `infra/cognito.tf`; applied 2026-05-04 |
| DRL-OUTAGE-5 | `smoke_test` in `spa.rs` had off-by-one: `format!("{}health", fn_url.trim_end_matches('/'))` produced `.../on.awshealth` | RESOLVED | Fixed to `format!("{}/health", fn_url.trim_end_matches('/'))` |
| DRL-OUTAGE-6 | `fn_url` in SM secret was raw Lambda Function URL (requires SigV4 auth); smoke test through it always fails | RESOLVED | Changed `fn_url` in SM secret to `https://${var.domain_name}` (CloudFront URL); smoke test hits the public endpoint |

## Lessons Learned

- **Partial applies are dangerous.** When a `file_system_config` block is removed from HCL to "match actual AWS state" (DRL-2026-05-01), there is no signal that the intended state (EFS SPA mount) was never reached. Future infra changes should either be applied end-to-end or backed by an explicit "deferred" work item.
- **Deploy identifiers belong in Secrets Manager, not GitHub Variables.** SM is self-rotated by `tofu apply`; GH Variables require manual mirroring and create drift. The new pattern (SM secret populated from infra outputs) eliminates a class of "variable not set" errors.
- **SPA serving architecture: CloudFront→S3 direct is simpler than Lambda EFS sync.** The EFS sync approach (D.4) required a Lambda invocation per deploy, a custom handler, and an extra EFS access point. CloudFront→S3 OAC eliminates all of this.
- **Smoke tests should hit the public URL, not the raw Lambda Function URL.** The Lambda Function URL requires SigV4 auth; real users go through CloudFront.

## Cross-References

- → W-WEB (SPA serving architecture changed from EFS to CloudFront→S3)
- → W-OTF (infra changes applied: new CF behaviors, OAC, Cognito callback, SM secret)
- → W-CI (GitHub Variables removed from design; SM fetch added to deploy-dev/prod.yml)
- → W-SEC (deploy-config SM secret added; W-SEC now covers deploy identifiers too)
- → DRL-2026-05-01-infra-plan-blockers (root cause: EFS mount removal from D.5 session)
