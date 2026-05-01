# deploy-baba — Zero-cost Rust abstractions for deployment automation
# All developer commands go through this justfile. Never call `cargo xtask` directly.

set dotenv-load := false

# Default AWS profile — pinned to stack.toml `aws.profile`.
# Recipes with a `PROFILE` parameter shadow this; argless recipes (sso-login, ui, dev-stack) use it directly.
PROFILE := "deploy-baba"

# ── Meta ──────────────────────────────────────────────────────────────────────

# List all available commands
default:
    @just --list

# ── Inner Loop (daily dev) ────────────────────────────────────────────────────

# Format all code
fmt:
    cargo xtask build format

# Run clippy (warnings = errors)
lint:
    cargo xtask build lint

# Fast compile check (no codegen)
check:
    cargo check --workspace

# Run unit tests only (fast)
test:
    cargo xtask test unit

# Run all tests including quarantine (slow, external deps)
test-all:
    cargo xtask test all

# Run tests for a single crate
test-crate CRATE:
    cargo xtask test crate {{CRATE}}

# Generate coverage report (opens in browser)
coverage:
    cargo xtask coverage report --open

# fmt + lint + test (the standard inner loop)
dev:
    just fmt && just lint && just test

# Full quality gate (fmt + lint + test + coverage floors + audit)
quality:
    cargo xtask quality all

# Build all crates (release)
build:
    cargo build --workspace --release

# ── Documentation ────────────────────────────────────────────────────────────

# Build and open rustdoc
docs:
    cargo doc --no-deps --workspace --open

# Build docs without opening (CI)
doc-check:
    cargo doc --no-deps --workspace

# ── Examples ─────────────────────────────────────────────────────────────────

# Run an example: just example 01_multi_format_config
example NAME:
    cargo run -p example_{{NAME}}

# Build the Lambda zip for aarch64 (requires cargo-lambda + aarch64 toolchain)
lambda-build:
    PATH="$HOME/.cargo/bin:$PATH" cargo lambda build --release --package deploy-baba-ui --target aarch64-unknown-linux-gnu

# Build Lambda zip + upload to the deployed function
lambda-deploy PROFILE="default":
    just aws-check {{PROFILE}} && just lambda-build && cargo xtask deploy lambda --profile {{PROFILE}}

# Build the email Lambda zip for aarch64 (separate non-VPC Lambda, handles SES sends)
# Output goes to infra/build/ so `tofu apply` (which runs with -chdir=infra) can find it
email-build:
    PATH="$HOME/.cargo/bin:$PATH" cargo lambda build --release --package email-lambda --target aarch64-unknown-linux-gnu
    mkdir -p infra/build
    zip -j infra/build/email-lambda.zip target/lambda/email-lambda/bootstrap

# Build email Lambda zip + update the deployed function
email-deploy PROFILE="default":
    just aws-check {{PROFILE}} && just email-build && aws lambda update-function-code \
        --function-name deploy-baba-email \
        --zip-file fileb://infra/build/email-lambda.zip \
        --profile {{PROFILE}}

# Verify the live deployment (curl apex + www health checks)
infra-verify DOMAIN="sislam.com":
    @echo "=== Verifying {{DOMAIN}} ==="
    curl -sI https://{{DOMAIN}} | head -1
    curl -sI https://www.{{DOMAIN}} | head -1
    curl -s https://{{DOMAIN}}/health
    @echo ""

# ── UI / Portfolio Site ──────────────────────────────────────────────────────

# Run the portfolio site locally on :3000, serving the pre-built SPA from web/dist/.
# Run `just web-build` first if web/dist/ is missing or stale.
# For hot-reloading frontend dev, use `just dev-stack` instead (Vite on :5173 + API on :3000).
ui:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f web/dist/index.html ]; then
        echo "web/dist/ missing — building SPA first..."
        just web-build
    fi
    eval "$(just dev-env)"
    env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN \
        cargo watch -x 'run --package deploy-baba-ui'

# Run the portfolio site once (no hot reload)
ui-run:
    #!/usr/bin/env bash
    if [ ! -f web/dist/index.html ]; then
        echo "web/dist/ missing — building SPA first..."
        just web-build
    fi
    cargo run --package deploy-baba-ui

# Build the UI binary only (fast check)
ui-build:
    cargo build --package deploy-baba-ui

# Tail CloudWatch logs for the deployed Lambda
ui-logs PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask aws logs --function deploy-baba-ui --profile {{PROFILE}}

# Open the live portfolio URL (reads from OpenTofu outputs)
ui-open PROFILE="default":
    cargo xtask infra output --key function_url --profile {{PROFILE}} | xargs open

# ── Utilities ────────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# Update all dependencies
update:
    cargo update

# Security audit of dependencies
audit:
    cargo audit

# ── AWS Profile ──────────────────────────────────────────────────────────────

# Validate AWS profile is configured and has required permissions
aws-check PROFILE="default":
    cargo xtask aws validate --profile {{PROFILE}}

