# DRL-2026-04-03-pow-apigateway
**Date:** 2026-04-03 | **Domain:** W-CTF, W-AUTH.POST-FIX, ADR-003
**Severity:** Architectural change (ADR-009 accepted)

## Summary
POST /api/contact now routes through an API Gateway HTTP API instead of the
Lambda Function URL, and includes PoW challenge-response bot protection.

## Entries

### DRL-FUA-5 (RESOLVED): GET+query params exposes PII
**Was:** `GET /api/contact?name=...&email=...&message=...` — PII visible in URLs, CloudFront access logs, browser history.
**Is now:** `POST /api/contact` with JSON body. CloudFront routes this path to API Gateway origin (no OAC), so body hash issue is bypassed.
**How fixed:** `infra/apigateway.tf` HTTP API HTTP V2 + Lambda proxy integration. `infra/cdn.tf` ordered_cache_behavior for `/api/contact` → apigw-contact origin (no OAC). See ADR-009.

### DRL-FUA-6 (OPEN): POW_SECRET in Lambda env var is not ideal
**Problem:** `POW_SECRET` is set as a Lambda environment variable via `infra/lambda.tf` + `infra/terraform.tfvars`. Value is visible in Lambda console and stored as plaintext env var.
**Plan:** Migrate to AWS Secrets Manager (W-SEC, W-CTF.4.11). Lambda will call SM at cold start. `infra/terraform.tfvars` `pow_secret` entry will be removed.
**Current state:** Random 64-char hex value; gitignored `terraform.tfvars`; acceptable interim.

### DRL-CTF-3 (RESOLVED): ADR-003 partially superseded
**Was:** ADR-003 stated "no API Gateway".
**Is now:** API Gateway is used for exactly one route (`POST /api/contact`). All other routes still use Lambda Function URL with OAC.
**Recorded as:** ADR-009 (`plans/adr/ADR-009-api-gateway-pow-post.md`).

### DRL-CTF-4 (RESOLVED): W-AUTH.POST-FIX partially addressed
**Was:** POST/PUT through CloudFront OAC blocked. Dashboard edit forms broken.
**Is now:** POST /api/contact fixed via API Gateway (ADR-009). Dashboard PUT/PATCH still broken (different routes, lower priority).
**Status:** Contact form resolved; dashboard edit forms deferred.
