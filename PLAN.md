> **ARCHIVED** — This monolith has been converted to the modular plan system.
> See `plans/INDEX.md` for the current plan. This file is kept as historical reference.
> Archived: 2026-03-23

---

# deploy-baba — Portfolio Repository Plan (Revised)
**GitHub:** `shantopagla/deploy-baba`
**Author:** Shanto | **Revised:** 2026-03-10
**Source Repo:** `~/shanto` (Baba Toolchain, ~85K LOC)

---

## Decisions Locked In

| Question | Decision |
|----------|----------|
| Name | `deploy-baba` |
| Developer interface | `justfile` only — xtask is an internal impl detail |
| crates.io publish | After polish (Phase 5 complete) |
| License | MIT |
| Scope | Universal crates + infra-types + xtask + AWS deployment |
| Database | SQLite on EFS + S3 backup (no PostgreSQL) |
| AWS compute | Lambda + Function URL (free tier) OR ECS Fargate Spot (~$5/mo) |
| UI / Portfolio site | `services/ui/` — Axum + Askama + utoipa; IS the deployed Lambda |
| AWS profile validation | `just aws-check` via SSM Parameter Store |
| GitHub | `shantopagla/deploy-baba` |

---

## Guiding Principle: justfile Is the Only Interface

Developers who clone this repo interact **exclusively** through `just` commands.
`cargo xtask` is never mentioned in user-facing documentation — it is purely an
implementation mechanism called by the justfile. This keeps the developer UX
consistent and makes the project approachable regardless of Rust experience level.

```
Developer → just <command> → justfile recipe → cargo xtask <subcommand>
                                              → cargo <command>
                                              → aws <command>
                                              → terraform <command>
                                              → docker <command>
```

---

## Repository Structure

```
shantopagla/deploy-baba/
├── README.md
├── Cargo.toml                       # Workspace manifest (resolver = "2")
├── Cargo.lock
├── justfile                         # THE developer interface — all commands here
├── LICENSE-MIT
├── LICENSE-APACHE
├── CONTRIBUTING.md
├── stack.toml                       # Example stack definition (checked in)
├── stack.example.toml               # Annotated reference config
│
├── .github/
│   └── workflows/
│       └── ci.yml                   # PR gate: fmt + clippy + test + doc-check
│
├── crates/
│   ├── config-core/
│   ├── config-toml/
│   ├── config-yaml/
│   ├── config-json/
│   ├── api-core/
│   ├── api-openapi/
│   ├── api-graphql/
│   ├── api-grpc/
│   ├── api-merger/
│   └── infra-types/                 # Includes SqliteConfig + S3BackupConfig
│
├── services/
│   └── ui/                          # Portfolio site + live API demo — the deployed binary
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs              # Dual-mode: Lambda (if AWS_LAMBDA_FUNCTION_NAME set)
│       │   │                        #            or plain Axum TCP server (local dev)
│       │   ├── router.rs            # All route registration + OpenAPI spec assembly
│       │   ├── openapi.rs           # #[derive(OpenApi)] root spec, aggregates all routes
│       │   ├── routes/
│       │   │   ├── mod.rs
│       │   │   ├── landing.rs       # GET /          → HTML portfolio landing page
│       │   │   ├── docs.rs          # GET /docs      → RapiDoc interactive explorer
│       │   │   ├── health.rs        # GET /health    → {"status":"ok","version":"..."}
│       │   │   └── api/
│       │   │       ├── mod.rs
│       │   │       ├── crates.rs    # GET /api/crates, GET /api/crates/{name}
│       │   │       ├── stack.rs     # GET /api/stack — parsed example stack.toml
│       │   │       └── demo.rs      # POST /api/demo/config/parse
│       │   │                        # POST /api/demo/spec/generate
│       │   └── templates/
│       │       ├── base.html        # Base layout: nav, footer, Tailwind CDN
│       │       └── landing.html     # Hero + architecture diagram + crate map + demos
│
├── xtask/                           # Implementation only — not user-facing
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                  # clap dispatcher
│       ├── build.rs                 # fmt, clippy, compile
│       ├── test.rs                  # unit + quarantine runner
│       ├── coverage.rs              # per-crate coverage floors
│       ├── quality.rs               # full gate (fmt+lint+test+coverage+audit)
│       ├── aws/
│       │   ├── mod.rs               # AWS SDK client setup, profile resolution
│       │   ├── validate.rs          # SSM-based profile validation
│       │   └── ssm.rs               # SSM parameter read/write helpers
│       ├── deploy/
│       │   ├── mod.rs               # Deploy orchestration entry point
│       │   ├── docker.rs            # Image build + tag
│       │   ├── ecr.rs               # Push to ECR Public
│       │   ├── lambda.rs            # Lambda function update (zip + publish)
│       │   └── ecs.rs               # ECS task definition update (always-on option)
│       ├── infra/
│       │   ├── mod.rs               # Terraform wrapper
│       │   ├── terraform.rs         # plan / apply / destroy
│       │   └── bootstrap.rs         # First-run: S3 state bucket + SSM sentinel param
│       └── database/
│           ├── mod.rs
│           ├── backup.rs            # SQLite → S3 (gzip + timestamp)
│           └── restore.rs           # S3 → SQLite (latest or specific version)
│
├── infra/                           # Terraform (generated + hand-maintained)
│   ├── main.tf
│   ├── variables.tf
│   ├── outputs.tf
│   ├── lambda.tf                    # Lambda function + Function URL (no API Gateway)
│   ├── efs.tf                       # EFS for SQLite persistence
│   ├── s3.tf                        # Backup bucket + Terraform state bucket
│   ├── iam.tf                       # Role + policies (least privilege)
│   ├── ssm.tf                       # SSM parameters (sentinel + app config)
│   └── eventbridge.tf               # Scheduled SQLite backup trigger
│
├── examples/
│   ├── 01_multi_format_config/
│   ├── 02_api_spec_generation/
│   ├── 03_spec_merger/
│   └── 04_infra_types/
│
└── docs/
    ├── architecture.md
    ├── aws-setup.md                  # Step-by-step: profile + permissions
    ├── zero-cost-philosophy.md
    └── crate-guide.md
```

---

## justfile — Complete Command Reference

This is the canonical command surface. Every command a developer ever needs is here.