# Print AWS setup instructions
aws-setup:
    @cat docs/aws-setup.md

# Print current caller identity
aws-whoami PROFILE="default":
    aws sts get-caller-identity --profile {{PROFILE}}

# Log in to AWS SSO. Populates ~/.aws/sso/cache — run once per workday before dev-stack/infra-plan/lambda-deploy.
sso-login:
    aws sso login --profile {{PROFILE}}

# ── Developer Environment ─────────────────────────────────────────────────────

# Print `export X=Y` lines for all env vars the local Rust binary needs.
# Fetches Cognito config from SSM (/deploy-baba/prod/cognito-*) and JWKS from the
# public Cognito endpoint. Consumed via `eval "$(just dev-env)"` in `just ui`.
# Requires a valid SSO session — run `just sso-login` first.
dev-env:
    #!/usr/bin/env bash
    set -euo pipefail
    AWS="env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN AWS_PROFILE={{PROFILE}} aws"
    pool_id=$($AWS ssm get-parameter --name /deploy-baba/prod/cognito-pool-id    --query Parameter.Value --output text)
    client_id=$($AWS ssm get-parameter --name /deploy-baba/prod/cognito-client-id --query Parameter.Value --output text)
    domain=$($AWS    ssm get-parameter --name /deploy-baba/prod/cognito-domain    --query Parameter.Value --output text)
    jwks=$(curl -fsSL "https://cognito-idp.us-east-1.amazonaws.com/${pool_id}/.well-known/jwks.json")
    echo "export AWS_PROFILE={{PROFILE}}"
    echo "export ANTHROPIC_API_KEY_ARN=root-anthropic-access-key"
    echo "export RAG_PUBLIC_ENABLED=1"
    echo "export COGNITO_POOL_ID=${pool_id}"
    echo "export COGNITO_CLIENT_ID=${client_id}"
    echo "export COGNITO_DOMAIN=${domain}"
    echo "export COGNITO_REGION=us-east-1"
    echo "export APP_DOMAIN=http://localhost:3000"
    printf 'export COGNITO_JWKS=%q\n' "${jwks}"

# Verify all prerequisites (rustup, cargo-lambda, node≥20, pnpm, tofu, AWS SSO, cache)
dev-doctor:
    bash scripts/dev-doctor.sh

# ── Web / SPA (Vite + React) ──────────────────────────────────────────────────

# Start Vite dev server on :5173 with /api proxy to :3000
web:
    pnpm --dir web dev

# Build SPA to web/dist/
web-build:
    pnpm --dir web run build

# Run Vitest unit tests
web-test:
    pnpm --dir web run test

# TypeScript type check (no emit)
web-typecheck:
    pnpm --dir web run typecheck

# ESLint
web-lint:
    pnpm --dir web run lint

# Regenerate src/api/types.gen.ts from the running local server (requires just ui on :3000)
web-types:
    pnpm --dir web run types

# Start both the Rust API server (:3000) and Vite dev server (:5173) in parallel
dev-stack:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' SIGINT SIGTERM EXIT
    env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN just ui &
    just web &
    wait

# ── Infrastructure (OpenTofu) ────────────────────────────────────────────────

# Bootstrap: create S3 state bucket + DynamoDB lock table (idempotent, run once per account)
infra-bootstrap PROFILE="default" REGION="us-east-1":
    bash scripts/bootstrap-tfstate.sh

# Preview infrastructure changes
infra-plan PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask infra plan --profile {{PROFILE}}

# Apply infrastructure changes
infra-apply PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask infra apply --profile {{PROFILE}}

# Destroy all infrastructure (prompt confirmation)
infra-destroy PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask infra destroy --profile {{PROFILE}}

# Show OpenTofu outputs (API endpoint URL, etc.)
infra-output PROFILE="default":
    cargo xtask infra output --profile {{PROFILE}}

# ── Deployment ───────────────────────────────────────────────────────────────

# Build Docker image locally
build-image:
    cargo xtask deploy docker

# Push a locally-built image to Amazon ECR.
# IMAGE must be the full ECR URI: <account>.dkr.ecr.<region>.amazonaws.com/<repo>:<tag>
# Example: just push-image default 123456789012.dkr.ecr.us-east-1.amazonaws.com/deploy-baba-ui:latest
push-image PROFILE="default" IMAGE="deploy-baba-ui:latest":
    just aws-check {{PROFILE}} && cargo xtask deploy push --image {{IMAGE}} --profile {{PROFILE}}

# Full deploy: quality gate → zip build → Lambda update (zip-based Lambda, ADR-003)
deploy PROFILE="default":
    just quality && just lambda-deploy {{PROFILE}}

# Deploy without quality gate (fast path)
deploy-fast PROFILE="default":
    just lambda-deploy {{PROFILE}}

# Dry run: build + validate, no push
deploy-dry PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy docker

# Wait for Lambda to settle after a code update (step 2 of full pipeline)
lambda-wait PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy wait --profile {{PROFILE}}

