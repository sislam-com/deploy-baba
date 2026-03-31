# deploy-baba — Zero-cost Rust abstractions for deployment automation
# All developer commands go through this justfile. Never call `cargo xtask` directly.

set dotenv-load := false

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

# Verify the live deployment (curl apex + www health checks)
infra-verify DOMAIN="sislam.com":
    @echo "=== Verifying {{DOMAIN}} ==="
    curl -sI https://{{DOMAIN}} | head -1
    curl -sI https://www.{{DOMAIN}} | head -1
    curl -s https://{{DOMAIN}}/health
    @echo ""

# ── UI / Portfolio Site ──────────────────────────────────────────────────────

# Run the portfolio site locally (Axum TCP server + cargo-watch hot reload)
ui:
    cargo watch -x 'run --package deploy-baba-ui'

# Run the portfolio site once (no hot reload)
ui-run:
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

# ── Infrastructure (OpenTofu) ────────────────────────────────────────────────

# Bootstrap: create S3 state bucket + write sentinel SSM param (first run only)
infra-bootstrap PROFILE="default" REGION="us-east-1":
    cargo xtask infra bootstrap --profile {{PROFILE}} --region {{REGION}}

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

# Build + push image to ECR Public
push-image PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy push --profile {{PROFILE}}

# Full deploy: quality gate → build → push → Lambda update
deploy PROFILE="default":
    just quality && just push-image {{PROFILE}} && cargo xtask deploy lambda --profile {{PROFILE}}

# Deploy without quality gate (fast path)
deploy-fast PROFILE="default":
    just push-image {{PROFILE}} && cargo xtask deploy lambda --profile {{PROFILE}}

# Dry run: build + validate, no push
deploy-dry PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy docker

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

# ── crates.io ────────────────────────────────────────────────────────────────

# Dry-run publish for all library crates
publish-dry:
    cargo xtask publish dry-run

# Publish all library crates in dependency order
publish:
    just quality && cargo xtask publish release
