# deploy-baba

**Full-stack portfolio platform on AWS Lambda — near-zero cost.**

A Rust + React application powering [sislam.com](https://sislam.com): interactive resume, RAG-powered Q&A, admin dashboard, and contact form — all running on a single Lambda function with SQLite on EFS.

Built on a composable crate ecosystem using trait-based composition with monomorphization, not dynamic dispatch.

```
┌─────────────────────────────────────────────────────────────────┐
│  React SPA (Vite + TypeScript)                                  │
│  Home / About / Ask / Contact / Dashboard                       │
├─────────────────────────────────────────────────────────────────┤
│  CloudFront CDN (S3 OAC for SPA, Lambda origin for /api)        │
├─────────────────────────────────────────────────────────────────┤
│  services/ui        │  services/email   │  services/llm-proxy   │
│  (main Lambda)      │  (SES Lambda)     │  (Anthropic proxy)    │
├─────────────────────┴───────────────────┴───────────────────────┤
│  SQLite on EFS (28 migrations) + S3 backup                      │
├─────────────────────────────────────────────────────────────────┤
│  Library Crates                                                  │
│  config-* │ api-* │ infra-types │ llm-* │ rag-* │ mcp          │
└─────────────────────────────────────────────────────────────────┘
```

## Features

- **Interactive resume** — timeline and capabilities views, PDF/DOCX downloads, generated from DB
- **RAG-powered Q&A** — ask questions about the portfolio; answers grounded in indexed codebase content
- **Admin dashboard** — authenticated CRUD for jobs, about sections, social links, challenges
- **Contact form** — proof-of-work spam protection, SES email delivery via dedicated Lambda
- **OpenAPI spec** — auto-generated dual spec (public + admin), served at `/docs`
- **Resume generation** — DB → Markdown → DOCX/PDF pipeline via xtask

## Project Structure

```
deploy-baba/
├── crates/            # 15 library crates (pure Rust, no binaries)
├── services/
│   ├── ui/            # Main Lambda: Axum API + SQLite + auth
│   ├── email/         # SES email Lambda (non-VPC)
│   └── llm-proxy/     # Anthropic API proxy Lambda (non-VPC)
├── web/               # React SPA (Vite + TypeScript)
├── infra/             # OpenTofu HCL (Lambda + EFS + S3 + CloudFront + Cognito)
├── xtask/             # Internal CLI (resume gen, deploy, cache, secrets)
├── examples/          # 4 runnable examples demonstrating crate usage
├── plans/             # Modular plan system (35 modules, 27 ADRs)
└── justfile           # Developer interface — all commands go through just
```

## Crate Map

| Crate | Purpose |
|-------|---------|
| **Config Layer** | |
| `config-core` | Universal traits: `ConfigParser<T>`, `ConfigValidator<T>` |
| `config-toml` | TOML implementation |
| `config-yaml` | YAML implementation |
| `config-json` | JSON implementation |
| **API Spec Layer** | |
| `api-core` | Universal traits: `ApiSpecGenerator` |
| `api-openapi` | OpenAPI 3.0 generator + model registry (SSOT for all API types) |
| `api-graphql` | GraphQL SDL generator |
| `api-grpc` | Protocol Buffers / gRPC generator |
| `api-merger` | Multi-format spec merging with conflict resolution |
| **LLM Layer** | |
| `llm-core` | Vendor-agnostic LLM provider traits and grounding contract |
| `llm-anthropic` | Anthropic Claude adapter |
| **RAG Layer** | |
| `rag-core` | Retrieval traits and document chunkers |
| `rag-sqlite` | SQLite FTS5 retrieval backend |
| `portfolio-rag-mcp` | MCP server for RAG integration |
| **Infrastructure** | |
| `infra-types` | Cloud-agnostic Stack, Service, Network, SQLite + S3 types |

## Development

All commands go through the [justfile](justfile). Run `just` to see everything.

```bash
just dev            # format + lint + test
just dev-stack      # Vite on :3000 + Rust API on :3001 (hot reload)
just quality        # full quality gate (fmt + clippy + test + audit)
just dev-doctor     # verify all prerequisites are installed
```

## Frontend

The SPA lives in `web/` — React + TypeScript, built with Vite.

```bash
just web            # start Vite dev server on :3000
just web-build      # production build to web/dist/
just web-types-offline  # regenerate TypeScript types from OpenAPI spec
just web-test       # run Vitest unit tests
```

## Examples

Runnable examples are in [`examples/`](examples/). Each is a standalone package
in the workspace.

```bash
just example 01_multi_format_config   # Parse the same config as TOML, YAML, and JSON
just example 02_api_spec_generation   # Generate OpenAPI, GraphQL, and Protobuf specs
just example 03_spec_merger           # Merge multiple specs with conflict resolution
just example 04_infra_types           # Build and serialize a Stack definition
```

## Deploy to AWS

The platform runs on AWS Lambda with near-zero cost. Infrastructure is managed with OpenTofu.

```bash
just sso-login                   # authenticate via AWS SSO
just infra-plan deploy-baba      # preview infrastructure changes
just infra-apply deploy-baba     # provision infrastructure
just deploy deploy-baba          # quality gate + Lambda build + update
just resume deploy-baba          # generate + upload resume files to S3
```

See [docs/aws-setup.md](docs/aws-setup.md) for full setup instructions.

## Architecture

Key decisions are documented as ADRs in [`plans/adr/`](plans/adr/):

- **ADR-001**: justfile-only interface (xtask is internal plumbing)
- **ADR-002**: SQLite on EFS + S3 backup (no PostgreSQL, no RDS)
- **ADR-003**: Lambda Function URL (no API Gateway, except ADR-009)
- **ADR-007**: OpenTofu over Terraform
- **ADR-008**: Cognito hosted UI auth with JWT RS256
- **ADR-012**: OpenAPI SSOT — all API types defined in `api-openapi`
- **ADR-015**: LLM provider abstraction with grounding contract

See [plans/INDEX.md](plans/INDEX.md) for the full project plan and module status.

### Documentation

| Doc | What it covers |
|-----|---------------|
| [Architecture](docs/architecture.md) | System overview, crate layers, services, design principles |
| [Services](docs/services.md) | All 3 Lambda functions, inter-service communication |
| [Web SPA](docs/web-spa.md) | React frontend, route map, API client, auth |
| [Database](docs/database.md) | SQLite migrations, schema, upsert convention, backup |
| [CI/CD](docs/ci-cd.md) | GitHub Actions, OIDC deployment, release tagging |
| [AWS Setup](docs/aws-setup.md) | IAM, SES, Cognito, bootstrap, deploy workflow |
| [Crate Guide](docs/crate-guide.md) | Per-crate API reference for all 16 library crates |
| [Zero-Cost Philosophy](docs/zero-cost-philosophy.md) | Why generics over `dyn`, why SQLite over RDS |
| [Skills](docs/skills.md) | Claude Code slash commands for this project |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT License](LICENSE-MIT) at your option.
