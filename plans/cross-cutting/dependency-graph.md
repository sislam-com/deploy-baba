# Dependency Graph — deploy-baba

---

## Internal Crate Dependency Order

Build and publish must follow this order (no circular deps):

```
Layer 0 (no internal deps):
  config-core
  api-core

Layer 1 (depends on layer 0):
  config-toml    → config-core
  config-yaml    → config-core
  config-json    → config-core
  api-openapi    → api-core
  api-graphql    → api-core
  api-grpc       → api-core
  infra-types    → config-core (optional feature), serde

Layer 2 (depends on layers 0–1):
  api-merger     → api-core, api-openapi, api-graphql, api-grpc

Binary (not published):
  services/ui    → config-core, config-toml, config-yaml, config-json,
                   api-core, api-openapi, api-merger, infra-types
  xtask          → (no internal crate deps — uses std::process::Command)
```

---

## Crate Inventory

| Crate | Source (~/shanto) | Work Done | Remaining |
|-------|-------------------|-----------|-----------|
| `config-core` | `crates/rust-config-core` | Renamed, `ConfigSource` added, docs polished | Per-crate README |
| `config-toml` | `crates/rust-config-toml` | Renamed, minor cleanup | Per-crate README |
| `config-yaml` | `crates/rust-config-yaml` | Stub completed (~200 lines) | Per-crate README |
| `config-json` | `crates/rust-config-json` | Stub completed (~200 lines) | Per-crate README |
| `api-core` | `crates/rust-api-core` | Internal fields removed, AsyncAPI stub | Per-crate README |
| `api-openapi` | `crates/rust-api-openapi` | Extracted, polished | Per-crate README |
| `api-graphql` | `crates/rust-api-graphql` | Stub completed (~250 lines) | Per-crate README |
| `api-grpc` | `crates/rust-api-grpc` | Stub completed (~250 lines) | Per-crate README |
| `api-merger` | `crates/rust-api-merger` | Extra strategies, internals renamed | Per-crate README |
| `infra-types` | `services/baba-stack` | Sanitized, DB replaced with SqliteConfig | Per-crate README |

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
lambda_http   = "0.13"          # Lambda ↔ Axum adapter
tower-http    = { version = "0.5", features = ["cors", "trace"] }
# Askama removed by ADR-019 — UI now rendered by React SPA in web/
utoipa-rapidoc = "4"            # RapiDoc at /docs
anyhow        = "1"             # OK in binary (not library)
tracing       = "0.1"
tracing-subscriber = "0.3"

# ── xtask (internal tooling, not published) ───────────────────────────────────
aws-config    = { version = "1", features = ["behavior-version-latest"] }
aws-sdk-sts   = "1"
aws-sdk-ssm   = "1"
aws-sdk-s3    = "1"
aws-sdk-ecr   = "1"             # ECR Public push
aws-sdk-lambda = "1"            # Lambda update-function-code
aws-sdk-efs   = "1"             # EFS describe (for db-backup target path)
aws-sdk-ecs   = "1"             # Fargate Spot option
aws-sdk-dynamodb = "1"          # Terraform state lock table
clap          = { version = "4", features = ["derive"] }
tokio         = { version = "1", features = ["full"] }
flate2        = "1"             # gzip for SQLite backups
```

---

## External CLI Tools (not Cargo deps)

| Tool | Install | Used by |
|------|---------|---------|
| `cargo-lambda` | `cargo install cargo-lambda` | `just lambda-build`, `just deploy` |
| `tofu` | `brew install opentofu` | `just infra-*` |
| `cargo-watch` | `cargo install cargo-watch` | `just ui` (hot reload) |
| `cargo-audit` | `cargo install cargo-audit` | `just audit`, `just quality` |
| `cargo-llvm-cov` | `cargo install cargo-llvm-cov` | `just coverage` |

---

## Cross-References
- → `plans/modules/` — per-crate implementation details
- → `plans/cross-cutting/quality-gates.md` — coverage floors per crate
- → `plans/cross-cutting/publishing.md` — publish order
