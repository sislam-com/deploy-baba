# ADR-020: GitHub Actions CI with OIDC

**Status:** Accepted
**Date:** 2026-04-30
**Affected modules:** W-CI, W-OTF, W-WEB
**Imported from:** njnewsroomproject ADR-012 (adapted for deploy-baba scope)

---

## Context

CI currently runs only format/clippy/test/docs via a 47-line `.github/workflows/ci.yml` — no deploy automation. Production is reached via manual `just deploy` from a developer laptop. Two problems:

1. No automated deploy pipeline means every deploy requires SSO credentials, terminal access, and careful manual sequencing. It cannot run in parallel with code review.
2. No deployment automation means no per-merge validation — a change merged to `main` only reaches prod when someone remembers to deploy. This is fragile and leaves prod stale.

Additionally, the SPA work (ADR-019) and release management (ADR-021) require CI to: build the SPA, sync to S3, invoke the Lambda sync handler, and push git tags — all with AWS access but without long-lived IAM credentials in GitHub secrets (a security anti-pattern).

---

## Decision

CI authenticates to AWS using **GitHub OIDC** (no long-lived keys). Two IAM roles:

- `deploy-baba-ci-deploy-dev` — assumed by workflows running on `refs/heads/main` (auto-deploy on every merge).
- `deploy-baba-ci-deploy-prod` — assumed by workflows running on `refs/tags/v*`, gated by GitHub `production` environment with mandatory manual approval.

Trust policy condition (per role, `deploy-baba-ci-deploy-dev` example):
```
StringEquals: { "token.actions.githubusercontent.com:aud": "sts.amazonaws.com" }
StringLike:   { "token.actions.githubusercontent.com:sub":
                  "repo:shantopagla/deploy-baba:ref:refs/heads/main" }
```

The OIDC provider (`token.actions.githubusercontent.com`) already exists in account `227655493757` (provisioned by the shared AWS account). Reference it via `data "aws_iam_openid_connect_provider"` rather than creating a duplicate.

Workflows use `aws-actions/configure-aws-credentials@v5` with `role-to-assume`. No GitHub secret holds an AWS key.

### IAM policy scope (per role)

Minimally-scoped to the operations each workflow needs:
```
lambda:UpdateFunctionCode       on deploy-baba-{env}-ui Lambda ARN
lambda:InvokeFunction           on deploy-baba-{env}-ui (sync handler invocation)
s3:PutObject, s3:DeleteObject   on s3://deploy-baba-{env}-spa-{acct}/* prefix
```

### Three workflows

| File | Trigger | Environment |
|---|---|---|
| `.github/workflows/ci.yml` | push to `main`, all PRs | n/a |
| `.github/workflows/deploy-dev.yml` | `workflow_run: [CI] success` on `main` | `dev` (auto) |
| `.github/workflows/deploy-prod.yml` | `push: tags: ['v*']` | `production` (manual approval) |

### CI jobs (`ci.yml`)

```
rust:    cargo fmt --check → cargo clippy -D warnings → cargo test --workspace
web:     pnpm typecheck → pnpm test → pnpm build          (conditional: web/package.json exists)
tofu:    tofu fmt -check -recursive → tofu init -backend=false → tofu validate
docs:    cargo doc --no-deps --workspace
```

### Deploy-dev flow (`deploy-dev.yml`)

```
OIDC assume deploy-baba-ci-deploy-dev
→ cargo lambda build --release --target aarch64-unknown-linux-gnu
→ aws lambda update-function-code (dev Lambda)
→ curl ${DEV_FN_URL}/health (smoke test)
→ [Phase C.2+] pnpm --dir web build → aws s3 sync → aws lambda invoke sync-spa
→ cargo run -p xtask -- release tag --kind dev --push  (ADR-021)
```

### Deploy-prod flow (`deploy-prod.yml`)

```
GitHub Production environment gate (manual approval)
→ OIDC assume deploy-baba-ci-deploy-prod
→ cargo lambda build → aws lambda update-function-code (prod Lambda)
→ [Phase C.2+] pnpm build → s3 sync → lambda invoke sync-spa
→ curl ${PROD_FN_URL}/health
```

### Dev environment

A `dev` Lambda workspace (`deploy-baba-dev`) is provisioned alongside prod via `tofu workspace new dev` + parameterised `infra/lambda.tf`. Lambda Function URL provides the dev endpoint. No custom domain for dev — the raw `*.lambda-url.us-east-1.on.aws` URL is sufficient for soak-testing.

---

## Consequences

**Positive:**
- No AWS credentials in GitHub secrets. Roles scoped per branch/tag — accidental prod deploy from a feature branch is impossible.
- Every merge to `main` auto-deploys to dev and tags `dev-vX.Y.Z` (ADR-021). Dev is always current.
- Prod deployment requires: (a) `just release-promote --push` (developer intent), (b) manual approval in GitHub UI (second human-in-the-loop).
- CloudTrail shows the GitHub run that assumed the role — full audit trail.

**Negative:**
- One-time setup: trust policy `sub` conditions must match repo slug exactly. Mitigation: adapted from njnewsroomproject's working `infra/ci-oidc.tf`.
- `deploy-dev.yml` requires `contents: write` permission for the release tag push.

---

## Cross-References
- → ADR-001 (justfile-only interface — CI uses the same `just` recipes locally)
- → ADR-007 (OpenTofu manages the OIDC roles via `infra/ci-oidc.tf`)
- → ADR-019 (SPA deploy steps added in Phase C.2)
- → ADR-021 (release tagging triggered by `deploy-dev.yml`)
- → `plans/modules/ci.md` (W-CI work items)
