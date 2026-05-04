# DRL-2026-05-04-adr009-ask-api-scope

**Date:** 2026-05-04
**Topic:** ADR-009 scope expanded beyond POST /api/contact
**Status:** Open

---

## Entries

### DRL-ASK-1: POST /api/ask added to API Gateway

**Observation:** ADR-009 states the API Gateway HTTP API covers "only one route: `POST /api/contact`". Commit `29562df` added `POST /api/ask` to the same API Gateway and CloudFront behavior, expanding the scope to two routes. The ADR text, title, and "Covers only one route" claim are now inaccurate.

**Resolution:** ADR-009 text should be updated to reflect the expanded scope (contact + ask). The decision rationale (OAC body hash mismatch for POST routes) applies equally to `/api/ask`.

### DRL-ASK-2: Rate limiting not enforced in production

**Observation:** The ask API has a 2-requests-per-limit design but this limit is not currently enforced in the production deployment.

**Resolution:** TODO — implement rate limiting for `POST /api/ask` in production.

---

## Open Items

| ID | Description | Blocking |
|----|-------------|---------|
| DRL-ASK-1 | Update ADR-009 text to acknowledge `/api/ask` as second API Gateway route | ADR accuracy |
| DRL-ASK-2 | Enforce 2-request rate limit for `/api/ask` in production | Security hardening |