# SPA-only deploy: build → S3 sync → sync-spa invoke → /health (steps 3–6)
# Requires: SPA_BUCKET, UI_FN_NAME, FN_URL env vars (or set via infra outputs)
spa-deploy PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy spa --profile {{PROFILE}} --sha "$(git rev-parse HEAD)"

# Full pipeline: quality → Lambda → wait → SPA build → S3 sync → sync-spa → /health
# Pass TAG=1 to also create a dev-vX.Y.Z git tag (mirrors deploy-dev.yml)
deploy-full PROFILE="default" TAG="":
    just quality
    just lambda-deploy {{PROFILE}}
    just lambda-wait {{PROFILE}}
    just spa-deploy {{PROFILE}}
    {{ if TAG != "" { "just release-tag dev push" } else { "echo 'Skipping dev tag — pass TAG=1 to enable'" } }}

# ── Database (SQLite + S3) ───────────────────────────────────────────────────

# Back up SQLite from EFS to S3
db-backup PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database backup --profile {{PROFILE}}

# Restore latest SQLite backup from S3 to EFS
db-restore PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database restore --profile {{PROFILE}}

# Restore a specific backup version from S3
db-restore-version VERSION PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database restore --version {{VERSION}} --profile {{PROFILE}}

# List available S3 backup versions
db-list-backups PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database list-backups --profile {{PROFILE}}

# ── Resume Generation ────────────────────────────────────────────────────────

# Generate resume files (2 formats × DOCX + PDF) from SQLite — outputs to target/resume/
resume-generate DB="deploy-baba.db":
    cargo xtask resume generate --db-path {{DB}}

# Upload generated resume files to S3 assets bucket
resume-upload PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask resume upload --profile {{PROFILE}}

# Full pipeline: generate + upload
resume PROFILE="default" DB="deploy-baba.db":
    just resume-generate {{DB}} && just resume-upload {{PROFILE}}

# ── RAG ──────────────────────────────────────────────────────────────────────

# Index all corpora (Rust, HCL, plans) into the RAG FTS index
rag-index DB="deploy-baba.db":
    cargo xtask rag ingest --db-path {{DB}}

# Index all corpora + .claude/ agent cache (local dev only)
rag-index-full DB="deploy-baba.db":
    cargo xtask rag ingest --db-path {{DB}} --include-cache

# Query the RAG index and print ranked chunks
rag-query QUERY DB="deploy-baba.db":
    cargo xtask rag query --db-path {{DB}} "{{QUERY}}"

# Retrieve chunks + generate a grounded answer via Claude (requires ANTHROPIC_API_KEY)
ask QUERY DB="deploy-baba.db":
    cargo xtask rag ask --db-path {{DB}} "{{QUERY}}"

# ── crates.io ────────────────────────────────────────────────────────────────

# Dry-run publish for all library crates
publish-dry:
    cargo xtask publish dry-run

# Publish all library crates in dependency order
publish:
    just quality && cargo xtask publish release

# ── Secrets Manager ──────────────────────────────────────────────────────────

# Write a secret to AWS Secrets Manager (e.g. just secret-put pow-secret $(openssl rand -hex 32))
secret-put NAME VALUE PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask secret put {{NAME}} {{VALUE}} --profile {{PROFILE}}

# Read a secret value from AWS Secrets Manager
secret-get NAME PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask secret get {{NAME}} --profile {{PROFILE}}

# List all managed secrets under the deploy-baba/prod/ prefix
secret-list PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask secret list --profile {{PROFILE}}

# Run llm-anthropic live integration tests using the key stored in Secrets Manager.
# Requires AWS auth and the anthropic-api-key secret to be provisioned.
test-llm PROFILE="default":
    just aws-check {{PROFILE}} && \
    ANTHROPIC_API_KEY=$(cargo xtask secret get anthropic-api-key --profile {{PROFILE}} | tail -1) \
    cargo test -p llm-anthropic -- --ignored --nocapture

# ── Agent Cache ──────────────────────────────────────────────────────────────

# Show cache age and whether it's stale vs current HEAD
cache-status:
    cargo xtask cache status

# Re-scan the codebase and rewrite .agent-cache/index.json
cache-refresh:
    cargo xtask cache refresh

# Delete the cache to force a full re-scan next session
cache-clear:
    cargo xtask cache clear

# ── Release Management ────────────────────────────────────────────────────────

# Dry-run: print the next version derived from conventional commits since the last dev-v* tag
release-next:
    cargo xtask release next

# Create a dev-vX.Y.Z annotated tag at HEAD (CI runs this automatically after a successful deploy)
release-tag KIND="dev" PUSH="":
    cargo xtask release tag --kind {{KIND}} {{ if PUSH != "" { "--push" } else { "" } }}

# Promote the latest dev-v* tag to vX.Y.Z, triggering deploy-prod.yml (with manual approval)
release-promote PUSH="":
    cargo xtask release promote {{ if PUSH != "" { "--push" } else { "" } }}
