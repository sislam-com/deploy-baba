# Architecture

Last updated: 2026-05-19

deploy-baba is a full-stack portfolio platform running on AWS Lambda with near-zero monthly cost. A React SPA talks to four Rust Lambda functions backed by SQLite on EFS, all provisioned via OpenTofu and deployed through GitHub Actions with OIDC.

```
┌─────────────────────────────────────────────────────────────────┐
│  Browser                                                        │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│  CloudFront CDN                                                 │
│  S3 OAC for SPA assets  ·  Lambda origin for /api/*             │
└─────┬───────────────────────────────────┬───────────────────────┘
      │                                   │
      ▼                                   ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────────┐  ┌───────────────┐
│ services/ui   │  │ services/     │  │ services/         │  │ services/     │
│ Main Lambda   │──│ email         │  │ llm-proxy       │  │ auth          │
│ (Axum+SQLite) │  │ (SES Lambda)  │  │ (Anthropic proxy) │  │ (Cognito IDP) │
│ VPC + EFS     │  │ no VPC        │  │ no VPC            │  │ no VPC        │
└──────┬────────┘  └───────────────┘  └───────────────────┘  └───────────────┘
       │
┌──────▼────────────────────────────────────────────────────────┐
│  SQLite on EFS  (28 migrations, WAL mode)  +  S3 backup         │
└───────────────────────────────────────────────────────────────┘
       │
┌──────▼────────────────────────────────────────────────────────┐
│  Library Crates (16)                                          │
│  config-* (4) · api-* (5) · llm-* (3) · rag-* (3) · infra-1 │
└───────────────────────────────────────────────────────────────┘
```

## Crate Layers

The workspace contains 16 library crates organized in five layers. Each crate solves one problem through trait-based composition with monomorphization (no `dyn` dispatch). See [crate-guide.md](crate-guide.md) for per-crate API documentation.

### Config Layer (4 crates)

Format-agnostic configuration parsing via `ConfigParser<T>` and `ConfigValidator<T>` traits.

| Crate | Role |
|-------|------|
| `config-core` | Trait definitions |
| `config-toml` | TOML implementation |
| `config-yaml` | YAML implementation |
| `config-json` | JSON implementation |

### API Spec Layer (5 crates)

Generate API specifications in multiple wire formats from Rust type definitions. The `api-openapi` crate is the SSOT for all API models ([ADR-012](../plans/adr/ADR-012-openapi-ssot.md)).

| Crate | Role |
|-------|------|
| `api-core` | `ApiSpecGenerator` trait |
| `api-openapi` | OpenAPI 3.0 generator + model registry (29 models) |
| `api-graphql` | GraphQL SDL generator |
| `api-grpc` | Protocol Buffers generator |
| `api-merger` | Multi-format merging with conflict resolution |

### LLM Layer (3 crates)

Vendor-agnostic LLM provider abstraction with a grounding contract and agentic tool-dispatch loop ([ADR-015](../plans/adr/ADR-015-llm-provider-abstraction-and-grounding-contract.md)).

| Crate | Role |
|-------|------|
| `llm-core` | `LlmProvider` trait, `ChatMessage` types, `run_agent_loop()`, grounding contract |
| `llm-anthropic` | Anthropic Claude adapter (Messages API, tool_use) |
| `llm-openai` | OpenAI adapter (WIP) |

### RAG Layer (3 crates)

Retrieval-augmented generation with SQLite FTS5 for hybrid keyword + full-text search across 7 portfolio corpora ([ADR-016](../plans/adr/ADR-016-rag-architecture.md)).

| Crate | Role |
|-------|------|
| `rag-core` | `Retriever` trait, `Embedder` trait, `PromptAssembler`, document chunkers |
| `rag-sqlite` | `RagStore` — SQLite FTS5 backend implementing `Retriever` |
| `portfolio-rag-mcp` | MCP server binary wrapping rag-sqlite for Claude Code |

### Infrastructure Layer (1 crate)

