# W-MCP: Private MCP Gateway
**Crate(s):** `crates/mcp-rs/`, `services/mcp-gateway/` | **Status:** WIP
**Coverage floor:** 70% | **Depends on:** W-AIL, W-OTF, W-AUTH | **Depended on by:** local/cloud agent workflows

## W-MCP.1 Purpose

Provide a private MCP layer for the portfolio monorepo. Local agents use `crates/mcp-rs` over stdio;
cloud agents use a Cognito-protected Lambda gateway that exposes read-only project context.

## W-MCP.2 Public Surface

- `crates/mcp-rs` — reusable MCP server crate and local binary.
- `.mcp-rs.toml` — repo-local read-only MCP configuration.
- `services/mcp-gateway` — Lambda HTTP adapter for `POST /mcp` and `GET /mcp/health`.
- `just mcp-smoke` — local MCP regression check.
- `just mcp-cloud-build` — builds the Lambda package with static context.
- `just mcp-cloud-deploy PROFILE` — uploads the gateway Lambda.
- `just mcp-cloud-smoke PROFILE` — validates auth and read-only cloud behavior.

## W-MCP.3 Implementation Notes

The gateway validates Cognito ID tokens using the existing deploy-time JWKS pattern, then delegates
JSON-RPC handling to `mcp-rs` in-process. The Lambda package includes:

- `bootstrap`
- `/var/task/mcp-rs.toml`
- `/var/task/mcp-context/` containing selected project context files

Read-only mode is enforced in `mcp-rs` policy by `read_only`, `enabled_tools`, and `disabled_tools`.
The cloud config disables command execution, env reads, cargo operations, and live SQLite queries.

Zero-cost constraints:

- Reuse the existing CloudFront distribution and HTTP API; add routes only.
- Do not add NAT Gateway, VPC config, VPC endpoint, custom domain, certificate, provisioned
  concurrency, CloudWatch metric alarm, queue, database, or always-on service for MCP.
- Keep observability to normal Lambda/API Gateway logs with existing retention.
- Use reserved concurrency only as a free blast-radius limiter.

## W-MCP.4 Work Items

| ID | Task | Status | Notes |
|---|---|---|---|
| W-MCP.4.1 | Import `mcp-rs` into workspace as `crates/mcp-rs` | DONE | Monorepo ownership; no sibling Cargo path |
| W-MCP.4.2 | Add config precedence and read-only policy controls | DONE | `--config`, `MCP_RS_CONFIG`, tool allow/deny lists |
| W-MCP.4.3 | Add private gateway service | DONE | `services/mcp-gateway` validates Cognito token |
| W-MCP.4.4 | Add context bundle build recipe | DONE | `just mcp-context-build` |
| W-MCP.4.5 | Add Lambda/API Gateway/CloudFront infra | DONE | Zero fixed-cost resources only; apply with `just infra-plan`/`just infra-apply` |
| W-MCP.4.6 | Add cloud smoke test | DONE | `just mcp-cloud-smoke PROFILE`; requires `MCP_BEARER_TOKEN` env var |
| W-MCP.4.7 | Evaluate OAuth/PKCE MCP compatibility | DEFERRED | Future standards compatibility phase |

## W-MCP.5 Test Strategy

1. `cargo test -p mcp-rs`
2. `cargo check -p mcp-gateway`
3. `just mcp-cloud-build`
4. `just infra-plan PROFILE`
5. `just mcp-cloud-smoke PROFILE` with `MCP_BEARER_TOKEN`

Security smoke cases:

- unauthenticated `POST /mcp` returns 401
- valid token can read `project://plans`
- valid token cannot call `read_env`, `just_run`, cargo tools, or SQLite query tools

## W-MCP.6 Cross-References

- → ADR-028 (Private Cloud MCP Gateway)
- → ADR-008 (Cognito Authentication)
- → ADR-017 (AI-DLC)
- → `AGENTS.md` local MCP startup protocol
