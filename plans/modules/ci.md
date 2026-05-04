# W-CI: CI/CD Pipeline
**Path:** `.github/workflows/` | **Status:** WIP
**Coverage floor:** n/a | **Depends on:** W-OTF, W-WEB, W-XT | **Depended on by:** (deployed binaries + SPA)

## W-CI.1 Purpose

Provide an automated CI/CD pipeline that:
1. Validates code on every PR and push to `main` (format, lint, test, tofu validate).
2. Auto-deploys to the dev environment on every successful merge to `main`.
3. Gates prod deployment behind a conventional-commits release tag + GitHub `production` environment manual approval.

No long-lived AWS credentials in GitHub secrets — authentication is via GitHub OIDC (ADR-020).

## W-CI.2 Public Surface

Three workflow files:

| File | Trigger | Steps |
|---|---|---|
| `.github/workflows/ci.yml` | push to `main`, all PRs | rust (fmt/clippy/test), web (typecheck/test/build — conditional on web/package.json), tofu (fmt/validate), docs |
| `.github/workflows/deploy-dev.yml` | `workflow_run: [CI] success` on `main` | OIDC assume → lambda-build → lambda-update → smoke test → [C.2] web-build → s3-sync → lambda-sync-spa → release tag dev-vX.Y.Z |
| `.github/workflows/deploy-prod.yml` | `push: tags: ['v*']` | `production` env gate (manual approval) → OIDC assume → lambda-build → lambda-update → [C.2] web-build → s3-sync → lambda-sync-spa → smoke test |

Release tagging is handled by `xtask/src/release/` (W-XT addition — see W-CI.4 items).

## W-CI.3 Implementation Notes

**OIDC roles** (`infra/ci-oidc.tf`):
- `deploy-baba-ci-deploy-dev` — trust: `refs/heads/main`; permissions: `lambda:UpdateFunctionCode`, `lambda:InvokeFunction`, `s3:PutObject`/`s3:DeleteObject` on the dev SPA bucket prefix.
- `deploy-baba-ci-deploy-prod` — trust: `refs/tags/v*`; same permissions scoped to prod Lambda + prod SPA bucket.

**Web job conditional:** The `web` job in `ci.yml` has:
```yaml
if: ${{ hashFiles('web/package.json') != '' }}
```
This makes the job a no-op until Phase D.1 lands `web/`.

**pnpm caching:** Uses `pnpm/action-setup@v5` + `actions/setup-node@v5` with `cache: pnpm` and `cache-dependency-path: web/pnpm-lock.yaml`.

**Lambda build:** Uses `cargo-lambda` via `pip install cargo-lambda` (same as njnewsroomproject). Builds aarch64 binary and zips.

**SPA sync (C.2+):**
```yaml
- run: pnpm --dir web install --frozen-lockfile
- run: pnpm --dir web run build
- run: aws s3 sync web/dist/ s3://${SPA_BUCKET}/${GITHUB_SHA}/ \
         --delete --cache-control "public,max-age=31536000,immutable" --exclude "index.html"
- run: aws s3 cp web/dist/index.html s3://${SPA_BUCKET}/${GITHUB_SHA}/index.html \
         --cache-control "no-cache"
- run: |
    PAYLOAD=$(printf '{"action":"sync-spa","sha":"%s"}' "${GITHUB_SHA}" | base64)
    aws lambda invoke --function-name ${UI_FN_NAME} --payload "${PAYLOAD}" /tmp/sync-resp.json
    grep -q '"status":"ok"' /tmp/sync-resp.json
```

**Release tagging (deploy-dev.yml):**
```yaml
- run: git config user.name "github-actions[bot]"
- run: git config user.email "github-actions[bot]@users.noreply.github.com"
- run: |
    if [ -n "$(git status --porcelain)" ]; then
      echo "::error::Worktree dirty before tag"; exit 1
    fi
- run: cargo run -q -p xtask -- release tag --kind dev --push
```
Requires `permissions: contents: write` and `fetch-depth: 0` checkout.