```makefile
# ── Meta ──────────────────────────────────────────────────────────────────────

# List all available commands
default:
    @just --list

# ── Inner Loop (daily dev) ────────────────────────────────────────────────────

# Format all code
fmt:
    cargo xtask build fmt

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
    cargo xtask test --crate {{CRATE}}

# Generate coverage report (opens in browser)
coverage:
    cargo xtask coverage report --open

# fmt + lint + test (the standard inner loop)
dev:
    just fmt && just lint && just test

# Full quality gate (fmt + lint + test + coverage floors + audit)
quality:
    cargo xtask quality gate

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
    cargo run --example {{NAME}}

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

# ── AWS Profile ───────────────────────────────────────────────────────────────

# Validate AWS profile is configured and has required permissions
# Reads the sentinel SSM parameter /deploy-baba/sentinel to verify access
aws-check PROFILE="default":
    cargo xtask aws validate --profile {{PROFILE}}

# Print AWS setup instructions
aws-setup:
    @cat docs/aws-setup.md

# Print current caller identity (who am I in AWS)
aws-whoami PROFILE="default":
    aws sts get-caller-identity --profile {{PROFILE}}

# ── Infrastructure (Terraform) ────────────────────────────────────────────────

# Bootstrap: create S3 state bucket + write sentinel SSM param (first run only)
infra-bootstrap PROFILE="default" REGION="us-east-1":
    cargo xtask infra bootstrap --profile {{PROFILE}} --region {{REGION}}

# Preview infrastructure changes (no apply)
infra-plan PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask infra plan --profile {{PROFILE}}

# Apply infrastructure changes
infra-apply PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask infra apply --profile {{PROFILE}}

# Destroy all infrastructure (prompt confirmation)
infra-destroy PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask infra destroy --profile {{PROFILE}}

# Show current Terraform outputs (API endpoint URL, etc.)
infra-output PROFILE="default":
    cargo xtask infra output --profile {{PROFILE}}

# ── Deployment ────────────────────────────────────────────────────────────────

# Build Docker image locally
build-image:
    cargo xtask deploy docker-build

# Build + push image to ECR Public (requires aws-check to pass)
push-image PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy ecr-push --profile {{PROFILE}}

# Full deploy: build image + push + update Lambda (or ECS task)
deploy PROFILE="default":
    just quality && just push-image {{PROFILE}} && cargo xtask deploy update --profile {{PROFILE}}

# Deploy without running quality gate (fast path, use carefully)
deploy-fast PROFILE="default":
    just push-image {{PROFILE}} && cargo xtask deploy update --profile {{PROFILE}}

# Dry run: build + validate, no push
deploy-dry PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy docker-build --dry-run

# ── Database (SQLite + S3) ────────────────────────────────────────────────────

# Back up SQLite file from EFS to S3 (timestamped gzip)
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

# ── UI / Portfolio Site ───────────────────────────────────────────────────────

# Run the portfolio site locally (plain Axum TCP server, hot-ish reload via cargo watch)
ui:
    cargo watch -x 'run --package deploy-baba-ui'

# Build the UI binary only (fast check)
ui-build:
    cargo build --package deploy-baba-ui

# Tail CloudWatch logs for the deployed Lambda
ui-logs PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask aws logs --function deploy-baba-ui --profile {{PROFILE}}

# Open the live portfolio URL (reads from Terraform outputs)
ui-open PROFILE="default":
    cargo xtask infra output --key function_url --profile {{PROFILE}} | xargs open

# ── crates.io ────────────────────────────────────────────────────────────────

# Dry-run publish for all crates (checks packaging, no upload)
publish-dry:
    cargo xtask publish dry-run

# Publish all crates to crates.io in dependency order (requires CARGO_REGISTRY_TOKEN)
publish:
    just quality && cargo xtask publish release
```

---

## UI Service: `services/ui/`

The portfolio site is a first-class Rust service that **uses the deploy-baba crates
directly** — making it a live demonstration of everything in the repo. It is the
only binary in the workspace and is what gets deployed to AWS Lambda.

### Dual-Mode Entry Point (`main.rs`)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().without_time().init();
    let app = router::build();

    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        // Running on Lambda — adapt Axum router to Lambda HTTP events
        lambda_http::run(app).await?;
    } else {
        // Local dev — plain TCP server on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        println!("→ http://localhost:3000");
        axum::serve(listener, app).await?;
    }
    Ok(())
}
```

No feature flags. Runtime detection is clean, testable, and avoids conditional
compilation complexity. `cargo-lambda` builds with `--release` for Lambda;
`just ui` runs directly for local dev.

### Route Surface (also the OpenAPI spec)

```
GET  /                         HTML portfolio landing page (Askama)
GET  /docs                     RapiDoc interactive API explorer
GET  /health                   {"status":"ok","version":"0.1.0","sha":"abcd1234"}

GET  /api/openapi.json         OpenAPI 3.0 spec (generated by api-openapi crate)
GET  /api/crates               [{name, version, description, traits, doc_url}, ...]
GET  /api/crates/{name}        Full crate metadata + public trait list
GET  /api/stack                Parsed stack.toml rendered as JSON
POST /api/demo/config/parse    Body: {format:"toml"|"yaml"|"json", content:"..."}
                               → Parsed config object or validation errors
POST /api/demo/spec/generate   Body: {fields:[{name,type,required},...], title:"..."}
                               → OpenAPI JSON for a synthetic schema
