# deploy-baba — Zero-cost Rust abstractions for deployment automation
# All developer commands go through this justfile. Never call `cargo xtask` directly.

set dotenv-load := false

# Set the shell to bash to support 'eval' and string manipulation

set shell := ["bash", "-c"]

# Default AWS profile — pinned to stack.toml `aws.profile`.
# Recipes with a `PROFILE` parameter shadow this; argless recipes (sso-login, ui, dev-stack) use it directly.

PROFILE := "deploy-baba"

# ── Meta ──────────────────────────────────────────────────────────────────────

# List all available commands
default:
    @just --list

# ── Inner Loop (daily dev) ────────────────────────────────────────────────────

# Format all code with configured formatters (Rust + Python + OpenTofu HCL)
fmt:
    cargo xtask build format
    just agent-fmt
    tofu fmt -recursive infra/

# Run clippy (warnings = errors) + verify HCL formatting
lint:
    cargo xtask build lint
    tofu fmt -check -recursive infra/

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
    cargo xtask test crate {{ CRATE }}

# Generate coverage report (opens in browser)
coverage:
    cargo xtask coverage report --open

# fmt + lint + test (the standard inner loop)
dev:
    @just dev-doctor
    @just dev-stack

# Full quality gate (fmt + lint + test + coverage floors + audit + HCL fmt check + agent)
quality:
    just web-types-offline && cargo xtask quality all && just web-coverage && just agent-lint && just agent-test && just agent-build && just mcp-build && just mcp-smoke && just mcp-rag-smoke && just mcp-cloud-build && just rag-index && tofu fmt -check -recursive infra/

# Build everything: all Rust Lambda zips + SPA + agent package + MCP gateway bundle
build: lambda-build-all web-build agent-build mcp-cloud-build

# Build all assets: Lambda zips + SPA + agent + MCP gateway + resume + RAG index
build-all: build resume-generate rag-index

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
    cargo run -p example_{{ NAME }}

# ── Lambda Builds (per-service) ──────────────────────────────────────────────
# Each Rust service compiles to an aarch64 Lambda zip via cargo-lambda.
# Convention: <service>-build produces infra/build/<service>-lambda.zip,
#             <service>-deploy builds + updates the live Lambda function.

# UI (main VPC Lambda — serves /api/*, /auth/*, /health, /docs)
lambda-build:
    PATH="$HOME/.cargo/bin:$PATH" cargo lambda build --release --package deploy-baba-ui --target aarch64-unknown-linux-gnu

lambda-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}

# Email (non-VPC, SES sends)
email-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-email --package email-lambda

# LLM Proxy (non-VPC, reaches api.anthropic.com)
llm-proxy-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-llm-proxy --package llm-proxy

# Auth (non-VPC, reaches Cognito IDP)
auth-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-auth --package auth-lambda

# Portfolio (VPC, read-only data — jobs, competencies, about, social-links, resume, challenges)
portfolio-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-portfolio --package portfolio-lambda

# Admin (VPC, dashboard CRUD — migration owner)
admin-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-admin --package admin-lambda

# Contact (non-VPC, PoW validation + email Lambda delegation)
contact-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-contact --package contact-lambda

# RAG (VPC, FTS5 retrieval + grounded generation)
rag-deploy ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-rag --package rag-lambda

# ── Consolidated Lambda Builds ───────────────────────────────────────────────

# Build all Rust Lambda zips (excludes Python agent and MCP gateway which have extra bundling steps)
lambda-build-all:
    #!/usr/bin/env bash
    set -euo pipefail
    for pkg in deploy-baba-ui email-lambda llm-proxy auth-lambda portfolio-lambda admin-lambda contact-lambda rag-lambda; do
        echo "Building ${pkg}..."
        PATH="$HOME/.cargo/bin:$PATH" cargo lambda build --release --package "$pkg" --target aarch64-unknown-linux-gnu
    done

# Deploy all Rust Lambdas (ENV: prod or dev)
lambda-deploy-all ENV="prod":
    just lambda-deploy {{ ENV }}
    just email-deploy {{ ENV }}
    just llm-proxy-deploy {{ ENV }}
    just auth-deploy {{ ENV }}
    just portfolio-deploy {{ ENV }}
    just admin-deploy {{ ENV }}
    just contact-deploy {{ ENV }}
    just rag-deploy {{ ENV }}