| Crate | Role |
|-------|------|
| `infra-types` | Cloud-agnostic `Stack`, `DeployConfig`, `SqliteConfig`, `ObservabilityConfig` |

## Services

Three Lambda functions, each purpose-built for its workload. See [services.md](services.md) for detailed documentation.

| Service | Runtime | VPC | Purpose |
|---------|---------|-----|---------|
| `services/ui` | Axum + lambda_http | Yes (EFS) | Main API + portfolio site backend |
| `services/auth` | Axum + lambda_http | No | Cognito IDP proxy — custom login flow for SPA |
| `services/email` | lambda_runtime | No | SES email delivery |
| `services/llm-proxy` | lambda_runtime | No | LLM routing + tool dispatch |

The UI Lambda is the only service that needs VPC access (for EFS-mounted SQLite). Auth, email, and LLM proxy run outside the VPC for direct internet access to Cognito, SES, and Anthropic APIs respectively.

## Frontend

React 18 SPA built with Vite and TypeScript, styled with Tailwind CSS. The SPA replaced server-side Askama templates ([ADR-019](../plans/adr/ADR-019-spa-replaces-askama.md)). See [web-spa.md](web-spa.md) for route map and build details.

- 7 marketing routes (Home, About, Ask, Contact, NotFound)
- 11 dashboard routes (CRUD for jobs, competencies, about, social links, challenges)
- Type-safe API client generated from OpenAPI spec via `openapi-fetch`
- Cognito authentication for dashboard access

## Infrastructure

21 OpenTofu HCL files in `infra/` provisioning 100+ AWS resources. See [aws-setup.md](aws-setup.md) for setup instructions.

Key resources: 4 Lambda functions, EFS file system + mount targets, 3 S3 buckets (SPA, assets, state), CloudFront distribution with OAC, Cognito user pool, SES email identity, API Gateway HTTP API (POST /api/contact only), VPC endpoints, EventBridge backup schedule, Secrets Manager, SSM Parameter Store, IAM roles + OIDC for CI/CD.

## Data Layer

SQLite on EFS with 28 numbered migrations, WAL mode for concurrent reads. See [database.md](database.md) for schema overview and migration system.

Key decisions: SQLite over PostgreSQL for zero cost ([ADR-002](../plans/adr/ADR-002-sqlite-over-postgresql.md)), upsert convention for idempotent seeding ([ADR-010](../plans/adr/ADR-010-sqlite-upsert-seed-convention.md)).

## CI/CD

GitHub Actions with OIDC-based AWS deployment — no long-lived access keys. See [ci-cd.md](ci-cd.md) for pipeline details.

| Workflow | Trigger | What it does |
|----------|---------|-------------|
| `ci.yml` | Push/PR to main | 5-job quality gate (fmt, test, docs, tofu, web) |
| `deploy-dev.yml` | CI success on main | Build + deploy to dev.sislam.com |
| `deploy-prod.yml` | Tag push (`v*`) | Deploy to sislam.com (production gate) |

## Design Principles

1. **Zero-cost abstractions** — generics + monomorphization, not `dyn` dispatch. See [zero-cost-philosophy.md](zero-cost-philosophy.md).
2. **Trait composition** — each crate defines traits; implementations are separate crates.
3. **thiserror everywhere** — structured errors with context, no `anyhow` in library crates.
4. **justfile is the interface** — developers never invoke xtask directly ([ADR-001](../plans/adr/ADR-001-justfile-only-interface.md)).
5. **SQLite over PostgreSQL** — zero monthly cost, no RDS, no connection pooling ([ADR-002](../plans/adr/ADR-002-sqlite-over-postgresql.md)).
6. **OpenTofu over Terraform** — MPL-2.0, drop-in HCL compatible ([ADR-007](../plans/adr/ADR-007-opentofu-over-terraform.md)).

## Cross-References

**Architecture decisions:** [plans/adr/](../plans/adr/) (27 ADRs)

**Project status:** [plans/INDEX.md](../plans/INDEX.md) (~95% complete)

**Per-crate API docs:** [crate-guide.md](crate-guide.md)
