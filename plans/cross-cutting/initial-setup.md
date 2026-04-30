# Initial Setup — deploy-baba

First-run guide for a developer cloning this repo. Assumes macOS + zsh.

Adapted from `~/njnewsroomproject/plans/cross-cutting/initial-setup.md`.

---

## Prerequisites

```bash
brew install rustup just opentofu awscli
brew install --cask docker
rustup install stable
rustup target add aarch64-unknown-linux-gnu
cargo install cargo-lambda
```

Node + pnpm (required for the Vite SPA, ADR-019):

```bash
nvm install 20       # or: brew install node@20
npm install -g pnpm  # or: corepack enable
```

Verify all prerequisites:

```bash
just dev-doctor      # runs scripts/dev-doctor.sh — prints a status table
```

---

## AWS

```bash
aws configure sso  # profile name: deploy-baba; account: 227655493757; role: AdministratorAccess
aws sso login --profile deploy-baba
aws sts get-caller-identity --profile deploy-baba  # → 227655493757
```

See `docs/aws-setup.md` for IAM setup detail (policies, user, etc.).

---

## One-time infrastructure bootstrap

```bash
just infra-bootstrap  # runs scripts/bootstrap-tfstate.sh
```

This creates:
- S3 bucket `deploy-baba-tfstate-${ACCOUNT_ID}` (versioning enabled) — OpenTofu state backend
- DynamoDB table `terraform-lock` (shared with other projects; idempotent)

Run once per AWS account. Safe to re-run; subsequent calls skip already-existing resources.

---

## First-time workspace bootstrap

```bash
just dev                     # cargo fmt + clippy + test (Rust workspace)
pnpm --dir web install       # web SPA dependencies
just web-typecheck           # confirm SPA builds
just lambda-build            # confirm Rust Lambda builds for arm64
```

---

## Local dev

Two terminals:

```bash
# Terminal 1 — local Rust Lambda binary on :3000
just ui

# Terminal 2 — Vite dev server on :5173 (proxies /api to :3000)
just web
```

Or combined via:

```bash
just dev-stack    # starts both in one terminal (Ctrl-C kills both)
```

Browse `http://localhost:5173` for the SPA (Vite HMR), or `http://localhost:3000` for the Lambda binary serving `web/dist/` (run `just web-build` first).

---

## OpenTofu infra (after bootstrap)

```bash
just infra-init           # tofu init with state backend
just infra-plan           # plan prod workspace changes
just infra-plan-dev       # plan dev workspace changes
```

Apply only when the plan looks correct:
```bash
just infra-apply-dev      # apply to dev
just infra-apply          # apply to prod (requires manual review)
```

---

## Deploy (after infra is applied)

```bash
# Manual (developer laptop)
just lambda-deploy dev     # Rust binary update to dev Lambda
just release-promote       # create vX.Y.Z tag → triggers deploy-prod.yml

# Automated (CI handles this on merge to main)
# deploy-dev.yml auto-deploys to dev + tags dev-vX.Y.Z after every successful CI run
```

---

## Agent cache

```bash
just cache-status    # compare .agent-cache/index.json SHA to HEAD
just cache-refresh   # rebuild cache from current git state
just cache-clear     # delete cache to force full re-scan next session
```

---

## Cross-References
- `plans/modules/dev-environment.md` (W-DEV work items)
- `docs/aws-setup.md` (IAM policy + profile config)
- `plans/adr/ADR-022-initial-setup.md` (ADR for this guide)
- `plans/adr/ADR-019-spa-replaces-askama.md` (pnpm/Node prerequisite rationale)
- `plans/INDEX.md`
