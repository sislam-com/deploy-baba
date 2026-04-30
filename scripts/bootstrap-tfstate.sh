#!/usr/bin/env bash
# Bootstrap OpenTofu state backend for deploy-baba.
# Idempotent — safe to run multiple times.
# Creates the tfstate S3 bucket and DynamoDB lock table if they don't already exist.

set -euo pipefail

PROFILE="${AWS_PROFILE:-deploy-baba}"
REGION="${AWS_REGION:-us-east-1}"

echo "→ Using profile: ${PROFILE}"
echo "→ Using region:  ${REGION}"

# Resolve account ID
ACCOUNT_ID=$(aws sts get-caller-identity --profile "${PROFILE}" --query Account --output text)
BUCKET="deploy-baba-tfstate-${ACCOUNT_ID}"
TABLE="terraform-lock"

echo "→ State bucket:   ${BUCKET}"
echo "→ Lock table:     ${TABLE}"
echo ""

ERRORS=0

# ── S3 state bucket ──────────────────────────────────────────────────────────

if aws s3api head-bucket --bucket "${BUCKET}" --profile "${PROFILE}" 2>/dev/null; then
    echo "✓ State bucket already exists: ${BUCKET}"
else
    echo "→ Creating state bucket: ${BUCKET}"
    if [[ "${REGION}" == "us-east-1" ]]; then
        aws s3api create-bucket \
            --bucket "${BUCKET}" \
            --region "${REGION}" \
            --profile "${PROFILE}"
    else
        aws s3api create-bucket \
            --bucket "${BUCKET}" \
            --region "${REGION}" \
            --create-bucket-configuration LocationConstraint="${REGION}" \
            --profile "${PROFILE}"
    fi
    echo "✓ State bucket created: ${BUCKET}"
fi

aws s3api put-bucket-versioning \
    --bucket "${BUCKET}" \
    --versioning-configuration Status=Enabled \
    --profile "${PROFILE}"
echo "✓ Versioning enabled"

aws s3api put-public-access-block \
    --bucket "${BUCKET}" \
    --public-access-block-configuration "BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true" \
    --profile "${PROFILE}"
echo "✓ Public access blocked"

aws s3api put-bucket-encryption \
    --bucket "${BUCKET}" \
    --server-side-encryption-configuration '{"Rules":[{"ApplyServerSideEncryptionByDefault":{"SSEAlgorithm":"AES256"}}]}' \
    --profile "${PROFILE}"
echo "✓ Encryption enabled"

# ── DynamoDB lock table ───────────────────────────────────────────────────────

if aws dynamodb describe-table --table-name "${TABLE}" --profile "${PROFILE}" --region "${REGION}" 2>/dev/null | grep -q '"TableStatus"'; then
    echo "✓ Lock table already exists: ${TABLE}"
else
    echo "→ Creating DynamoDB lock table: ${TABLE}"
    aws dynamodb create-table \
        --table-name "${TABLE}" \
        --attribute-definitions AttributeName=LockID,AttributeType=S \
        --key-schema AttributeName=LockID,KeyType=HASH \
        --billing-mode PAY_PER_REQUEST \
        --region "${REGION}" \
        --profile "${PROFILE}"
    echo "✓ Lock table created: ${TABLE}"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
if [[ ${ERRORS} -eq 0 ]]; then
    echo "Bootstrap complete. State backend is ready."
    echo ""
    echo "  Bucket: ${BUCKET}"
    echo "  Table:  ${TABLE}"
    echo "  Region: ${REGION}"
else
    echo "Bootstrap finished with ${ERRORS} error(s). Review output above." >&2
    exit 1
fi
