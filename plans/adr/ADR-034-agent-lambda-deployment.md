# ADR-034: Agent Lambda Deployment Pattern

**Date:** 2026-05-24
**Status:** Accepted
**Affected modules:** W-AGT, W-OTF, W-CI, W-DX

## Context

The cover letter agent (ADR-033) runs as a LangGraph Python application. It needs to be deployed alongside the existing Rust Lambda services. Portfolio already has a proven multi-Lambda pattern (ADR-031) with email, auth, llm-proxy, and mcp-gateway services.

Key constraints:
- Must follow zero-cost philosophy (no always-on compute)
- Python runtime (not Rust) — different build toolchain
- No direct DB access needed (data retrieved via Lambda invoke to Rust services)
- Must integrate with existing service-protocol routing from UI Lambda

## Decision

> The LangGraph agent deploys as a standalone Python Lambda function, invoked by the UI Lambda via service-protocol (ADR-031). It has no VPC attachment and no EFS mount.

### Lambda specification

| Property | Value |
|----------|-------|
| Name | `deploy-baba-{env}-agent` |
| Runtime | Python 3.13 on arm64 |
| Handler | Mangum (FastAPI → Lambda adapter) |
| Memory | 512 MB |
| Timeout | 120 seconds (cover letter generation is multi-step) |
| VPC | None (calls other Lambdas via SDK, Anthropic API directly) |
| EFS | None (no direct DB access) |
| Layers | None (deps bundled via uv + zip) |

### IAM permissions

- `lambda:InvokeFunction` on `deploy-baba-{env}-ui` (to retrieve resume data, run matcher)
- `secretsmanager:GetSecretValue` on `/{project}/{env}/api-keys` (Anthropic API key)
- `s3:PutObject` on `{bucket}/cover-letters/*` (upload generated cover letters)
- `s3:GetObject` on `{bucket}/cover-letters/*` (generate presigned URLs)
- `logs:*` on own CloudWatch log group

### Build and deploy

```
just agent-build    # uv export → pip install → zip
just agent-deploy   # build + aws lambda update-function-code
```

Build process:
1. `uv export --frozen > requirements.txt` in `services/agent/`
2. `pip install -t build/agent-lambda/ -r requirements.txt`
3. Copy `services/agent/src/` into `build/agent-lambda/`
4. `zip -r infra/build/agent-lambda.zip build/agent-lambda/`

### S3 artifact storage

Cover letters stored at `s3://{bucket}/cover-letters/{date}/{hash}.{html,pdf}`.
Lifecycle rule: 30-day expiration on the `cover-letters/` prefix.

### OpenTofu

New file: `infra/agent-lambda.tf` containing:
- `aws_lambda_function.agent`
- `aws_iam_role.agent_execution`
- `aws_iam_role_policy.agent_permissions`
- `aws_cloudwatch_log_group.agent`

Modify `infra/iam.tf`: add `lambda:InvokeFunction` for agent Lambda to UI Lambda's invoke policy.

## Consequences

- First Python Lambda in the portfolio project — establishes the pattern for future Python services
- Separate build toolchain (`uv` + `pip` instead of `cargo-lambda`)
- Cold start will be slower than Rust Lambdas (~2-3s vs ~50ms) — acceptable for cover letter generation which is already multi-second
- CI needs a Python job for `just agent-test` and `just agent-build`
