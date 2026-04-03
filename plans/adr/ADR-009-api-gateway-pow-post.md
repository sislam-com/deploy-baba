# ADR-009: API Gateway HTTP API for POST /api/contact

**Status:** Accepted
**Date:** 2026-04-03
**Affected modules:** W-CTF, W-OTF, W-UI

---

## Context

ADR-003 established "Lambda Function URL, no API Gateway" as the primary deployment
pattern. However, CloudFront OAC + AWS_IAM Lambda Function URL rejects POST request
bodies because CloudFront sends `UNSIGNED-PAYLOAD` instead of the actual SHA-256 body
hash, causing `InvalidSignatureException` (DRL-FUA-3).

This means all POST/PUT/DELETE routes must either:
1. Accept body-less semantics (e.g., query params — exposes PII in URLs/logs), or
2. Bypass the OAC path for those routes

## Decision

Add a **minimal API Gateway HTTP API V2** (`infra/apigateway.tf`) as a second
CloudFront origin exclusively for `POST /api/contact`. The API Gateway:
- Proxies to the same UI Lambda function (payload format 2.0)
- Does **not** use OAC signing — POST bodies reach Lambda correctly
- Is NOT exposed directly (no DNS record); only reachable via CloudFront
- Covers only one route: `POST /api/contact`

All other routes continue to use the Lambda Function URL origin with OAC.

## Consequences

**Good:**
- POST bodies work correctly end-to-end
- No PII in URLs, CloudFront logs, or browser history
- Proof-of-work bot protection becomes viable (requires POST JSON body)
- API Gateway HTTP API cost: effectively $0 for portfolio traffic

**Bad:**
- ADR-003 is no longer strictly true — API Gateway is now used for one route
- Two different origin types in CloudFront (Lambda Function URL + API Gateway)
- API Gateway adds another resource to manage

## Notes

- ADR-003 stands for all other routes; this is a targeted exception for POST contact
- If POST needs expand to other routes (dashboard edits), extend this API or reconsider
  a full API Gateway migration (W-AUTH.POST-FIX originally planned Option C)
- PoW challenges (`GET /api/contact/challenge`) still go via Lambda Function URL + OAC