```

The `/api/demo/*` endpoints actually invoke `config-core` / `config-toml` /
`config-yaml` and `api-openapi` at runtime — the site is a live demo, not a mock.

### Landing Page Design (`landing.html`)

Served server-side via Askama — compiles into the binary, zero runtime file I/O.
Tailwind CDN for styling (acceptable for a portfolio site; no build step needed).

Sections in order:
1. **Hero** — name, tagline, GitHub badge, crates.io badges (coming soon), live API link
2. **Architecture** — the three-layer diagram rendered as an HTML/SVG diagram
3. **Crate Map** — interactive table: click a crate → its `/api/crates/{name}` data
   loads inline showing traits and description
4. **Live Demos** — two embedded forms:
   - Config parser: paste TOML/YAML/JSON, choose format, see parsed output
   - Spec generator: describe a struct, get OpenAPI JSON back
5. **Zero-Cost Philosophy** — short prose section explaining monomorphization
6. **Deploy Your Own** — snippet: `git clone` → `just dev` → `just deploy`

### Key Dependencies (`services/ui/Cargo.toml`)

```toml
[dependencies]
# Internal crates — this is what makes it a live demo
config-core    = { workspace = true }
config-toml    = { workspace = true }
config-yaml    = { workspace = true }
config-json    = { workspace = true }
api-core       = { workspace = true }
api-openapi    = { workspace = true }
api-merger     = { workspace = true }
infra-types    = { workspace = true }

# Web framework
axum           = { workspace = true }
lambda_http    = { workspace = true }
tokio          = { workspace = true }
tower-http     = { workspace = true }

# Templates (compile-time, embedded in binary)
askama         = { workspace = true }
askama_axum    = { workspace = true }

# OpenAPI spec + docs
utoipa         = { workspace = true }
utoipa-rapidoc = { workspace = true }

# Serialization + error handling
serde          = { workspace = true }
serde_json     = { workspace = true }
thiserror      = { workspace = true }
anyhow         = { workspace = true }    # OK in a binary service (not a library)
tracing        = { workspace = true }
tracing-subscriber = { workspace = true }
```

Note: `api-graphql` and `api-grpc` are NOT imported here — the live demo only
covers config parsing and OpenAPI generation. GraphQL/gRPC are demonstrated via
static examples in the crate map section, keeping the Lambda binary lean.

### Lambda Binary Size (target)

Askama templates: ~0 overhead (compile-time).
No database driver in the UI itself (SQLite is for app state, not the portfolio page).
Target: < 10 MB compressed Lambda zip.

```toml
# services/ui/Cargo.toml profile
[profile.release]
lto            = true
codegen-units  = 1
opt-level      = "z"    # optimize for size (Lambda cold start ∝ binary size)
strip          = true
```

### Not Published to crates.io

`services/ui/` is a binary crate. The `publish = false` flag is set in its
`Cargo.toml`. Only the library crates under `crates/` are published.

---

## AWS Architecture (Updated)

Lambda Function URL replaces API Gateway entirely — simpler, cheaper, one fewer
resource to manage in Terraform.

```
  HTTPS Request ──► CloudFront ──► Lambda Function URL ──► Lambda
  (sislam.com)      (cache: off)   (origin, HTTPS-only)

                    ┌──────────────────────────────┐
  HTTPS Request ──► │  Lambda Function URL          │  Free HTTPS endpoint
  (browser/curl)    │  (auth: NONE, CORS: enabled)  │  No API Gateway needed
                    └─────────────┬────────────────┘
                                  │ invokes
                    ┌─────────────▼────────────────┐
                    │  Lambda: deploy-baba-ui       │  Rust binary (Axum adapter)
                    │  Runtime: provided.al2023     │  aarch64, 256MB, ~5ms cold start
                    │  Timeout: 30s                 │
                    └────────┬────────────┬─────────┘
                             │ mount EFS  │ reads
                    ┌────────▼──────┐  ┌──▼───────────────┐
                    │  EFS          │  │  SSM Params       │
                    │  /mnt/db/     │  │  /deploy-baba/*   │
                    │  app.db       │  │  (config values)  │
                    └────────┬──────┘  └──────────────────┘
                             │ scheduled backup
                    ┌────────▼──────┐
                    │  S3           │
                    │  backups/     │  EventBridge: daily
                    └───────────────┘
```

**Cost (Lambda + Function URL option):**

| Service | Free Tier / Always Free | Post-Free-Tier |
|---------|------------------------|----------------|
| Lambda (256MB, <1M req/mo) | 1M req, 400K GB-sec free | ~$0.20/M req |
| Lambda Function URL | Free (no added charge) | Free |
| EFS (SQLite, ~1MB) | 5GB free (12 mo) | ~$0.001/mo |
| S3 backup | 5GB free | ~$0.001/mo |
| ECR Public (image) | Free | Free |
| SSM Standard Params | Always free | Free |
| CloudWatch Logs (basic) | 5GB free | ~$0.50/GB |
| EventBridge (daily backup) | 14M events free | Free |
| **Total** | **$0/month** | **~$0–1/month** |

---

## xtask Module Map (Internal Implementation)

Invoked only by the justfile. Never documented as a user-facing API.

```
xtask/
├── main.rs          → clap subcommands: build | test | coverage | quality |
│                      aws | infra | deploy | database | publish
│
├── build.rs         → fmt [--check], lint, compile
│   CALLED BY:         just fmt, just lint, just check, just build
│
├── test.rs          → unit, all, --crate, quarantine isolation
│   CALLED BY:         just test, just test-all, just test-crate
│
├── coverage.rs      → per-crate floors, HTML report, --open flag
│   CALLED BY:         just coverage
│
├── quality.rs       → orchestrates fmt-check + lint + test + coverage + audit
│   CALLED BY:         just quality
│
├── aws/
│   ├── mod.rs       → shared AWS SDK client, profile resolution from ~/.aws/config
│   ├── validate.rs  → reads /deploy-baba/sentinel from SSM, verifies expected value
│   │                   exits with clear error message if profile not configured
│   └── ssm.rs       → get_parameter, put_parameter helpers
│   CALLED BY:         just aws-check, just aws-whoami (as pre-check in all deploy/infra)
│
├── deploy/
│   ├── mod.rs       → deploy mode selection (lambda vs ecs from stack.toml)
│   │                   The target binary is always `services/ui` (deploy-baba-ui)
│   ├── docker.rs    → docker build --platform linux/arm64, tag
│   │                   (arm64 = Lambda provided.al2023 runtime, cheaper + faster)
│   ├── ecr.rs       → ECR Public auth + push (free tier, public repo)
│   ├── lambda.rs    → cargo lambda build --release, zip, aws lambda update-function-code
│   │                   Reads Function URL from Terraform outputs after deploy
│   └── ecs.rs       → register new task definition, update service (always-on option)
│   CALLED BY:         just build-image, just push-image, just deploy, just deploy-fast
│
├── infra/
│   ├── mod.rs       → terraform wrapper, reads AWS profile + region from stack.toml
│   ├── terraform.rs → init, plan, apply, destroy, output
│   └── bootstrap.rs → create S3 state bucket if not exists, write SSM sentinel param
│                       /deploy-baba/sentinel = "deploy-baba-configured"
│   CALLED BY:         just infra-bootstrap, just infra-plan, just infra-apply,
│                      just infra-destroy, just infra-output
│
└── database/
    ├── mod.rs       → SQLite + S3 config from stack.toml
    ├── backup.rs    → gzip SQLite file, upload to s3://<bucket>/backups/<timestamp>.db.gz
    └── restore.rs   → list S3 objects, download latest (or --version), decompress
    CALLED BY:         just db-backup, just db-restore, just db-restore-version,
                       just db-list-backups
```

---

## AWS Profile Setup (`docs/aws-setup.md`)

A developer cloning this repo needs an AWS profile with a specific IAM policy.
The `just aws-check` command validates setup by reading a known SSM parameter.

### Required IAM Permissions (least privilege)

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "STSValidation",
      "Effect": "Allow",
      "Action": ["sts:GetCallerIdentity"],
      "Resource": "*"
    },
    {
      "Sid": "SSMAccess",
      "Effect": "Allow",
      "Action": ["ssm:GetParameter", "ssm:PutParameter", "ssm:DeleteParameter", "ssm:DescribeParameters"],
      "Resource": "arn:aws:ssm:*:*:parameter/deploy-baba/*"
    },
    {
      "Sid": "ECRPublicPush",
      "Effect": "Allow",
      "Action": ["ecr-public:*"],
      "Resource": "*"
    },
    {
      "Sid": "LambdaFullLifecycle",
      "Effect": "Allow",
      "Action": [
        "lambda:CreateFunction", "lambda:DeleteFunction", "lambda:GetFunction",
        "lambda:UpdateFunctionCode", "lambda:UpdateFunctionConfiguration",
        "lambda:PublishVersion", "lambda:AddPermission", "lambda:RemovePermission",
        "lambda:CreateFunctionUrlConfig", "lambda:UpdateFunctionUrlConfig",
        "lambda:GetFunctionUrlConfig", "lambda:ListFunctions"
      ],
      "Resource": "arn:aws:lambda:*:*:function:deploy-baba-*"
    },
    {
      "Sid": "IAMRolesForTerraform",
      "Effect": "Allow",
      "Action": [
        "iam:CreateRole", "iam:DeleteRole", "iam:GetRole", "iam:ListRoles",
        "iam:AttachRolePolicy", "iam:DetachRolePolicy", "iam:PutRolePolicy",
        "iam:DeleteRolePolicy", "iam:GetRolePolicy", "iam:ListRolePolicies",
        "iam:ListAttachedRolePolicies", "iam:PassRole", "iam:TagRole", "iam:UntagRole"
      ],
      "Resource": "arn:aws:iam::*:role/deploy-baba-*"
    },
    {
      "Sid": "S3StateAndBackups",
      "Effect": "Allow",
      "Action": [
        "s3:CreateBucket", "s3:DeleteBucket", "s3:GetBucketLocation",
        "s3:ListBucket", "s3:GetBucketVersioning", "s3:PutBucketVersioning",
        "s3:GetBucketPolicy", "s3:PutBucketPolicy", "s3:DeleteBucketPolicy",
        "s3:GetObject", "s3:PutObject", "s3:DeleteObject",
        "s3:GetEncryptionConfiguration", "s3:PutEncryptionConfiguration"
      ],
      "Resource": [
        "arn:aws:s3:::deploy-baba-*",
        "arn:aws:s3:::deploy-baba-*/*"
      ]
    },
    {
      "Sid": "DynamoDBTerraformLock",
      "Effect": "Allow",
      "Action": [
        "dynamodb:CreateTable", "dynamodb:DeleteTable", "dynamodb:DescribeTable",
        "dynamodb:GetItem", "dynamodb:PutItem", "dynamodb:DeleteItem"
      ],
      "Resource": "arn:aws:dynamodb:*:*:table/terraform-lock"
    },
    {
      "Sid": "EFSForSQLite",
      "Effect": "Allow",
      "Action": [
        "elasticfilesystem:CreateFileSystem", "elasticfilesystem:DeleteFileSystem",
        "elasticfilesystem:DescribeFileSystems",
        "elasticfilesystem:CreateMountTarget", "elasticfilesystem:DeleteMountTarget",
        "elasticfilesystem:DescribeMountTargets",
        "elasticfilesystem:CreateAccessPoint", "elasticfilesystem:DeleteAccessPoint",
        "elasticfilesystem:DescribeAccessPoints",
        "elasticfilesystem:ClientMount", "elasticfilesystem:ClientWrite",
        "elasticfilesystem:PutLifecycleConfiguration", "elasticfilesystem:TagResource"
      ],
      "Resource": "*"
    },
    {
      "Sid": "EC2VPCForLambdaAndEFS",
      "Effect": "Allow",
      "Action": [
        "ec2:DescribeVpcs", "ec2:DescribeSubnets", "ec2:DescribeSecurityGroups",
        "ec2:CreateSecurityGroup", "ec2:DeleteSecurityGroup",
        "ec2:AuthorizeSecurityGroupIngress", "ec2:AuthorizeSecurityGroupEgress",
        "ec2:RevokeSecurityGroupIngress", "ec2:RevokeSecurityGroupEgress",
        "ec2:DescribeNetworkInterfaces", "ec2:CreateNetworkInterface",
        "ec2:DeleteNetworkInterface", "ec2:DescribeAvailabilityZones"
      ],
      "Resource": "*"
    },
    {
      "Sid": "CloudWatchLogs",
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup", "logs:DeleteLogGroup", "logs:DescribeLogGroups",
        "logs:CreateLogStream", "logs:PutLogEvents", "logs:GetLogEvents",
        "logs:FilterLogEvents", "logs:PutRetentionPolicy"
      ],
      "Resource": "arn:aws:logs:*:*:log-group:/aws/lambda/deploy-baba-*"
    },
    {
      "Sid": "EventBridgeBackupSchedule",
      "Effect": "Allow",
      "Action": [
        "events:CreateRule", "events:DeleteRule", "events:DescribeRule",
        "events:PutRule", "events:PutTargets", "events:RemoveTargets",
        "events:ListTargetsByRule", "events:TagResource"
      ],
      "Resource": "arn:aws:events:*:*:rule/deploy-baba-*"
    }
  ]
}
```

### Local `~/.aws/config` setup

```ini
[profile deploy-baba]
region = us-east-1
output = json
# For SSO users:
sso_start_url = https://your-org.awsapps.com/start
sso_account_id = 123456789012
sso_role_name = DeployBabaDeveloper
sso_region = us-east-1
# For access key users: use ~/.aws/credentials instead
```

### Validation Flow (`just aws-check`)

```
1. Load profile from ~/.aws/config (or use --profile flag)
2. Call sts:GetCallerIdentity (verifies credentials are valid)
3. Call ssm:GetParameter on /deploy-baba/sentinel
4. Assert value == "deploy-baba-configured"
5. Print: ✓ AWS profile 'deploy-baba' is configured correctly
         ✓ Account: 123456789012, Region: us-east-1, User: arn:aws:iam::...