# Build the read-only context bundle consumed by the private cloud MCP gateway
mcp-context-build:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf build/mcp-context build/mcp-gateway
    mkdir -p build/mcp-context/.agent-cache build/mcp-context/plans/modules build/mcp-context/plans/adr build/mcp-context/services/ui/migrations build/mcp-gateway
    cp .agent-cache/index.json build/mcp-context/.agent-cache/index.json
    cp plans/INDEX.md build/mcp-context/plans/INDEX.md
    cp plans/modules/*.md build/mcp-context/plans/modules/
    cp plans/adr/*.md build/mcp-context/plans/adr/
    cp services/ui/migrations/*.sql build/mcp-context/services/ui/migrations/
    cp justfile build/mcp-context/justfile
    cp stack.example.toml build/mcp-context/stack.example.toml
    python3 - <<'PY'
    from pathlib import Path
    text = Path(".mcp-rs.toml").read_text()
    text = text.replace('workspace_root = "."', 'workspace_root = "/var/task/mcp-context"')
    text = text.replace('log_path = "mcp-audit.jsonl"', 'log_path = "/tmp/mcp-gateway-audit.jsonl"')
    Path("build/mcp-gateway/mcp-rs.toml").write_text(text)
    PY

# Build the private Cognito-protected MCP gateway Lambda zip
mcp-cloud-build: mcp-context-build
    PATH="$HOME/.cargo/bin:$PATH" cargo lambda build --release --package mcp-gateway --target aarch64-unknown-linux-gnu
    cp target/lambda/mcp-gateway/bootstrap build/mcp-gateway/bootstrap
    chmod +x build/mcp-gateway/bootstrap
    mkdir -p infra/build
    rm -f infra/build/mcp-gateway-lambda.zip
    cd build/mcp-gateway && zip -qr ../../infra/build/mcp-gateway-lambda.zip bootstrap mcp-rs.toml ../mcp-context

# Build + upload the private MCP gateway Lambda
mcp-cloud-deploy ENV="prod":
    just aws-check {{ PROFILE }} && just mcp-cloud-build && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-mcp-gateway \
        --zip-path infra/build/mcp-gateway-lambda.zip

# Smoke the deployed private MCP gateway. Requires MCP_BEARER_TOKEN with a valid Cognito ID token.
mcp-cloud-smoke PROFILE="default" BASE_URL="https://sislam.com":
    @status=$(curl -s -o /tmp/mcp-unauth.out -w "%{http_code}" -X POST {{ BASE_URL }}/mcp -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'); test "$status" = "401"
    @test -n "$MCP_BEARER_TOKEN" || (echo "Set MCP_BEARER_TOKEN to a valid Cognito ID token" >&2; exit 1)
    @curl -fsS -H "Authorization: Bearer $MCP_BEARER_TOKEN" -H "Content-Type: application/json" -X POST {{ BASE_URL }}/mcp -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | jq .
    @curl -fsS -H "Authorization: Bearer $MCP_BEARER_TOKEN" -H "Content-Type: application/json" -X POST {{ BASE_URL }}/mcp -d '{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}' | jq .
    @curl -fsS -H "Authorization: Bearer $MCP_BEARER_TOKEN" -H "Content-Type: application/json" -X POST {{ BASE_URL }}/mcp -d '{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"project://plans"}}' | jq .

# ── Agent (Python/LangGraph) ──────────────────────────────────────────────────

# Run the agent service locally on :3003 with auto-reload
agent-dev:
    cd services/agent && UI_BASE_URL=http://localhost:3001 PYTHONPATH=src uv run uvicorn handler:app --host 0.0.0.0 --port 3003 --reload

# Run agent tests
agent-test:
    cd services/agent && PYTHONPATH=src uv run pytest tests/ -v

# Lint + typecheck the agent service
agent-lint:
    cd services/agent && uv run ruff check src/ tests/ && uv run ruff format --check src/ tests/ && PYTHONPATH=src uv run mypy src/

# Format the agent service
agent-fmt:
    cd services/agent && uv run ruff format src/ tests/

# Build the agent Lambda deployment package
agent-build:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf build/agent-lambda
    mkdir -p build/agent-lambda infra/build
    cd services/agent
    uv export --frozen --no-dev --no-emit-project > /tmp/agent-requirements.txt
    uv pip install --python 3.13 --target ../../build/agent-lambda -r /tmp/agent-requirements.txt --only-binary :all: --quiet
    cp -r src/* ../../build/agent-lambda/
    cd ../../build/agent-lambda
    zip -qr ../../infra/build/agent-lambda.zip .

# Build agent Lambda zip + update the deployed function
agent-deploy ENV="prod":
    just aws-check {{ PROFILE }} && just agent-build && cargo xtask deploy lambda \
        --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}-agent \
        --zip-path infra/build/agent-lambda.zip

# ── PDF Lambda (Docker-based WeasyPrint service) ─────────────────────────────

# Build and push PDF Lambda Docker image to ECR
pdf-build ENV="prod":
    #!/usr/bin/env bash
    set -euo pipefail
    just aws-check {{ PROFILE }}
    REGION=$(aws configure get region --profile {{ PROFILE }} 2>/dev/null || echo "us-east-1")
    AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text --profile {{ PROFILE }})
    ECR_REPO="${AWS_ACCOUNT}.dkr.ecr.${REGION}.amazonaws.com/deploy-baba-{{ ENV }}-pdf"
    echo "Building PDF Lambda image for ${ECR_REPO}..."
    aws ecr get-login-password --region ${REGION} --profile {{ PROFILE }} | \
        docker login --username AWS --password-stdin ${AWS_ACCOUNT}.dkr.ecr.${REGION}.amazonaws.com
    docker buildx build --platform linux/amd64 --provenance=false --sbom=false --tag deploy-baba-pdf:latest services/pdf/
    docker tag deploy-baba-pdf:latest "${ECR_REPO}:latest"
    docker push "${ECR_REPO}:latest"
    echo "PDF Lambda image pushed: ${ECR_REPO}:latest"

# Build and deploy PDF Lambda (requires infra to be applied first for ECR repo)
pdf-deploy ENV="prod":
    #!/usr/bin/env bash
    set -euo pipefail
    just pdf-build {{ ENV }}
    echo "Updating PDF Lambda function with new image..."
    just aws-check {{ PROFILE }}
    REGION=$(aws configure get region --profile {{ PROFILE }} 2>/dev/null || echo "us-east-1")
    AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text --profile {{ PROFILE }})
    aws lambda update-function-code \
        --function-name "deploy-baba-{{ ENV }}-pdf" \
        --image-uri "${AWS_ACCOUNT}.dkr.ecr.${REGION}.amazonaws.com/deploy-baba-{{ ENV }}-pdf:latest" \
        --profile {{ PROFILE }}
    echo "PDF Lambda deployed successfully"

# Verify the live deployment (curl apex + www health checks)
infra-verify DOMAIN="sislam.com":
    @echo "=== Verifying {{ DOMAIN }} ==="
    curl -sI https://{{ DOMAIN }} | head -1
    curl -sI https://www.{{ DOMAIN }} | head -1
    curl -s https://{{ DOMAIN }}/health
    @echo ""

# ── UI / Portfolio Site ──────────────────────────────────────────────────────
# Run the Rust API server locally on :3001 with cargo watch for auto-reload.
# The API handles /api/*, /auth/*, /health, /docs — not the SPA.

# For full-stack dev with the Vite dev server, use `just dev-stack` instead.
ui ENV="dev":
    #!/usr/bin/env bash
    set -euo pipefail
    eval "$(just dev-env {{ ENV }})"
    env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN \
        AWS_PROFILE={{ PROFILE }} cargo watch -x 'run --package deploy-baba-ui'

# Run the Rust API server once (no hot reload) on :3001.
ui-run:
    #!/usr/bin/env bash
    set -euo pipefail
    eval "$(just dev-env)"
    env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN AWS_PROFILE={{ PROFILE }} \
        cargo run --package deploy-baba-ui

# Build the UI binary only (fast check)
ui-build:
    cargo build --package deploy-baba-ui

# Tail CloudWatch logs for the deployed Lambda
ui-logs PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask aws logs --function deploy-baba-ui --profile {{ PROFILE }}

# Open the live portfolio URL (reads from OpenTofu outputs)
ui-open:
    open $(tofu -chdir=infra output -raw function_url)

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
    cargo xtask aws validate --profile {{ PROFILE }}

# Print AWS setup instructions
aws-setup:
    @cat docs/aws-setup.md

# Print current caller identity
aws-whoami PROFILE="default":
    aws sts get-caller-identity --profile {{ PROFILE }}

# Log in to AWS SSO. Populates ~/.aws/sso/cache — run once per workday before dev-stack/infra-plan/lambda-deploy.
sso-login:
    @V=$(aws --version 2>&1); if ! echo "$V" | grep -q "aws-cli/2"; then echo "ERROR: AWS CLI v2 required for 'aws sso login'. Found: $V" >&2; echo "Install: brew install awscli" >&2; exit 1; fi
    aws sso login --profile {{ PROFILE }}

# ── Developer Environment ─────────────────────────────────────────────────────
# Print `export X=Y` lines for all env vars the local Rust binary needs.
# Fetches Cognito config from SSM (/deploy-baba/<ENV>/cognito-*) and JWKS from the
# public Cognito endpoint. Consumed via `eval "$(just dev-env)"` in `just ui`.

# Requires a valid SSO session — run `just sso-login` first.
dev-env ENV="prod":
    #!/usr/bin/env bash
    set -euo pipefail
    AWS="env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN AWS_PROFILE={{ PROFILE }} aws"
    if ! $AWS sts get-caller-identity --query Account --output text >/dev/null 2>&1; then
        echo "ERROR: AWS SSO session expired or not configured. Run: just sso-login" >&2
        exit 1
    fi
    pool_id=$($AWS ssm get-parameter --name /deploy-baba/{{ ENV }}/cognito-pool-id    --query Parameter.Value --output text)
    client_id=$($AWS ssm get-parameter --name /deploy-baba/{{ ENV }}/cognito-client-id --query Parameter.Value --output text)
    domain=$($AWS    ssm get-parameter --name /deploy-baba/{{ ENV }}/cognito-domain    --query Parameter.Value --output text)
    jwks=$(curl -fsSL "https://cognito-idp.us-east-1.amazonaws.com/${pool_id}/.well-known/jwks.json")
    echo "export AWS_PROFILE={{ PROFILE }}"
    echo "export ANTHROPIC_API_KEY_ARN=root-anthropic-access-key"
    echo "export LINKEDIN_SECRET_ARN=deploy-baba/prod/linkedin-api-key"
    echo "export RAG_PUBLIC_ENABLED=1"
    echo "export COGNITO_POOL_ID=${pool_id}"
    echo "export COGNITO_CLIENT_ID=${client_id}"
    echo "export COGNITO_DOMAIN=${domain}"
    echo "export COGNITO_REGION=us-east-1"
    echo "export APP_DOMAIN=http://localhost:3000"
    printf 'export COGNITO_JWKS=%q\n' "${jwks}"

    # AWS region for boto3 clients in agent service
    echo "export AWS_REGION=us-east-1"
    echo "export AWS_REGION_OVERRIDE=us-east-1"

    # Agent service infrastructure refs (UI Lambda for resume data, PDF Lambda, S3 bucket)
    echo "export UI_LAMBDA_NAME=deploy-baba-prod"
    echo "export PDF_LAMBDA_NAME=deploy-baba-prod-pdf"
    echo "export ARTIFACTS_BUCKET=deploy-baba-assets-062513063428"

    # Fetch actual secret values for Python agent service (checks env var first)
    anthropic_key=$($AWS secretsmanager get-secret-value --secret-id deploy-baba/prod/anthropic-api-key --query SecretString --output text 2>/dev/null || echo "")
    if [ -n "$anthropic_key" ] && [ "$anthropic_key" != "placeholder-set-via-just-secret-put" ]; then
        printf 'export ANTHROPIC_API_KEY=%q\n' "$anthropic_key"
    fi

    linkedin_secret=$($AWS secretsmanager get-secret-value --secret-id deploy-baba/prod/linkedin-api-key --query SecretString --output text 2>/dev/null || echo "")
    if [ -n "$linkedin_secret" ] && [ "$linkedin_secret" != "placeholder-set-via-just-secret-put" ]; then
        client_id=$(printf '%s' "$linkedin_secret" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('client_id',''))")
        client_secret=$(printf '%s' "$linkedin_secret" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('client_secret',''))")
        [ -n "$client_id" ] && printf 'export LINKEDIN_CLIENT_ID=%q\n' "$client_id"
        [ -n "$client_secret" ] && printf 'export LINKEDIN_CLIENT_SECRET=%q\n' "$client_secret"
    fi

# Verify all prerequisites (rustup, cargo-lambda, node≥20, pnpm, tofu, AWS SSO, cache)
dev-doctor:
    bash scripts/dev-doctor.sh

# ── Web / SPA (Vite + React) ──────────────────────────────────────────────────

# Start Vite dev server on :3000 with /api proxy to :3001
web:
    pnpm --dir web dev

# Build SPA to web/dist/
web-build:
    pnpm --dir web run build

# Run Vitest unit tests
web-test:
    pnpm --dir web run test

# Run Vitest coverage report for web/
web-coverage:
    pnpm --dir web run coverage

# TypeScript type check (no emit)
web-typecheck:
    pnpm --dir web run typecheck

# ESLint
web-lint:
    pnpm --dir web run lint

# Regenerate src/api/types.gen.ts from the running local server (requires just ui on :3001).

# Prefer web-types-offline for CI and offline use.
web-types:
    pnpm --dir web run types

# Emit public OpenAPI spec to web/openapi.json (offline; no server required)
api-spec:
    cargo run -q -p api-openapi --bin emit-spec > web/openapi.json

# Derive web/src/api/types.gen.ts from the offline-emitted spec
web-types-offline: api-spec
    pnpm --dir web exec openapi-typescript openapi.json -o src/api/types.gen.ts

# Download latest SQLite backup from S3 to deploy-baba.db (skips if fresh within 1 hour)
db-sync:
    #!/usr/bin/env bash
    set -euo pipefail
    DB="deploy-baba.db"
    if [ -f "$DB" ]; then
        # Warn if there are unsynced dashboard edits that would be lost
        unsynced=$(sqlite3 "$DB" "SELECT COUNT(*) FROM _change_log WHERE synced = 0;" 2>/dev/null || echo "0")
        if [ "$unsynced" -gt 0 ]; then
            echo "⚠️  $DB has $unsynced unsynced dashboard edit(s). Run 'just db-changes' to review."
            echo "    To discard: just db-reset && just db-sync"
            echo "    To capture: run /sync-dashboard-data first, then just db-changes-ack"
            exit 1
        fi
        # Skip download if the file is fresh AND passes integrity check
        integrity=$(sqlite3 "$DB" "PRAGMA quick_check;" 2>&1)
        if [ "$integrity" = "ok" ]; then
            age=$(( $(date +%s) - $(stat -f %m "$DB") ))
            if [ "$age" -lt 3600 ]; then
                echo "⏩ $DB is ${age}s old (< 1h) and healthy — skipping S3 sync"
                exit 0
            fi
        else
            echo "⚠️  $DB failed integrity check — will re-download from S3"
            rm -f "$DB" "${DB}-wal" "${DB}-shm"
        fi
    fi
    echo "⬇️  Syncing latest S3 backup → $DB"
    just db-restore {{ PROFILE }}

# Start the Rust API server (:3001), Vite dev server (:3000), and agent service (:3003) in parallel.
# Port assignments: 3000=Vite SPA, 3001=Rust API, 3002=Auth Lambda, 3003=Python Agent

# Pre-cleans stale processes on all ports and guarantees cleanup on exit.
dev-stack:
    #!/usr/bin/env bash
    set -euo pipefail

    # Pre-empt any stale processes on our ports
    for port in 3000 3001 3002 3003; do
        pid=$(lsof -ti tcp:$port 2>/dev/null || true)
        if [ -n "$pid" ]; then
            echo "Port $port occupied by PID $pid — stopping..."
            kill "$pid" 2>/dev/null || true
            sleep 0.5
            kill -9 "$pid" 2>/dev/null || true
        fi
    done

    # If a local DB exists, validate it; if not, try S3 sync then fall back to fresh-from-migrations
    if [ -f deploy-baba.db ]; then
        integrity=$(sqlite3 deploy-baba.db "PRAGMA quick_check;" 2>&1)
        if [ "$integrity" != "ok" ]; then
            echo "⚠️  deploy-baba.db is corrupted — removing and will recreate"
            rm -f deploy-baba.db deploy-baba.db-wal deploy-baba.db-shm
        fi
    fi

    if [ ! -f deploy-baba.db ]; then
        if just db-sync 2>/dev/null; then
            # Verify the downloaded backup is healthy
            if [ -f deploy-baba.db ]; then
                integrity=$(sqlite3 deploy-baba.db "PRAGMA quick_check;" 2>&1)
                if [ "$integrity" != "ok" ]; then
                    echo "⚠️  S3 backup is corrupt — creating fresh DB from migrations instead"
                    rm -f deploy-baba.db deploy-baba.db-wal deploy-baba.db-shm
                fi
            fi
        else
            echo "⚠️  S3 sync unavailable — creating fresh DB from migrations"
        fi
    fi

    # Load AWS env (SSO required) — fall back to offline dev mode if unavailable
    if DEV_ENV_OUTPUT=$(just dev-env 2>&1); then
        eval "$DEV_ENV_OUTPUT"
    else
        echo "WARNING: AWS SSO not available — running in offline dev mode (auth bypass)"
        echo "  Run 'just sso-login' to enable full functionality."
        export DEV_MODE=1
        export COGNITO_POOL_ID="offline"
        export COGNITO_CLIENT_ID="offline"
        export COGNITO_DOMAIN="offline"
        export COGNITO_REGION="us-east-1"
        export COGNITO_JWKS='{}'
        export RAG_PUBLIC_ENABLED=1
    fi

    # Start API server directly (use just ui for cargo watch + auto-reload)
    echo "Starting API server on :3001..."
    env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN \
        AWS_PROFILE={{ PROFILE }} cargo run --package deploy-baba-ui &
    API_PID=$!

    # Start auth service (Cognito IDP proxy on :3002)
    echo "Starting auth service on :3002..."
    env -u AWS_ACCESS_KEY_ID -u AWS_SECRET_ACCESS_KEY -u AWS_SESSION_TOKEN \
        AWS_PROFILE={{ PROFILE }} cargo run --package auth-lambda &
    AUTH_PID=$!

    # Wait for API to be ready before starting Vite (avoids proxy errors)
    for i in $(seq 1 60); do
        if curl -sf http://localhost:3001/health >/dev/null 2>&1; then
            echo "API ready on :3001"
            break
        fi
        if ! kill -0 $API_PID 2>/dev/null; then
            echo "API server exited unexpectedly" >&2
            exit 1
        fi
        sleep 1
    done

    # Start agent service (Python/LangGraph) if uv is available
    AGENT_PID=""
    if command -v uv &>/dev/null; then
        echo "Starting agent service on :3003..."
        (cd services/agent && UI_BASE_URL=http://localhost:3001 PYTHONPATH=src uv run uvicorn handler:app --host 0.0.0.0 --port 3003 --reload) &
        AGENT_PID=$!
    else
        echo "⚠️  uv not found — skipping agent service on :3003 (install: curl -LsSf https://astral.sh/uv/install.sh | sh)"
    fi

    # Start Vite dev server
    echo "Starting Vite dev server on :3000..."
    pnpm --dir web dev &
    WEB_PID=$!

    CLEANED_UP=0
    cleanup() {
        if [ "$CLEANED_UP" -eq 1 ]; then return; fi
        CLEANED_UP=1
        echo ""
        echo "Shutting down dev servers..."
        kill $WEB_PID 2>/dev/null || true
        kill $API_PID 2>/dev/null || true
        kill $AUTH_PID 2>/dev/null || true
        [ -n "$AGENT_PID" ] && kill $AGENT_PID 2>/dev/null || true
        sleep 1
        kill -9 $WEB_PID 2>/dev/null || true
        kill -9 $API_PID 2>/dev/null || true
        kill -9 $AUTH_PID 2>/dev/null || true
        [ -n "$AGENT_PID" ] && kill -9 $AGENT_PID 2>/dev/null || true
        for port in 3000 3001 3002 3003; do
            pid=$(lsof -ti tcp:$port 2>/dev/null || true)
            [ -n "$pid" ] && kill -9 "$pid" 2>/dev/null || true
        done
        echo "Ports released."
    }
    trap cleanup SIGINT SIGTERM EXIT

    wait

# ── Infrastructure (OpenTofu) ────────────────────────────────────────────────

# Bootstrap: create S3 state bucket + DynamoDB lock table (idempotent, run once per account)
infra-bootstrap PROFILE="default" REGION="us-east-1":
    bash scripts/bootstrap-tfstate.sh

# Preview infrastructure changes (WORKSPACE: default=prod, dev=dev-named resources)
infra-plan WORKSPACE="default":
    just aws-check {{ PROFILE }} && cargo xtask infra plan --workspace {{ WORKSPACE }} --aws-profile {{ PROFILE }}

# Apply infrastructure changes (WORKSPACE: default=prod, dev=dev-named resources)
infra-apply WORKSPACE="default":
    just aws-check {{ PROFILE }} && cargo xtask infra apply --workspace {{ WORKSPACE }} --aws-profile {{ PROFILE }}

# Destroy all infrastructure (prompt confirmation)
infra-destroy WORKSPACE="default":
    just aws-check {{ PROFILE }} && cargo xtask infra destroy --workspace {{ WORKSPACE }} --aws-profile {{ PROFILE }}

# Show OpenTofu outputs (API endpoint URL, etc.)
infra-output WORKSPACE="default":
    cargo xtask infra output --workspace {{ WORKSPACE }} --aws-profile {{ PROFILE }}

# ── Deployment ───────────────────────────────────────────────────────────────

# Build Docker image locally
build-image:
    cargo xtask deploy docker

# Push a locally-built image to Amazon ECR.
# IMAGE must be the full ECR URI: <account>.dkr.ecr.<region>.amazonaws.com/<repo>:<tag>

# Example: just push-image default 123456789012.dkr.ecr.us-east-1.amazonaws.com/deploy-baba-ui:latest
push-image PROFILE="default" IMAGE="deploy-baba-ui:latest":
    just aws-check {{ PROFILE }} && cargo xtask deploy push --image {{ IMAGE }} --profile {{ PROFILE }}

# Full deploy: quality gate → zip build → Lambda update (zip-based Lambda, ADR-003)
deploy ENV="prod":
    just quality && just lambda-deploy {{ ENV }}

# Deploy without quality gate (fast path)
deploy-fast ENV="prod":
    just lambda-deploy {{ ENV }}

# Dry run: build + validate, no push
deploy-dry:
    just aws-check {{ PROFILE }} && cargo xtask deploy docker

# Wait for Lambda to settle after a code update (step 2 of full pipeline)
lambda-wait ENV="prod":
    just aws-check {{ PROFILE }} && cargo xtask deploy wait --profile {{ PROFILE }} --function deploy-baba-{{ ENV }}

# SPA-only deploy: build → S3 sync → sync-spa invoke → /health (steps 3–6)

# Requires: SPA_BUCKET, UI_FN_NAME, FN_URL env vars (or set via infra outputs)
spa-deploy:
    just aws-check {{ PROFILE }} && cargo xtask deploy spa --profile {{ PROFILE }} --sha "$(git rev-parse HEAD)"

# Full pipeline: quality → Lambda → wait → SPA build → S3 sync → sync-spa → /health

# Pass TAG=1 to also create a dev-vX.Y.Z git tag (mirrors deploy-dev.yml)
deploy-full ENV="prod" TAG="":
    just quality
    just lambda-deploy {{ ENV }}
    just lambda-wait {{ ENV }}
    just spa-deploy
    {{ if TAG != "" { "just release-tag dev push" } else { "echo 'Skipping dev tag — pass TAG=1 to enable'" } }}

# Deploy all services + SPA + resume + RAG (full production push)
deploy-all ENV="prod":
    just quality
    just lambda-deploy-all {{ ENV }}
    just agent-deploy {{ ENV }}
    just mcp-cloud-deploy {{ ENV }}
    just lambda-wait {{ ENV }}
    just spa-deploy
    just resume-upload
    just rag-sync {{ ENV }}

# ── Database (SQLite + S3) ───────────────────────────────────────────────────

# Check local SQLite integrity
db-check DB="deploy-baba.db":
    @sqlite3 {{ DB }} "PRAGMA integrity_check;"

# Show unsynced dashboard edits (change-tracking log)
db-changes DB="deploy-baba.db":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f "{{ DB }}" ]; then
        echo "No database file found at {{ DB }}"
        exit 0
    fi
    count=$(sqlite3 "{{ DB }}" "SELECT COUNT(*) FROM _change_log WHERE synced = 0;" 2>/dev/null || echo "0")
    if [ "$count" = "0" ]; then
        echo "No unsynced dashboard changes."
    else
        echo "$count unsynced change(s):"
        sqlite3 -header -column "{{ DB }}" \
            "SELECT table_name, natural_key, operation, changed_at FROM _change_log WHERE synced = 0 ORDER BY changed_at;"
    fi

# Mark all change-log entries as synced (run after generating a sync migration)
db-changes-ack DB="deploy-baba.db":
    sqlite3 "{{ DB }}" "UPDATE _change_log SET synced = 1 WHERE synced = 0;"
    @echo "All change-log entries marked as synced."

# Delete local SQLite and let next startup recreate from migrations
db-reset:
    rm -f deploy-baba.db deploy-baba.db-wal deploy-baba.db-shm
    @echo "Local DB deleted. Next 'just dev-stack' will create a fresh one from migrations."

# Back up SQLite from EFS to S3
db-backup PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask database backup --profile {{ PROFILE }}

# Restore latest SQLite backup from S3 to EFS
db-restore PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask database restore --profile {{ PROFILE }}

# Restore a specific backup version from S3
db-restore-version VERSION PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask database restore --version {{ VERSION }} --profile {{ PROFILE }}

# List available S3 backup versions
db-list-backups PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask database list-backups --profile {{ PROFILE }}

# ── Resume Generation ────────────────────────────────────────────────────────

# Generate resume files (2 formats × DOCX + PDF) from SQLite — outputs to target/resume/
resume-generate DB="deploy-baba.db":
    cargo xtask resume generate --db-path {{ DB }}

# Upload generated resume files to S3 assets bucket
resume-upload PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask resume upload --profile {{ PROFILE }}

# Full pipeline: generate + upload
resume PROFILE="default" DB="deploy-baba.db":
    just resume-generate {{ DB }} && just resume-upload {{ PROFILE }}

# ── RAG ──────────────────────────────────────────────────────────────────────

# Index all corpora (Rust, HCL, plans) into the RAG FTS index
rag-index DB="deploy-baba.db":
    cargo xtask rag ingest --db-path {{ DB }}

# Index all corpora + .claude/ agent cache (local dev only)
rag-index-full DB="deploy-baba.db":
    cargo xtask rag ingest --db-path {{ DB }} --include-cache

# Query the RAG index and print ranked chunks
rag-query QUERY DB="deploy-baba.db":
    cargo xtask rag query --db-path {{ DB }} "{{ QUERY }}"

# Retrieve chunks + generate a grounded answer via Claude (requires ANTHROPIC_API_KEY)
ask QUERY DB="deploy-baba.db":
    cargo xtask rag ask --db-path {{ DB }} "{{ QUERY }}"

# Run RAG evaluation suite (retrieval-only, no LLM key needed)
rag-eval DB="deploy-baba.db":
    cargo xtask rag eval --db-path {{ DB }} --retrieval-only

# Run full RAG evaluation with LLM (fetches Anthropic key from Secrets Manager)
rag-eval-full DB="deploy-baba.db" PROFILE="default":
    just aws-check {{ PROFILE }} && \
    ANTHROPIC_API_KEY=$(cargo xtask secret get anthropic-api-key --profile {{ PROFILE }} | tail -1) \
    cargo xtask rag eval --db-path {{ DB }}

# Run RAG eval filtered by category (portfolio, architecture, code, edge-case)
rag-eval-category CATEGORY DB="deploy-baba.db":
    cargo xtask rag eval --db-path {{ DB }} --retrieval-only --category {{ CATEGORY }}

# Upload local RAG index to S3 for a given environment (dev or prod)
rag-push ENV PROFILE="default" DB="deploy-baba.db":
    #!/usr/bin/env bash
    set -euo pipefail
    just aws-check {{ PROFILE }}
    ACCOUNT_ID=$(aws sts get-caller-identity --profile {{ PROFILE }} --query Account --output text)
    BUCKET="deploy-baba-{{ ENV }}-backups-${ACCOUNT_ID}"
    gzip -c {{ DB }} > /tmp/rag-index.db.gz
    echo "Uploading RAG index to s3://${BUCKET}/rag-index.db.gz"
    aws s3 cp /tmp/rag-index.db.gz "s3://${BUCKET}/rag-index.db.gz" --profile {{ PROFILE }}
    rm -f /tmp/rag-index.db.gz
    echo "done: RAG index uploaded to ${BUCKET}"

# Trigger Lambda to ingest RAG index from S3
rag-ingest ENV PROFILE="default":
    #!/usr/bin/env bash
    set -euo pipefail
    just aws-check {{ PROFILE }}
    FN="deploy-baba-{{ ENV }}"
    echo "Invoking ${FN} with action=ingest-rag"
    RESP=$(aws lambda invoke --function-name "${FN}" \
      --payload '{"action":"ingest-rag"}' \
      --cli-binary-format raw-in-base64-out \
      --profile {{ PROFILE }} \
      /tmp/rag-ingest-resp.json 2>&1)
    cat /tmp/rag-ingest-resp.json
    rm -f /tmp/rag-ingest-resp.json
    echo ""
    echo "done: RAG ingest triggered on ${FN}"

# Full RAG sync: rebuild index locally, upload to S3, trigger Lambda ingest
rag-sync ENV PROFILE="default":
    just rag-index-full
    just rag-push {{ ENV }} {{ PROFILE }}
    just rag-ingest {{ ENV }} {{ PROFILE }}

# RAG sync + agent analysis: full sync then run LangGraph quality report
rag-sync-agent ENV="dev" PROFILE="default":
    just rag-index-full
    just rag-eval-full
    just rag-push {{ ENV }} {{ PROFILE }}
    just rag-ingest {{ ENV }} {{ PROFILE }}
    cd services/agent && ANTHROPIC_API_KEY=$(cargo xtask secret get anthropic-api-key --profile {{ PROFILE }} | tail -1) \
    PYTHONPATH=src uv run python -m agent.rag_sync

# ── Local MCP ────────────────────────────────────────────────────────────────

# Build the local mcp-rs server binary
mcp-build:
    cargo build --release --package mcp-rs

# Verify local mcp-rs initialize, tools/list, and resources/list over stdio
mcp-smoke:
    #!/usr/bin/env python3
    import json
    import os
    import subprocess
    import sys

    env = os.environ.copy()
    env["MCP_RS_CONFIG"] = ".mcp-rs.toml"
    env.setdefault("RUST_LOG", "mcp_rs=warn")

    proc = subprocess.Popen(
        ["target/release/mcp-rs"],
        cwd=os.getcwd(),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )

    def read_json_response():
        while True:
            line = proc.stdout.readline()
            if not line:
                raise RuntimeError(proc.stderr.read())
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                # Some local MCP servers may emit startup logs on stdout.
                continue

    def rpc(method, params=None):
        request = {"jsonrpc": "2.0", "id": method, "method": method}
        if params is not None:
            request["params"] = params
        proc.stdin.write(json.dumps(request) + "\n")
        proc.stdin.flush()
        response = read_json_response()
        if "error" in response:
            raise RuntimeError(response["error"])
        return response["result"]

    try:
        init = rpc("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "just-mcp-smoke", "version": "0.1.0"}})
        tools = rpc("tools/list")
        resources = rpc("resources/list")
        cache = rpc("resources/read", {"uri": "project://cache"})
        plans = rpc("resources/read", {"uri": "project://plans"})
        print(f"initialized: {init.get('serverInfo', {}).get('name', 'mcp-rs')}")
        print(f"tools: {len(tools.get('tools', []))}")
        print(f"resources: {len(resources.get('resources', []))}")
        print(f"cache_bytes: {len(cache.get('contents', [{}])[0].get('text', ''))}")
        print(f"plans_bytes: {len(plans.get('contents', [{}])[0].get('text', ''))}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()

# Verify local portfolio RAG MCP tools over stdio
mcp-rag-smoke DB="deploy-baba.db":
    #!/usr/bin/env python3
    import json
    import os
    import subprocess

    env = os.environ.copy()
    env["DATABASE_PATH"] = os.path.abspath("{{ DB }}")
    env["RAG_CORPORA_PATH"] = "/Users/shantopagla/portfolio"
    env.setdefault("RUST_LOG", "portfolio_rag_mcp=warn")

    proc = subprocess.Popen(
        ["cargo", "run", "-q", "-p", "portfolio-rag-mcp"],
        cwd="/Users/shantopagla/portfolio",
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )

    def read_json_response():
        while True:
            line = proc.stdout.readline()
            if not line:
                raise RuntimeError(proc.stderr.read())
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                # Skip startup logs emitted on stdout before JSON-RPC responses.
                continue

    def rpc(method, params=None):
        request = {"jsonrpc": "2.0", "id": method, "method": method}
        if params is not None:
            request["params"] = params
        proc.stdin.write(json.dumps(request) + "\n")
        proc.stdin.flush()
        response = read_json_response()
        if "error" in response:
            raise RuntimeError(response["error"])
        return response["result"]

    try:
        init = rpc("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "just-mcp-rag-smoke", "version": "0.1.0"}})
        tools = rpc("tools/list")
        corpora = rpc("tools/call", {"name": "list_corpora", "arguments": {}})
        results = rpc("tools/call", {"name": "query_rag", "arguments": {"query": "architecture", "corpus": "plan", "top_k": 3, "max_content_length": 200}})
        print(f"initialized: {init.get('serverInfo', {}).get('name', 'portfolio-rag')}")
        tool_count = len(tools.get('tools', []))
        print(f"tools: {tool_count}")
        assert tool_count == 9, f"expected 9 tools, got {tool_count}"
        print(f"corpora: {corpora.get('corpus_count', 0)}")
        print(f"rag_results: {results.get('result_count', 0)}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()

# Inspect recent local mcp-rs audit entries
mcp-audit-tail LINES="25":
    tail -n {{ LINES }} mcp-audit.jsonl

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
    just aws-check {{ PROFILE }} && cargo xtask secret put {{ NAME }} '{{ VALUE }}' --profile {{ PROFILE }}

# Read a secret value from AWS Secrets Manager
secret-get NAME PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask secret get {{ NAME }} --profile {{ PROFILE }}

# List all managed secrets under the deploy-baba/prod/ prefix
secret-list PROFILE="default":
    just aws-check {{ PROFILE }} && cargo xtask secret list --profile {{ PROFILE }}

# Run llm-anthropic live integration tests using the key stored in Secrets Manager.

# Requires AWS auth and the anthropic-api-key secret to be provisioned.
test-llm PROFILE="default":
    just aws-check {{ PROFILE }} && \
    ANTHROPIC_API_KEY=$(cargo xtask secret get anthropic-api-key --profile {{ PROFILE }} | tail -1) \
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
    cargo xtask release tag --kind {{ KIND }} {{ if PUSH != "" { "--push" } else { "" } }}

# Promote the latest dev-v* tag to vX.Y.Z, triggering deploy-prod.yml (with manual approval)
release-promote PUSH="":
    cargo xtask release promote {{ if PUSH != "" { "--push" } else { "" } }}
