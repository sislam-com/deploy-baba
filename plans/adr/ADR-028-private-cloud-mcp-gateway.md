# ADR-028: Private Cloud MCP Gateway

**Date:** 2026-05-22
**Status:** Proposed
**Affected modules:** W-MCP, W-AIL, W-RAG, W-OTF, W-CI

## Context

The portfolio already uses local MCP servers for agent context, but the setup was split between a
separate `mcp-rs` checkout and portfolio-local configuration. That made cloud deployment awkward:
the portfolio would need to package a sibling binary or depend on a machine-specific Cargo path.

Cloud-hosted MCP also crosses a real security boundary. The first deployable version must be private,
owner-only, read-only, and compatible with the portfolio's zero-cost AWS architecture.

## Decision

The portfolio becomes the monorepo owner for MCP deployment. `mcp-rs` lives as `crates/mcp-rs` in the
workspace and exposes both a library API and a local stdio binary. The cloud gateway lives in
`services/mcp-gateway` and depends on the workspace crate directly.

The cloud endpoint is private Cognito-authenticated HTTP:

- `POST /mcp` accepts JSON-RPC MCP requests.
- `GET /mcp/health` returns a gateway health response.
- Every `POST /mcp` request requires a valid Cognito ID token.
- The gateway loads `/var/task/mcp-rs.toml` and serves a bundled read-only context directory.

Cloud v1 is read-only. It exposes resources and safe file/search/query tools over the bundled context
only. It blocks environment reads, command execution, cargo operations, live SQLite access, and
mutation-oriented workflows.

The stack preserves ADR-005's zero-cost posture:

- no NAT Gateway, VPC attachment, VPC endpoint, provisioned concurrency, or always-on compute
- no new hosted zone, certificate, custom domain, or Cognito pool
- no new CloudWatch metric alarm; logs use existing retention defaults
- the gateway reuses the existing CloudFront distribution and HTTP API origin
- Lambda concurrency is capped with reserved concurrency, which limits exposure without adding cost

Full MCP OAuth/PKCE and broader Streamable HTTP compatibility are deferred. The private Cognito
gateway is the production baseline.

## Consequences

### Positive

- No sibling checkout, machine-specific Cargo path, or runtime subprocess wrapper.
- Portfolio owns its deployable MCP surface and context bundle.
- `mcp-rs` remains reusable inside the monorepo as both a crate and a binary.
- Security defaults are explicit and testable.

### Negative

- `mcp-rs` is no longer independently versioned from the portfolio unless later split back out.
- The workspace lockfile now owns MCP dependencies.
- Public third-party MCP clients may need a later OAuth compatibility phase.
- The shared HTTP API still has per-request pricing after free-tier limits, matching the existing
  `/api/contact` and `/api/ask` architecture rather than introducing a new fixed-cost service.

## Cross-References

- → ADR-005 (Zero-Cost Philosophy)
- → ADR-008 (Cognito Authentication for Admin Dashboard)
- → ADR-017 (AI-Assisted Development Lifecycle)
- → ADR-023 (Agentic Tool-Dispatch Architecture)
- → `plans/modules/mcp-cloud.md`