**GitHub variables used** (set in repo Settings → Variables):
- `CI_DEPLOY_DEV_ROLE_ARN` — ARN of `deploy-baba-ci-deploy-dev`
- `CI_DEPLOY_PROD_ROLE_ARN` — ARN of `deploy-baba-ci-deploy-prod`
- `DEV_UI_FN_NAME` — Lambda function name for dev environment
- `PROD_UI_FN_NAME` — Lambda function name for prod environment
- `DEV_SPA_BUCKET` — SPA bucket name for dev
- `PROD_SPA_BUCKET` — SPA bucket name for prod
- `DEV_FN_URL` — Lambda Function URL for dev health check
- `PROD_FN_URL` — Lambda Function URL for prod health check

## W-CI.4 Work Items

| ID | Task | Status | Notes |
|---|---|---|---|
| W-CI.4.1 | `xtask/src/release/{mod,git,version,changelog}.rs` | DONE | 23 unit tests; `just release-next/tag/promote` wired |
| W-CI.4.2 | Wire `release` subcommand in `xtask/src/main.rs` | DONE | `spawn_blocking` wrapper since release ops are sync |
| W-CI.4.3 | Add `just release-next`, `release-tag`, `release-promote` recipes | DONE | Justfile additions |
| W-CI.4.4 | `infra/ci-oidc.tf` — two IAM OIDC roles | DONE | `deploy-baba-ci-deploy-dev` + `deploy-baba-ci-deploy-prod`; outputs role ARNs |
| W-CI.4.5 | Provision dev Lambda workspace (`tofu workspace new dev`) | TODO | Requires `tofu workspace new dev` + apply (manual one-time) |
| W-CI.4.6 | Update `.github/workflows/ci.yml` — add web + tofu jobs | DONE | web job conditional on `web/package.json`; tofu fmt+validate |
| W-CI.4.7 | New `.github/workflows/deploy-dev.yml` (C.1 — Lambda only) | DONE | OIDC + lambda-build + lambda-update + health check + release tag |
| W-CI.4.8 | New `.github/workflows/deploy-prod.yml` | DONE | `production` env gate (manual approval) + same deploy flow |
| W-CI.4.9 | Set GitHub Variables (manual, one-time) | DONE | GH Variables replaced by SM fetch (RESOLVED 2026-05-04) |
| W-CI.4.10 | Create GitHub `production` environment + Required Reviewers | TODO | Repo Settings → Environments → production |
| W-CI.4.11 | Extend deploy-dev.yml with SPA sync (C.2) | DONE | pnpm build → s3 sync → lambda invoke sync-spa → assert ok; worktree-clean guard before tag |
| W-CI.4.12 | Extend deploy-prod.yml with SPA sync (C.2) | DONE | Same as dev; no tag step (prod triggered by tag push) |
| W-CI.4.13 | Local deploy pipeline: `xtask deploy spa`, `just deploy-full/spa-deploy/lambda-wait`, `/deploy --full` skill | DONE | Steps 2–6 in Rust; opt-in `--tag`; prod confirmation gate in skill |

## W-CI.5 Test Strategy

- W-CI.4.1–4.3 (xtask release): `just release-next` on clean repo prints a version; `just release-tag KIND=dev PUSH=0` creates a local-only tag; re-run is a no-op.
- W-CI.4.6–4.8 (workflows): push a feature branch → `ci.yml` runs all jobs green; merge to `main` → `deploy-dev.yml` runs, Lambda updates, smoke curl returns `{"status":"ok","version":...}`, `dev-vX.Y.Z` tag appears on GitHub.
- W-CI.4.11–4.12 (SPA sync): merge after D.1 lands → S3 sync succeeds → Lambda invoke `sync-spa` returns `"status":"ok"` → `curl ${DEV_FN_URL}/` returns the SPA `index.html`.

## W-CI.6 Cross-References

- → ADR-001 (justfile-only interface)
- → ADR-007 (OpenTofu — ci-oidc.tf managed by tofu)
- → ADR-019 (SPA deploy added in C.2)
- → ADR-020 (OIDC CI decision)
- → ADR-021 (release tagging decision)
- → `plans/modules/xtask.md` (W-XT — release subcommand lives in xtask)
- → `plans/modules/web.md` (W-WEB — web/ must exist before C.2)
- → `plans/cross-cutting/quality-gates.md` (gate definitions mapped to CI jobs)