```

The sentinel parameter is created during `just infra-bootstrap`. Trying to validate
before bootstrapping gives a clear error:
```
✗ SSM parameter /deploy-baba/sentinel not found.
  Run `just infra-bootstrap --profile <profile>` first.
```

---

## AWS Architecture: Two Deployment Options

Both options use SQLite on EFS with S3 backup. The difference is compute.

### Option A: Lambda + Function URL (Recommended — near-zero cost)

```
                    ┌──────────────────────────────┐
  HTTPS Request ──► │  Lambda Function URL          │  Free HTTPS endpoint
  (browser/curl)    │  (auth: NONE, CORS: enabled)  │  No API Gateway needed
                    └─────────────┬────────────────┘
                                  │ invokes
                    ┌─────────────▼─────────────────┐
                    │  Lambda: deploy-baba-ui        │  (Rust binary via cargo-lambda)
                    │  Runtime: provided.al2023      │  aarch64, 256MB, ~5ms cold start
                    │  Timeout: 30s                  │
                    └────────┬─────────┬────────────┘
                             │ mount   │ reads
                    ┌────────▼──────┐  ┌──▼───────────────┐
                    │  EFS          │  │  SSM Params       │
                    │  /mnt/db/     │  │  /deploy-baba/*   │
                    │  app.db       │  │  (config values)  │
                    └────────┬──────┘  └──────────────────┘
                             │ scheduled backup
                    ┌────────▼──────┐
                    │  S3           │
                    │  backups/     │  EventBridge: daily
                    └───────────────┘
```

**Cost breakdown (Lambda + Function URL option):**
| Service | Free Tier | Typical Monthly (low traffic) |
|---------|-----------|-------------------------------|
| Lambda (256MB, <1M req/mo) | 1M req, 400K GB-sec/month | $0 |
| Lambda Function URL | Free (no added charge) | Free |
| EFS | 5GB free (first year) | ~$0.001 (tiny SQLite file) |
| S3 backup | 5GB free | ~$0.001 |
| ECR Public | Free | $0 |
| SSM Standard | Free | $0 |
| CloudWatch Logs | 5GB free | $0 |
| EventBridge | 14M events/month free | $0 |
| **Total** | | **~$0/month** (within free tier) |

After free tier expiry: ~$0–1/month at low traffic.

**Implementation notes:**
- Use `cargo-lambda` for cross-compilation and packaging
- Use `lambda_http` crate to adapt Axum router to Lambda
- Lambda reads SQLite via EFS mount at `/mnt/db/deploy-baba.db`
- EFS access point scoped to `/deploy-baba` directory

### Option B: ECS Fargate Spot (always-on, ~$5-7/month)

```
  HTTP Request  ──► NLB ($0.008/LCU-hr) ──► ECS Fargate Spot task
                                              (0.25 vCPU, 0.5GB)
                                              │ mount EFS
                                              └► SQLite ──► S3 backup
```

**Cost breakdown (ECS Fargate Spot):**
| Service | Monthly |
|---------|---------|
| ECS Fargate Spot (0.25vCPU, 0.5GB) | ~$1.50 |
| NLB | ~$5.50 |
| EFS | ~$0.001 |
| S3 backup | ~$0.001 |
| ECR Public | $0 |
| SSM | $0 |
| **Total** | **~$7/month** |

**Deployment mode selection:** Set in `stack.toml`:
```toml
[deploy]
mode = "lambda"    # or "ecs-fargate-spot"
```
The `just deploy` command reads this and delegates to the correct xtask module.

---

## infra-types Crate: SQLite + S3 Changes

`database.rs` replaces the PostgreSQL/MySQL engine concept entirely:

```rust
// infra-types/src/database.rs

/// SQLite database configuration with optional S3 backup.
/// This is the only database type supported in deploy-baba.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    /// Filesystem path to the SQLite file (e.g. "/mnt/db/deploy-baba.db")
    pub path: PathBuf,

    /// S3 backup configuration. None means no automatic backup.
    pub backup: Option<S3BackupConfig>,

    /// WAL mode enabled by default for concurrent reads
    #[serde(default = "default_wal")]
    pub wal_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3BackupConfig {
    /// S3 bucket name (must already exist — created by Terraform)
    pub bucket: String,

    /// Key prefix for backup objects, e.g. "backups/"
    #[serde(default = "default_prefix")]
    pub prefix: String,

    /// Backup retention: number of versions to keep in S3
    #[serde(default = "default_retention")]
    pub retain_versions: u32,

    /// Backup schedule as a cron expression (EventBridge format)
    /// Default: "rate(1 day)"
    #[serde(default = "default_schedule")]
    pub schedule: String,
}
```

`stack.toml` reflects this:
```toml
[database]
path = "/mnt/db/deploy-baba.db"
wal_mode = true

[database.backup]
bucket = "deploy-baba-backups-<account-id>"
prefix = "backups/"
retain_versions = 7
schedule = "rate(1 day)"
```

---

## xtask: New Modules (deploy, infra, aws, database)

These are the entirely new modules — the existing build/test/coverage/quality modules
are extracted from `~/shanto` with cleanup. These four are written fresh.

### `xtask/src/aws/validate.rs` — SSM validation

```rust
pub async fn validate_profile(profile: &str) -> Result<CallerInfo, AwsError> {
    let config = load_aws_config_for_profile(profile).await?;
    let sts   = aws_sdk_sts::Client::new(&config);
    let ssm   = aws_sdk_ssm::Client::new(&config);

    // Step 1: verify credentials
    let identity = sts.get_caller_identity().send().await
        .map_err(|_| AwsError::InvalidCredentials(profile.to_string()))?;

    // Step 2: verify SSM access + sentinel param
    let param = ssm.get_parameter()
        .name("/deploy-baba/sentinel")
        .send().await
        .map_err(|_| AwsError::SsmNotBootstrapped)?;

    if param.parameter().and_then(|p| p.value()) != Some("deploy-baba-configured") {
        return Err(AwsError::SentinelMismatch);
    }

    Ok(CallerInfo { account: identity.account, arn: identity.arn, profile: profile.to_string() })
}
```

### `xtask/src/infra/bootstrap.rs` — First-run setup

On first run (`just infra-bootstrap`):
1. Create S3 bucket for Terraform state (`deploy-baba-tfstate-<account-id>`)
2. Enable S3 versioning on state bucket
3. Write SSM standard parameter `/deploy-baba/sentinel = "deploy-baba-configured"`
4. Write SSM parameter `/deploy-baba/region = <region>`
5. Write SSM parameter `/deploy-baba/account = <account-id>`
6. Run `terraform init` with S3 backend config

After `just infra-apply`, key Terraform outputs include:
- `function_url` — the live HTTPS endpoint (e.g. `https://abc123.lambda-url.us-east-1.on.aws/`)
- `efs_id` — EFS file system ID
- `backup_bucket` — S3 bucket name for SQLite backups

`just infra-output` prints these. `just ui-open` reads `function_url` and opens it.

### `xtask/src/deploy/lambda.rs` — Lambda deployment

```
1. cargo lambda build --release --target aarch64-unknown-linux-gnu
2. cargo lambda package --output-format zip
3. aws lambda update-function-code --function-name deploy-baba-api \
       --zip-file fileb://target/lambda/deploy-baba-api/bootstrap.zip
4. aws lambda publish-version
5. Print: ✓ Lambda updated, new version: <n>
```

### `xtask/src/database/backup.rs` — SQLite S3 backup

```
1. Read SqliteConfig from stack.toml
2. Connect to EFS-mounted SQLite at config.path (or via SSH/Lambda invoke in CI)
3. Run VACUUM INTO '/tmp/backup.db' (creates clean copy without WAL)
4. gzip /tmp/backup.db → /tmp/backup-<timestamp>.db.gz
5. Upload to s3://<bucket>/<prefix><timestamp>.db.gz
6. List all backups, delete oldest if count > retain_versions
7. Print: ✓ Backup complete: s3://.../backups/2026-03-10T14:00:00Z.db.gz
```

---

## Revised xtask Module Table

### Extracted from `~/shanto` (cleanup only)

| Module | Source | Work Needed |
|--------|--------|-------------|
| `build.rs` | `xtask/src/build.rs` | Strip Baba targets, update crate names |
| `test.rs` | `xtask/src/test.rs` | Keep quarantine logic, strip service-specific groups |
| `coverage.rs` | `xtask/src/coverage.rs` | Update crate names + floors |
| `quality.rs` | `xtask/src/quality.rs` | Update to use new crate list |

### New modules (write fresh)

| Module | Lines est. | Notes |
|--------|-----------|-------|
| `aws/mod.rs` + `validate.rs` + `ssm.rs` | ~200 | AWS SDK v2 (aws-sdk-sts, aws-sdk-ssm) |
| `infra/mod.rs` + `terraform.rs` + `bootstrap.rs` | ~300 | std::process::Command wrapping terraform |
| `deploy/mod.rs` + `docker.rs` + `ecr.rs` + `lambda.rs` + `ecs.rs` | ~400 | cargo-lambda integration |
| `database/mod.rs` + `backup.rs` + `restore.rs` | ~250 | aws-sdk-s3, flate2 for gzip |

### Drop entirely (Baba-internal, not applicable)

`auth/`, `ci/`, `cloud/` — Baba-specific SSO flows, CI provider abstractions, ECS-specific monitoring.

---

## Crate Inventory (unchanged from v1 plan)

| Crate | Source | Work |
|-------|--------|------|
| `config-core` | `crates/rust-config-core` | Rename, add `ConfigSource`, polish docs |
| `config-toml` | `crates/rust-config-toml` | Rename, minor cleanup |
| `config-yaml` | `crates/rust-config-yaml` | Complete stub (~200 lines) |
| `config-json` | `crates/rust-config-json` | Complete stub (~200 lines) |
| `api-core` | `crates/rust-api-core` | Remove internal fields, AsyncAPI stub |
| `api-openapi` | `crates/rust-api-openapi` | Extract, polish |
| `api-graphql` | `crates/rust-api-graphql` | Complete stub (~250 lines) |
| `api-grpc` | `crates/rust-api-grpc` | Complete stub (~250 lines) |
| `api-merger` | `crates/rust-api-merger` | Extra strategies, rename internals |
| `infra-types` | `services/baba-stack` | Sanitize + replace DB with SqliteConfig |

---

## Dependency Order

```
config-core          (no internal deps)
config-toml          → config-core
config-yaml          → config-core
config-json          → config-core
api-core             (no internal deps)
api-openapi          → api-core
api-graphql          → api-core
api-grpc             → api-core
api-merger           → api-core, api-openapi, api-graphql, api-grpc
infra-types          → config-core (optional feature), serde

services/ui          → config-core, config-toml, config-yaml, config-json,
                       api-core, api-openapi, api-merger, infra-types
                       (binary — not published to crates.io)
```

---

## Key External Dependencies

```toml
# ── Library crates (config-*, api-*, infra-types) ─────────────────────────────
serde         = { version = "1", features = ["derive"] }
thiserror     = "2"

# Config layer
toml          = "0.8"
serde_yaml    = "0.9"
serde_json    = "1"

# API layer
utoipa        = { version = "4", features = ["axum_extras"] }

# ── services/ui ───────────────────────────────────────────────────────────────
axum          = "0.7"
lambda_http   = "0.13"          # Lambda ↔ Axum adapter; used at runtime on Lambda
tower-http    = { version = "0.5", features = ["cors", "trace"] }
askama        = "0.12"          # Compile-time templates → embedded in binary
askama_axum   = "0.4"
utoipa-rapidoc = "4"            # RapiDoc at /docs
anyhow        = "1"             # OK in binary (not library)
tracing       = "0.1"
tracing-subscriber = "0.3"

# ── xtask (internal tooling, not published) ───────────────────────────────────
aws-config    = { version = "1", features = ["behavior-version-latest"] }
aws-sdk-sts   = "1"
aws-sdk-ssm   = "1"
aws-sdk-s3    = "1"
aws-sdk-ecr   = "1"    # ECR Public push
aws-sdk-lambda = "1"   # Lambda update-function-code
aws-sdk-efs   = "1"    # EFS describe (for db-backup target path)
aws-sdk-ecs   = "1"    # Fargate Spot option
clap          = { version = "4", features = ["derive"] }
tokio         = { version = "1", features = ["full"] }
flate2        = "1"    # gzip for SQLite backups

# ── External CLI tools (installed separately, not Cargo deps) ─────────────────
# cargo-lambda  → `cargo install cargo-lambda`   (Lambda packaging)
# terraform     → `brew install terraform`        (infra management)
# cargo-watch   → `cargo install cargo-watch`     (just ui hot reload)
# cargo-audit   → `cargo install cargo-audit`     (just audit)
# cargo-llvm-cov → `cargo install cargo-llvm-cov` (just coverage)
```

---

## Coverage Floors

```
config-core:    90%
api-core:       90%
config-toml:    85%
config-yaml:    85%
config-json:    85%
api-openapi:    80%
api-graphql:    80%
api-grpc:       80%
api-merger:     80%
infra-types:    75%
```

---

## Build Phases (Revised)

### Phase 1 — Scaffold (½ day)
- [ ] `git init`, workspace `Cargo.toml`, justfile, dual licenses
- [ ] All 10 crate + `services/ui/` + xtask directory stubs
- [ ] `stack.toml` + `stack.example.toml` with SQLite + Lambda config
- [ ] `.github/workflows/ci.yml`
- [ ] `CONTRIBUTING.md`, `docs/aws-setup.md` skeleton

### Phase 2 — Extract & Clean Library Crates (1–2 days)
- [ ] `config-core`, `config-toml` from `~/shanto/crates/`
- [ ] `api-core`, `api-openapi`, `api-merger` from `~/shanto/crates/`
- [ ] `infra-types` from `services/baba-stack` — sanitize, replace DB types with SqliteConfig
- [ ] `cargo build --workspace` clean

### Phase 3 — Complete Library Stubs (1 day)
- [ ] `config-yaml`, `config-json` implementations
- [ ] `api-graphql`, `api-grpc` implementations

### Phase 4 — xtask (1 day)
- [ ] Extract build/test/coverage/quality from `~/shanto/xtask/`
- [ ] Write `aws/` module (validate, SSM helpers)
- [ ] Write `infra/` module (terraform wrapper, bootstrap)
- [ ] Write `deploy/` module (docker, ECR Public, lambda — targets `services/ui/`)
- [ ] Write `database/` module (backup, restore)
- [ ] All justfile commands wired and tested locally

### Phase 5 — UI Service (1–1.5 days)
- [ ] `services/ui/` scaffolded: `main.rs` with dual-mode entry point
- [ ] `router.rs` with all routes registered + OpenAPI spec assembled via utoipa
- [ ] `routes/landing.rs` — Askama template handler
- [ ] `routes/api/crates.rs` — crate metadata endpoints (data hardcoded from workspace)
- [ ] `routes/api/stack.rs` — parse and serve example stack.toml via config-toml
- [ ] `routes/api/demo.rs` — live config-parse + spec-generate endpoints
- [ ] `routes/docs.rs` — RapiDoc at `/docs`
- [ ] `templates/landing.html` — hero, architecture diagram, crate map, live demos, deploy snippet
- [ ] `templates/base.html` — layout with Tailwind CDN
- [ ] `just ui` runs clean locally; all routes respond correctly
- [ ] `cargo lambda build --release` produces valid Lambda zip

### Phase 6 — Terraform + End-to-End Deploy (½ day)
- [ ] `infra/lambda.tf` — Lambda function, Function URL, EFS mount, IAM role
- [ ] `infra/efs.tf`, `infra/s3.tf`, `infra/ssm.tf`, `infra/eventbridge.tf`
- [ ] `just infra-bootstrap` → `just infra-apply` → `just deploy` works end-to-end
- [ ] `just ui-open` opens the live Function URL in browser
- [ ] `just db-backup` and `just db-restore` tested against deployed EFS
- [ ] `infra/cdn.tf` — CloudFront distribution + Route53 records for sislam.com
- [ ] Verify ACM cert covers sislam.com (or create new cert with DNS validation)
- [ ] `https://sislam.com` and `https://www.sislam.com` resolve and serve the portfolio

### Phase 7 — Examples + Docs (1 day)
- [ ] 4 standalone examples, each <100 lines
- [ ] Top-level README — ASCII diagram + crate map + quick-start + link to live site
- [ ] Per-crate README files
- [ ] `docs/aws-setup.md` complete with IAM policy + config snippets
- [ ] `docs/architecture.md`, `docs/zero-cost-philosophy.md`
- [ ] All public traits have rustdoc examples

### Phase 8 — Quality Pass (½ day)
- [ ] `just quality` passes clean (library crates only — UI binary excluded from coverage gate)
- [ ] All library coverage floors met
- [ ] `cargo audit` clean
- [ ] GitHub Actions green end-to-end

### Phase 9 — Publish (after polish)
- [ ] `just publish-dry` passes for all 10 library crates
- [ ] Tag `v0.1.0`
- [ ] `just publish` (publishes in dependency order)
- [ ] Live Function URL added to README as "See it live" link

**Total Estimate: ~7–8 days of focused work**

---

## What a New Developer Does (First-Time Setup)

```bash
git clone https://github.com/shantopagla/deploy-baba
cd deploy-baba

# 1. Verify Rust toolchain + install CLI tools
rustup show
cargo install cargo-lambda cargo-watch cargo-audit cargo-llvm-cov

# 2. Inner loop — no AWS needed
just dev                        # fmt + lint + test
just ui                         # portfolio site at http://localhost:3000

# 3. Optional: set up AWS and deploy the portfolio site live
# Follow docs/aws-setup.md to configure ~/.aws/config + IAM permissions
just aws-check deploy-baba          # validates profile via SSM
just infra-bootstrap deploy-baba    # first time only: S3 state bucket + SSM sentinel
just infra-apply deploy-baba        # provisions Lambda, EFS, S3 backup, Function URL
just deploy deploy-baba             # quality gate → build → push → Lambda update
just ui-open deploy-baba            # opens the live Function URL in browser
```

---

*Plan revised to use justfile as sole developer interface. xtask is implementation only.*
*SQLite + S3 replaces PostgreSQL. Lambda + API Gateway recommended for near-zero cost.*

---

## Deployment Drift Log — Phase 6 (2026-03-18)

Recorded during the first real `terraform init` + `plan` + `apply` run. All items below represent gaps between the plan and actual state, fixed in-session unless noted.

### 1. Backend bootstrap not automated: S3 bucket and DynamoDB table missing

**Plan says:** `just infra-bootstrap` creates S3 state bucket + SSM sentinel.
**Reality:** `infra-bootstrap` was not yet implemented in xtask. The S3 bucket (`deploy-baba-tfstate`) and DynamoDB lock table (`terraform-lock`) were absent, causing `terraform init` to fail immediately.

**Fix applied:** Manually bootstrapped via AWS CLI:
```bash
aws s3api create-bucket --bucket deploy-baba-tfstate --region us-east-1
aws s3api put-bucket-versioning --bucket deploy-baba-tfstate --versioning-configuration Status=Enabled
aws s3api put-bucket-encryption ...  # AES256
aws s3api put-public-access-block ... # block all public access
aws dynamodb create-table --table-name terraform-lock \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST
```

**Required plan update:** `xtask/src/infra/bootstrap.rs` must also create the DynamoDB lock table (it was only mentioned for S3 in the plan). The `just infra-bootstrap` command must be implemented and tested before any new developer can run `just infra-apply`.

**Also:** The plan says bucket name is `deploy-baba-tfstate-<account-id>` but `infra/main.tf` uses `deploy-baba-tfstate` (no suffix). These must be kept in sync.

---

### 2. Security group cycle in `infra/efs.tf`

**Plan says:** EFS SG allows NFS ingress from Lambda SG; Lambda SG egresses to EFS SG.
**Reality:** Both `aws_security_group.efs` and `aws_security_group.lambda_efs` referenced each other inline (ingress/egress blocks), creating a Terraform cycle error:
`Error: Cycle: aws_security_group.efs, aws_security_group.lambda_efs`

**Fix applied:** Extracted the cross-SG rules into separate resources:
```hcl
resource "aws_security_group_rule" "efs_ingress_from_lambda" { ... }
resource "aws_security_group_rule" "lambda_egress_to_efs"    { ... }
```

**Required plan update:** Phase 6 checklist should note: *"Security groups for EFS/Lambda must use separate `aws_security_group_rule` resources for cross-references (inline rules create a cycle)."*

---

### 3. `aws_efs_mount_target` does not support `tags`

**Reality:** The AWS provider rejects `tags` on `aws_efs_mount_target`. Terraform error:
`Error: Argument named "tags" is not expected here`

**Fix applied:** Removed `tags` block from `aws_efs_mount_target.baba_db`.

---

### 4. `lambda.tf` `depends_on` referenced non-existent `aws_iam_role_policy_attachment` resources

**Reality:** `lambda.tf` listed `aws_iam_role_policy_attachment.lambda_efs/s3/ssm` in `depends_on`, but `iam.tf` defines those as `aws_iam_role_policy` (inline policies, not attachments). Also missing `lambda_vpc` from the dependency list.

**Fix applied:** Updated `depends_on` to:
```hcl
depends_on = [
  aws_cloudwatch_log_group.lambda,
  aws_iam_role_policy_attachment.lambda_logs,
  aws_iam_role_policy_attachment.lambda_vpc,
  aws_iam_role_policy.lambda_efs,
  aws_iam_role_policy.lambda_s3,
  aws_iam_role_policy.lambda_ssm,
]
```

---

### 5. Lambda zip (`build/lambda.zip`) missing — `filebase64sha256` fails at plan time

**Plan says:** `just deploy` runs `cargo lambda build --release` and produces the zip.
**Reality:** `infra/variables.tf` defaults `lambda_code_path = "./build/lambda.zip"`. Terraform's `filebase64sha256()` is evaluated at **plan time**, so `terraform plan` fails if the file doesn't exist — before any deploy has ever run.

**Fix required (not yet applied):** Two options:
- **Option A (preferred):** Build the Lambda zip as a prerequisite before `terraform plan/apply`. Add a `just lambda-build` recipe that runs `cargo lambda build --release --target aarch64-unknown-linux-gnu` and packages the zip to `infra/build/lambda.zip`. Wire it into `just infra-apply` as a dependency.
- **Option B:** Use `try()` or a `null_resource`/`external` data source to make the hash optional on first run, though this is fragile.

**cargo-lambda not installed:** The tool is listed in the new-developer setup (`cargo install cargo-lambda`) but was not present in the current environment. Must be installed before `just deploy` can work.

**Required plan update:** Phase 6 checklist must include:
- `cargo install cargo-lambda` (add to bootstrap docs / dev-setup)
- `just lambda-build` — new justfile recipe wrapping `cargo lambda build`
- `just infra-apply` must depend on `just lambda-build` (or at minimum document the prerequisite)

---

### 6. Terraform provider deprecation warnings (non-blocking, will become errors)

| File | Warning | Fix needed |
|------|---------|-----------|
| `infra/eventbridge.tf:6` | `is_enabled` deprecated — use `state` | Replace `is_enabled = true` with `state = "ENABLED"` |
| `infra/s3.tf:41` | `aws_s3_bucket_lifecycle_configuration` rule missing `filter` block | Add `filter {}` to each lifecycle rule |

These are warnings in provider v5.100 but will become errors in a future version. Should be fixed before next plan/apply.

---

### Phase 6 Checklist (Updated)

- [x] `infra/cdn.tf` — CloudFront distribution + Route53 records
- [x] `infra/outputs.tf` — cloudfront_distribution_id, cloudfront_domain_name, site_url added
- [ ] **Bootstrap prerequisite:** Implement `xtask/src/infra/bootstrap.rs` to create S3 bucket + DynamoDB table + SSM sentinel atomically
- [ ] **Lambda build prerequisite:** Add `just lambda-build` recipe; wire into `just infra-apply`
- [ ] **Install `cargo-lambda`:** Add to dev-setup docs and bootstrap step
- [ ] **Fix `eventbridge.tf`:** Replace `is_enabled` with `state = "ENABLED"`
- [ ] **Fix `s3.tf` lifecycle rules:** Add `filter {}` block to each rule
- [ ] `terraform plan` clean (zero errors, zero warnings)
- [ ] `terraform apply` — 28 resources created
- [ ] CloudFront propagates (~5–15 min)
- [ ] `curl -I https://sislam.com` → 200
- [ ] `curl -I https://www.sislam.com` → 200
- [ ] `curl https://sislam.com/health` → `{"status":"ok",...}`

---

## xtask / justfile Drift Log — Phase 6 (2026-03-18)

Audit of gaps between `justfile` recipes and their `cargo xtask` implementations.
All items below have been fixed in this session.

### 1. `infra/bootstrap.rs` — Wrong bucket name, missing DynamoDB, missing security hardening

| Field | Was | Fixed To |
|-------|-----|----------|
| S3 bucket name | `deploy-baba-terraform-state` | `deploy-baba-tfstate` |
| DynamoDB lock table | **missing** | `terraform-lock` (PAY_PER_REQUEST, LockID hash key) |
| S3 encryption | **missing** | AES256 server-side encryption |
| S3 public access block | **missing** | `block_public_acls/policy`, `ignore_public_acls`, `restrict_public_buckets = true` |
| SSM sentinel value | `"bootstrap-complete"` | `"deploy-baba-configured"` |
| terraform init after bootstrap | **missing** | Added — calls `terraform -chdir=infra init` after all AWS resources |

**Just recipe:** `just infra-bootstrap PROFILE REGION` → `cargo xtask infra bootstrap --profile --region`

### 2. `infra/terraform.rs` — Wrong directory handling, no profile propagation

| Issue | Was | Fixed |
|-------|-----|-------|
| Dir passed as positional arg | `cmd.arg(dir)` (invalid for terraform) | `cmd.arg(format!("-chdir={}", dir))` before subcommand |
| Default dir | None (ran from CWD) | Defaults to `"infra"` |
| AWS profile for terraform | Not set | `cmd.env("AWS_PROFILE", profile)` on subprocess |
| `terraform output` | No `-json` flag | Added `-json` for machine-readable output |

### 3. `infra/mod.rs` — Missing `--profile` and `--region` args on terraform actions

`InfraAction::Plan/Apply/Destroy/Output` all had no `--profile` argument, but
`justfile` recipes call them with `--profile {{PROFILE}}`. Added `profile: Option<String>`
to all four actions. Added `region: Option<String>` to `Bootstrap`.

### 4. `deploy/lambda.rs` — Wrong build command, zip path, and function name

| Field | Was | Fixed To |
|-------|-----|----------|
| Build command | `cargo build --release` | `cargo lambda build --release --package deploy-baba-ui --target aarch64-unknown-linux-gnu` |
| Zip path | `lambda-deployment.zip` (CWD) | `infra/build/lambda.zip` |
| Zip contents | `target/release/` directory | `target/lambda/deploy-baba-ui/bootstrap` (single binary) |
| Default function name | `deploy-baba-api` | `deploy-baba-prod` |
| AWS profile | Hardcoded `None` | Accepts `--profile` and passes to `create_aws_config` |

### 5. `deploy/mod.rs` — Lambda action missing `--profile`

Added `profile: Option<String>` to `DeployAction::Lambda`.
Fixed call in `main.rs` that hardcoded `Lambda { function: None }` to include `profile: None`.

### 6. `justfile` — Missing deployment recipes

Added:
- `lambda-build` — wraps `cargo lambda build` with correct package/target flags; sets `PATH` to include `~/.cargo/bin` for rustup-managed toolchains
- `lambda-deploy PROFILE` — `aws-check` + `lambda-build` + `cargo xtask deploy lambda --profile`
- `infra-verify DOMAIN` — curl verification of apex + www HTTPS and `/health` endpoint

### 7. `Cargo.toml` (workspace) — Missing `aws-sdk-dynamodb`

Added `aws-sdk-dynamodb = "1"` to `[workspace.dependencies]` and `xtask/Cargo.toml`.

### Phase 6 xtask Checklist

- [x] `xtask/src/infra/bootstrap.rs` — fixed bucket name, added DynamoDB, encryption, public-access-block, sentinel, terraform init
- [x] `xtask/src/infra/terraform.rs` — fixed `-chdir`, default dir, AWS_PROFILE env var, `-json` on output
- [x] `xtask/src/infra/mod.rs` — added `--profile` to Plan/Apply/Destroy/Output, `--region` to Bootstrap
- [x] `xtask/src/deploy/lambda.rs` — fixed build command, zip path, function name, profile support
- [x] `xtask/src/deploy/mod.rs` — added `--profile` to Lambda action
- [x] `xtask/src/main.rs` — fixed `Lambda { function: None }` to include `profile: None`
- [x] `justfile` — added `lambda-build`, `lambda-deploy`, `infra-verify`
- [x] `Cargo.toml` — added `aws-sdk-dynamodb = "1"` to workspace
- [x] `xtask` compiles clean (`cargo build --package xtask` ✅)
