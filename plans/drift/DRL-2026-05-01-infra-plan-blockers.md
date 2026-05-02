# DRL-2026-05-01-infra-plan-blockers

**Date:** 2026-05-01 | **Status:** RESOLVED | **Affects:** W-OTF, W-WEB, W-RAG

## Summary

Three pre-existing HCL bugs blocked `just infra-plan` from producing a clean plan output.
All fixed in the same session as the `dev.sislam.com` domain fix.

---

## Entry 1 — Duplicate `aws_caller_identity` data source

**File:** `infra/s3-spa.tf:1` vs `infra/s3.tf:57`

`s3-spa.tf` (added in Phase D.4) declared `data "aws_caller_identity" "current" {}` which
already existed in `s3.tf`. OpenTofu rejects duplicate resource names.

**Fix:** Removed the declaration from `s3-spa.tf:1`; `s3.tf` remains the single declaration.

---

## Entry 2 — Duplicate `file_system_config` block in Lambda

**File:** `infra/lambda.tf:49-52`

Phase D.4 added a second `file_system_config` block for the SPA EFS access point (`/mnt/spa`).
The AWS provider schema (v5.100.0) enforces `MaxItems: 1` for this block.

Confirmed via `aws lambda get-function-configuration` that the deployed Lambda
(`deploy-baba-prod`) only has the `/mnt/db` EFS mount — the second block was never applied.
The SPA is served via S3 CloudFront origin, not from EFS at runtime.

**Fix:** Removed the second `file_system_config` block. HCL now matches actual AWS state.

---

## Entry 3 — Missing `filter {}` in S3 lifecycle rule

**File:** `infra/s3-spa.tf:32` (the `aws_s3_bucket_lifecycle_configuration.spa` rule)

AWS provider v5.x requires an explicit `filter {}` block (even if empty) on lifecycle rules.
Without it the provider emits a deprecation warning and will error in a future release.

**Fix:** Added `filter {}` to the `prune-old-sha-prefixes` rule.

---

## Context

These three bugs were introduced when SPA infrastructure was added (Phase D.4, commit `7b20f3f`).
The plan had never been run cleanly since that point; `just infra-apply` was deferred pending
the Lambda binary being ready. The bugs surfaced when running `just infra-plan` to validate
the `dev.sislam.com` domain + ACM wildcard cert changes.
