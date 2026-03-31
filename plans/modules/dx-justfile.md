# W-DX: dx-justfile (justfile + docs + examples)
**Path:** `justfile`, `docs/`, `examples/` | **Status:** WIP
**Depends on:** W-XT (justfile calls xtask) | **Depended on by:** (developer-facing)

---

## W-DX.1 Purpose

The complete developer-facing interface. The justfile is the only entry point developers
interact with. Docs and examples make the crates approachable.

→ ADR-001 (justfile is the only interface)

---

## W-DX.2 Complete Command Reference

```makefile
# ── Meta ──────────────────────────────────────────────────────────────────────
default:
    @just --list

# ── Inner Loop (daily dev) ────────────────────────────────────────────────────
fmt:
    cargo xtask build format
lint:
    cargo xtask build lint
check:
    cargo check --workspace
test:
    cargo xtask test unit
test-all:
    cargo xtask test all
test-crate CRATE:
    cargo xtask test crate {{CRATE}}
coverage:
    cargo xtask coverage report --open
dev:
    just fmt && just lint && just test
quality:
    cargo xtask quality all
build:
    cargo build --workspace --release

# ── Documentation ─────────────────────────────────────────────────────────────
docs:
    cargo doc --no-deps --workspace --open
doc-check:
    cargo doc --no-deps --workspace

# ── Examples ──────────────────────────────────────────────────────────────────
example NAME:
    cargo run --example {{NAME}}

# ── Utilities ─────────────────────────────────────────────────────────────────
clean:
    cargo clean
update:
    cargo update
audit:
    cargo audit

# ── AWS Profile ───────────────────────────────────────────────────────────────
aws-check PROFILE="default":
    cargo xtask aws validate --profile {{PROFILE}}
aws-setup:
    @cat docs/aws-setup.md
aws-whoami PROFILE="default":
    aws sts get-caller-identity --profile {{PROFILE}}

# ── Infrastructure (OpenTofu) ─────────────────────────────────────────────────
infra-bootstrap PROFILE="default" REGION="us-east-1":
    cargo xtask infra bootstrap --profile {{PROFILE}} --region {{REGION}}
infra-plan PROFILE="default":
    just lambda-build && just aws-check {{PROFILE}} && cargo xtask infra plan --profile {{PROFILE}}
infra-apply PROFILE="default":
    just lambda-build && just aws-check {{PROFILE}} && cargo xtask infra apply --profile {{PROFILE}}
infra-destroy PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask infra destroy --profile {{PROFILE}}
infra-output PROFILE="default":
    cargo xtask infra output --profile {{PROFILE}}
infra-verify DOMAIN:
    curl -sI https://{{DOMAIN}} | head -1
    curl -sI https://www.{{DOMAIN}} | head -1
    curl -s https://{{DOMAIN}}/health

# ── Deployment ────────────────────────────────────────────────────────────────
lambda-build:
    PATH="$HOME/.cargo/bin:$PATH" cargo lambda build --release --package deploy-baba-ui \
      --target aarch64-unknown-linux-gnu
lambda-deploy PROFILE="default":
    just aws-check {{PROFILE}} && just lambda-build && \
      cargo xtask deploy lambda --profile {{PROFILE}}
build-image:
    cargo xtask deploy docker-build
push-image PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy ecr-push --profile {{PROFILE}}
deploy PROFILE="default":
    just quality && just push-image {{PROFILE}} && cargo xtask deploy update --profile {{PROFILE}}
deploy-fast PROFILE="default":
    just push-image {{PROFILE}} && cargo xtask deploy update --profile {{PROFILE}}
deploy-dry PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask deploy docker-build --dry-run

# ── Database (SQLite + S3) ────────────────────────────────────────────────────
db-backup PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database backup --profile {{PROFILE}}
db-restore PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database restore --profile {{PROFILE}}
db-restore-version VERSION PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database restore --version {{VERSION}} --profile {{PROFILE}}
db-list-backups PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask database list-backups --profile {{PROFILE}}

# ── UI / Portfolio Site ───────────────────────────────────────────────────────
ui:
    cargo watch -x 'run --package deploy-baba-ui'
ui-build:
    cargo build --package deploy-baba-ui
ui-logs PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask aws logs --function deploy-baba-ui --profile {{PROFILE}}
ui-open PROFILE="default":
    cargo xtask infra output --key function_url --profile {{PROFILE}} | xargs open

# ── crates.io ─────────────────────────────────────────────────────────────────
publish-dry:
    cargo xtask publish dry-run
publish:
    just quality && cargo xtask publish release

# ── Agent Cache ───────────────────────────────────────────────────────────────
cache-status:
    cargo xtask cache status
cache-refresh:
    cargo xtask cache refresh
cache-clear:
    cargo xtask cache clear
```

