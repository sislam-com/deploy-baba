# DRL-2026-03-27-function-url-auth — Lambda Function URL Auth Incident

**Date:** 2026-03-27
**Topic:** Attempted migration from `AWS_IAM` + CloudFront OAC to `NONE` + public access on Lambda Function URL caused live site outage (~2 hrs). Reverted.

---

## Incident Summary

The live site (`sislam.com`) returned `{"Message":"Forbidden"}` for all requests. The root cause was a split-apply state: `infra/cdn.tf` had been applied at some point (removing the CloudFront OAC for the Lambda origin) while `infra/lambda.tf` still had `authorization_type = "AWS_IAM"`. CloudFront sent unsigned requests; Lambda rejected them.

An attempted fix switched the Function URL to `authorization_type = "NONE"` and added a `FunctionURLAllowPublicAccess` resource-based policy (`principal = "*"`, `lambda:FunctionUrlAuthType = "NONE"` condition). This condition **does not evaluate for anonymous requests** in this account — the site remained broken with 403 on the direct Function URL even after apply.

During diagnosis, the original Function URL was deleted and recreated via CLI (changing its ID from `7omjlnbziajd6wespfswvf5szu0znwzx` to `7zlgo4y6wezunsml6ridvwhdqe0lofnu`). This caused Terraform state drift.

---

## Resolution

1. Reverted `infra/lambda.tf` and `infra/cdn.tf` to committed state (`git checkout HEAD -- infra/`)
2. Removed stale Function URL from state: `tofu state rm aws_lambda_function_url.baba`
3. Imported current URL into state: `tofu import aws_lambda_function_url.baba deploy-baba-prod`
4. Applied with auto-approve: `cargo xtask infra apply --profile default --auto-approve`

Result: `AWS_IAM` + CloudFront OAC (`deploy-baba-lambda-oac`, id: `E1T8RKBXX8GBY9`) restored. CloudFront distribution updated to new Function URL. Site returned 200 OK.

---

## Drift Entries

### DRL-FUA-1 — Function URL ID Changed

**Before incident:** `7omjlnbziajd6wespfswvf5szu0znwzx.lambda-url.us-east-1.on.aws`
**After incident:** `7zlgo4y6wezunsml6ridvwhdqe0lofnu.lambda-url.us-east-1.on.aws`

CloudFront distribution was updated by the final apply to point to the new URL. All traffic routes correctly. TF state is aligned.

---

### DRL-FUA-2 — `authorization_type = "NONE"` + `FunctionURLAllowPublicAccess` Does NOT Work

**Finding:** Adding `aws_lambda_permission` with `principal = "*"` and `lambda:FunctionUrlAuthType = "NONE"` condition fails to grant public access in this account. The `lambda:FunctionUrlAuthType` condition key is not satisfied for anonymous requests — both VPC and non-VPC functions were affected.

**Decision:** Do not attempt this migration again without first testing in a scratch account or with a non-critical function. The current `AWS_IAM` + CloudFront OAC approach is the production-safe path.

**Known limitation:** CloudFront OAC + `AWS_IAM` breaks POST/PUT request body signing. Admin form submissions (dashboard edits) will fail via CloudFront.

**Deferred:** W-AUTH.POST-FIX — investigate alternative for POST/PUT auth:
- Option A: Use API Gateway (violates ADR-003)
- Option B: App-layer auth only (remove `AWS_IAM`, accept security trade-off — blocked by account-level `FunctionURLAllowPublicAccess` issue)
- Option C: Use Lambda function invocation URL via CloudFront custom origin without OAC body signing (requires investigation)

---

### DRL-FUA-3 — Root Cause of POST Body Hash Mismatch (2026-04-03)

**Date:** 2026-04-03

**Finding:** Definitively diagnosed the W-AUTH.POST-FIX signature error using `awscurl`:

```
# Succeeds — awscurl computes actual body hash e3b0c44... (empty body)
awscurl --service lambda -X POST [function-url] --data '{...}'  → 200

# Fails — Lambda Function URL rejects UNSIGNED-PAYLOAD for AWS_IAM auth
awscurl --service lambda -X POST [function-url] -H "x-amz-content-sha256: UNSIGNED-PAYLOAD"  → 403 InvalidSignatureException

# Also fails for GET with UNSIGNED-PAYLOAD
awscurl --service lambda -X GET [function-url] -H "x-amz-content-sha256: UNSIGNED-PAYLOAD"  → 403 InvalidSignatureException
```

**Root cause:** Lambda Function URL with `authorization_type = "AWS_IAM"` requires the actual SHA256 body hash in the SigV4 signature. CloudFront OAC always sends `x-amz-content-sha256: UNSIGNED-PAYLOAD` and never computes the real hash. For GET requests, CloudFront does compute the empty-body hash `e3b0c44...` (via `x-amz-content-sha256`), so GET works. For POST/PUT, CloudFront doesn't hash the body → mismatch → 403.

**This is a fundamental AWS architectural constraint** — there is no CloudFront configuration that makes OAC compute the actual body hash. The only solutions are:
1. Eliminate the POST body (use GET + query params for small payloads)
2. Use a non-OAC path for mutations (Lambda invoke via SDK, bypassing CloudFront entirely)
3. Use API Gateway instead of Function URL (violates ADR-003)

### DRL-FUA-4 — Contact Form: GET-based workaround for W-AUTH.POST-FIX (2026-04-03)

**Date:** 2026-04-03

**Applied to:** `GET /api/contact` (contact form submission)

**Approach:** Changed contact form from POST+JSON body to GET+query params. Since GET has no body, CloudFront computes the empty-body hash correctly and Lambda accepts the request.

**Changes made:**
- `services/ui/src/routes/contact.rs`: extractor changed from `Json<ContactSubmitRequest>` → `Query<ContactSubmitRequest>`
- `services/ui/src/router.rs`: route changed from `post(contact_submit)` → `get(contact_submit)`
- `services/ui/templates/contact.html`: JS fetch changed from `method: 'POST'` + JSON body → `method: 'GET'` + `URLSearchParams`

**Security note:** Form fields (name, email, subject, message) appear in URL and are logged in CloudFront access logs and Lambda logs. Acceptable for a public contact form on a portfolio site.

**Rate limiting and honeypot still work:** `x-forwarded-for` header is always forwarded by CloudFront (regardless of origin request policy). The `website` honeypot field is passed as a query param and checked server-side.

**SES v2 IAM — TO identity check:** SES v2 also checks IAM authorization on the exact destination identity ARN (e.g., `identity/contact-sislam@shantopagla.com`). The domain resource `identity/shantopagla.com` does NOT cover individual address checks. Use a wildcard: `arn:aws:ses:...:identity/*@shantopagla.com`.

**Custom origin request policy (`deploy-baba-lambda-oac-policy`):** `headers_config.header_behavior = "none"` — forwards no viewer headers to origin. This minimizes the signed header set in OAC SigV4 signatures. Cookies are forwarded separately (`cookie_behavior = "all"`) so session auth works. `x-forwarded-for` always added by CloudFront.

**Dashboard mutations still broken (W-AUTH.POST-FIX):** Admin PUT/POST (dashboard edit forms) still fail via CloudFront OAC. Deferred — requires architectural solution (Option B or C from above). GET-based workaround is not viable for large payloads.

---

## Open Items

| ID | Description | Priority |
|----|-------------|----------|
| W-AUTH.POST-FIX | Dashboard PUT/POST fail through CloudFront OAC — dashboard edit forms broken on live site | P1 |
| DRL-FUA-2 | Understand why `lambda:FunctionUrlAuthType=NONE` fails for anon requests | P2 |
