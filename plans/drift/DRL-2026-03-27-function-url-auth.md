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

**Known limitation:** CloudFront OAC + `AllViewerExceptHostHeader` + `AWS_IAM` breaks POST/PUT request body signing (chunked transfer hash mismatch). Admin form submissions (dashboard edits) will fail via CloudFront.

**Deferred:** W-AUTH.POST-FIX — investigate alternative for POST/PUT auth:
- Option A: Use API Gateway (violates ADR-003)
- Option B: App-layer auth only (remove `AWS_IAM`, accept security trade-off — blocked by account-level `FunctionURLAllowPublicAccess` issue)
- Option C: Use Lambda function invocation URL via CloudFront custom origin without OAC body signing (requires investigation)

---

## Open Items

| ID | Description | Priority |
|----|-------------|----------|
| W-AUTH.POST-FIX | POST/PUT requests fail through CloudFront OAC due to body hash mismatch | P1 |
| DRL-FUA-2 | Understand why `lambda:FunctionUrlAuthType=NONE` fails for anon requests | P2 |