---

## W-DX.3 Implementation Notes

### W-DX.3.1 Per-Crate READMEs (TODO)

Each of the 10 library crates needs a `README.md`. Template:

```markdown
# <crate-name>

<one-sentence description>

## Usage

```rust
// minimal working example
```

## Features
- <trait/struct 1>
- <trait/struct 2>

## License
MIT OR Apache-2.0
```

Status:
| Crate | README | Notes |
|-------|--------|-------|
| config-core | TODO | |
| config-toml | TODO | |
| config-yaml | TODO | |
| config-json | TODO | |
| api-core | TODO | |
| api-openapi | TODO | |
| api-graphql | TODO | |
| api-grpc | TODO | |
| api-merger | TODO | |
| infra-types | TODO | |

### W-DX.3.2 Examples (TODO)

4 standalone examples under `examples/`:

| Directory | Demonstrates | Status |
|-----------|-------------|--------|
| `01_multi_format_config/` | Parse TOML + YAML + JSON with same ConfigSource trait | TODO |
| `02_api_spec_generation/` | Generate OpenAPI spec from ApiSchema | TODO |
| `03_spec_merger/` | Merge two ApiSchemas, generate all three formats | TODO |
| `04_infra_types/` | Deserialize stack.toml into StackConfig | TODO |

Each example < 100 lines, compiles standalone, has inline comments.

### W-DX.3.3 New Developer Setup

```bash
git clone https://github.com/shantopagla/deploy-baba
cd deploy-baba

# 1. Verify Rust toolchain + install CLI tools
rustup show
cargo install cargo-lambda cargo-watch cargo-audit cargo-llvm-cov

# 2. Inner loop — no AWS needed
just dev                            # fmt + lint + test
just ui                             # portfolio site at http://localhost:3000

# 3. Optional: set up AWS and deploy
# Follow docs/aws-setup.md to configure ~/.aws/config + IAM permissions
just aws-check deploy-baba          # validates profile via SSM
just infra-bootstrap deploy-baba    # first time only: S3 state + DynamoDB + SSM sentinel
just infra-apply deploy-baba        # provisions Lambda, EFS, S3, CloudFront
just lambda-deploy deploy-baba      # build + upload Lambda zip
just infra-verify sislam.com        # verify HTTPS endpoints respond
```

### W-DX.3.4 Docs (TODO — Phase 7)

| File | Status | Notes |
|------|--------|-------|
| `docs/aws-setup.md` | TODO | Full IAM policy + profile config (content in cross-cutting/aws-setup-spec.md) |
| `docs/architecture.md` | TODO | Three-layer diagram + crate map |
| `docs/zero-cost-philosophy.md` | TODO | → ADR-005 |
| `docs/crate-guide.md` | TODO | Which crate to use for what |

---

## W-DX.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-DX.3.1 | Per-crate READMEs (10 crates) | TODO | See W-DX.3.1 table above |
| W-DX.4.1 | 4 example directories | TODO | See W-DX.3.2 table above |
| W-DX.5.1 | Top-level README | TODO | ASCII diagram + crate map + quick-start + live link |
| W-DX.6.1 | docs/aws-setup.md | TODO | Copy from cross-cutting/aws-setup-spec.md |
| W-DX.6.2 | docs/zero-cost-philosophy.md | TODO | |

---

## W-DX.5 Test Strategy

Integration testing via `just dev` (fmt + lint + test). The justfile itself is exercised
end-to-end by running each recipe. No unit tests for the justfile; correctness is verified
by the xtask quality gate and CI.

---

## W-DX.6 Cross-References
- → ADR-001 (justfile philosophy)
- → W-XT (xtask implementation behind every recipe)
- → `plans/cross-cutting/aws-setup-spec.md` — IAM policy for docs
- → `plans/drift/DRL-2026-03-18-xtask.md` — added lambda-build, lambda-deploy, infra-verify
