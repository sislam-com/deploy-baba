# AWS Setup Guide

This guide walks you through configuring AWS access for deploying deploy-baba.

## 1. Create an AWS Profile

Add this to `~/.aws/config`:

```ini
[profile deploy-baba]
region = us-east-1
output = json
```

For access key authentication, add credentials to `~/.aws/credentials`:

```ini
[deploy-baba]
aws_access_key_id = YOUR_KEY
aws_secret_access_key = YOUR_SECRET
```

For SSO users, run `aws configure sso` once and accept defaults for the `deploy-baba` profile (SSO start URL and account ID are in your AWS access portal). Then refresh credentials daily with:

```bash
just sso-login
```

This calls `aws sso login --profile deploy-baba`, opens the browser for authorization, and populates `~/.aws/sso/cache`. All `just` recipes that start the local binary (`just ui`, `just dev-stack`) strip stale static keys and set `AWS_PROFILE=deploy-baba`, so the SSO cache is picked up automatically.

The new `just dev-env` recipe fetches all config needed by the local Rust binary and prints `export X=Y` lines. It is sourced at startup by `just ui` (and via cascade, `just dev-stack`):

- **Cognito** — pool ID, client ID, and hosted-UI domain are fetched from SSM Parameter Store (`/deploy-baba/prod/cognito-pool-id`, `cognito-client-id`, `cognito-domain`). JWKS is fetched from the public Cognito endpoint (`https://cognito-idp.us-east-1.amazonaws.com/<pool_id>/.well-known/jwks.json`). With these set, the binary runs with real Cognito auth (no dev-mode bypass). The Cognito user pool client already whitelists `http://localhost:3000/auth/callback` so local login works.
- **Anthropic key** — `ANTHROPIC_API_KEY_ARN=root-anthropic-access-key` (Secrets Manager bare name). Binary fetches value at cold start.
- **RAG Q&A** — `RAG_PUBLIC_ENABLED=1` enables the `/api/ask` route locally.
- **App domain** — `APP_DOMAIN=http://localhost:3000` for correct OAuth redirect URLs.

If SSO has lapsed or the role lacks `ssm:GetParameter` access, `just dev-env` fails fast before the binary starts. Re-run `just sso-login` and retry.

## 2. Required IAM Permissions

Attach this policy to the IAM user/role used for deployment. It covers both
CI/CD operations and full OpenTofu provisioning (first-time `just infra-apply`).

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "STSValidation",
      "Effect": "Allow",
      "Action": ["sts:GetCallerIdentity"],
      "Resource": "*"
    },
    {
      "Sid": "SSMAccess",
      "Effect": "Allow",
      "Action": ["ssm:GetParameter", "ssm:PutParameter", "ssm:DeleteParameter", "ssm:DescribeParameters"],
      "Resource": "arn:aws:ssm:*:*:parameter/deploy-baba/*"
    },
    {
      "Sid": "ECRPublicPush",
      "Effect": "Allow",
      "Action": ["ecr-public:*"],
      "Resource": "*"
    },
    {
      "Sid": "LambdaFullLifecycle",
      "Effect": "Allow",
      "Action": [
        "lambda:CreateFunction", "lambda:DeleteFunction", "lambda:GetFunction",
        "lambda:UpdateFunctionCode", "lambda:UpdateFunctionConfiguration",
        "lambda:PublishVersion", "lambda:AddPermission", "lambda:RemovePermission",
        "lambda:CreateFunctionUrlConfig", "lambda:UpdateFunctionUrlConfig",
        "lambda:GetFunctionUrlConfig", "lambda:ListFunctions"
      ],
      "Resource": "arn:aws:lambda:*:*:function:deploy-baba-*"
    },
    {
      "Sid": "IAMRolesForTerraform",
      "Effect": "Allow",
      "Action": [
        "iam:CreateRole", "iam:DeleteRole", "iam:GetRole", "iam:ListRoles",
        "iam:AttachRolePolicy", "iam:DetachRolePolicy", "iam:PutRolePolicy",
        "iam:DeleteRolePolicy", "iam:GetRolePolicy", "iam:ListRolePolicies",
        "iam:ListAttachedRolePolicies", "iam:PassRole", "iam:TagRole", "iam:UntagRole"
      ],
      "Resource": "arn:aws:iam::*:role/deploy-baba-*"
    },
    {
      "Sid": "S3StateAndBackups",
      "Effect": "Allow",
      "Action": [
        "s3:CreateBucket", "s3:DeleteBucket", "s3:GetBucketLocation",
        "s3:ListBucket", "s3:GetBucketVersioning", "s3:PutBucketVersioning",
        "s3:GetBucketPolicy", "s3:PutBucketPolicy", "s3:DeleteBucketPolicy",
        "s3:GetObject", "s3:PutObject", "s3:DeleteObject",
        "s3:GetEncryptionConfiguration", "s3:PutEncryptionConfiguration"
      ],
      "Resource": [
        "arn:aws:s3:::deploy-baba-*",
        "arn:aws:s3:::deploy-baba-*/*"
      ]
    },
    {
      "Sid": "DynamoDBTerraformLock",
      "Effect": "Allow",
      "Action": [
        "dynamodb:CreateTable", "dynamodb:DeleteTable", "dynamodb:DescribeTable",
        "dynamodb:GetItem", "dynamodb:PutItem", "dynamodb:DeleteItem"
      ],
      "Resource": "arn:aws:dynamodb:*:*:table/terraform-lock"
    },
    {
      "Sid": "EFSForSQLite",
      "Effect": "Allow",
      "Action": [
        "elasticfilesystem:CreateFileSystem", "elasticfilesystem:DeleteFileSystem",
        "elasticfilesystem:DescribeFileSystems",
        "elasticfilesystem:CreateMountTarget", "elasticfilesystem:DeleteMountTarget",
        "elasticfilesystem:DescribeMountTargets",
        "elasticfilesystem:CreateAccessPoint", "elasticfilesystem:DeleteAccessPoint",
        "elasticfilesystem:DescribeAccessPoints",
        "elasticfilesystem:ClientMount", "elasticfilesystem:ClientWrite",
        "elasticfilesystem:PutLifecycleConfiguration", "elasticfilesystem:TagResource"
      ],
      "Resource": "*"
    },
    {
      "Sid": "EC2VPCForLambdaAndEFS",
      "Effect": "Allow",
      "Action": [
        "ec2:DescribeVpcs", "ec2:DescribeSubnets", "ec2:DescribeSecurityGroups",
        "ec2:CreateSecurityGroup", "ec2:DeleteSecurityGroup",
        "ec2:AuthorizeSecurityGroupIngress", "ec2:AuthorizeSecurityGroupEgress",
        "ec2:RevokeSecurityGroupIngress", "ec2:RevokeSecurityGroupEgress",
        "ec2:DescribeNetworkInterfaces", "ec2:CreateNetworkInterface",
        "ec2:DeleteNetworkInterface", "ec2:DescribeAvailabilityZones"
      ],
      "Resource": "*"
    },
    {
      "Sid": "CloudWatchLogs",
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup", "logs:DeleteLogGroup", "logs:DescribeLogGroups",
        "logs:CreateLogStream", "logs:PutLogEvents", "logs:GetLogEvents",
        "logs:FilterLogEvents", "logs:PutRetentionPolicy"
      ],
      "Resource": "arn:aws:logs:*:*:log-group:/aws/lambda/deploy-baba-*"
    },
    {
      "Sid": "EventBridgeBackupSchedule",
      "Effect": "Allow",
      "Action": [
        "events:CreateRule", "events:DeleteRule", "events:DescribeRule",
        "events:PutRule", "events:PutTargets", "events:RemoveTargets",
        "events:ListTargetsByRule", "events:TagResource"
      ],
      "Resource": "arn:aws:events:*:*:rule/deploy-baba-*"
    }
  ]
}
```

## 3. SES Manual Setup (One-Time, Per Account + Region)

SES configuration has two layers:

**Managed by OpenTofu** (`infra/ses.tf`):
- Domain identity for `mail.sislam.com` with DKIM, SPF, and DMARC records

**NOT managed by OpenTofu — manual AWS Console steps:**
- Verifying the `it@sislam.com` email identity (console wizard + click-through link)
- Requesting SES production access (account + region scoped, Support ticket form)

Both are one-time setup per target account. Neither has a Terraform/OpenTofu resource
that can automate the human-in-the-loop verification.

### 3a. Verify the sender email identity

1. AWS Console → SES → **Verified identities** → **Create identity** → Email address
2. Enter the address used as `SES_ACK_FROM_EMAIL` (currently `it@sislam.com`)
3. Open the verification email AWS sends; click the confirmation link
4. Confirm status shows **Verified** in the console

Verification command (no `--profile` flag needed if using the default profile):
```bash
aws sesv2 get-email-identity --email-identity it@sislam.com --region us-east-1
# append --profile <name> only if using a non-default profile
```

### 3b. Request SES production access

By default, new SES accounts are in **sandbox mode**: `send_email` to any
unverified recipient address is rejected with `MessageRejected`. Production access
removes this restriction.

1. AWS Console → SES → **Account dashboard** → **Request production access**
2. Fill out the form:
   - **Mail type:** Transactional
   - **Website URL:** `https://sislam.com`
   - **Use case description:**
     > Transactional acknowledgement emails for a personal portfolio contact form.
     > Each email is opt-in — the submitter initiated the request by filling out the
     > form. Low volume (≤100/day). Messages contain a thank-you note and a verbatim
     > copy of what the user submitted.
   - Acknowledge the bounce/complaint handling commitment
3. Submit and wait for approval (typically within 24 hours)

Verification command once approved:
```bash
aws sesv2 get-account --region us-east-1
# append --profile <name> only for a non-default profile
```
Look for:
- `ProductionAccessEnabled: true`
- `EnforcementStatus: HEALTHY`
- `Details.ReviewDetails.Status: GRANTED`

Note the `CaseId` from `Details.ReviewDetails.CaseId` for your records.

### 3c. Deploy after production access is granted

```bash
just infra-apply <profile>   # restores SES_ACK_FROM_EMAIL in the Lambda env
just email-deploy <profile>  # pushes the email Lambda binary
```

### 3d. End-to-end verification

1. Submit the contact form at `https://sislam.com/contact` using an external
   (non-verified) Gmail address
2. Confirm the admin notification arrives at `contact-sislam@shantopagla.com`
3. Confirm the acknowledgement email arrives in the Gmail inbox
4. Tail the email Lambda logs and look for `info!(to = ..., "acknowledgement email sent")`
   with no `warn!(code = "message_rejected", ...)` lines:
   ```bash
   just email-logs <profile>
   ```

### 3e. Troubleshooting

If the ack email stops working (e.g. after expanding to a new region, or if email
identity verification lapses), the symptom is a `warn!(code = "message_rejected", ...)`
line in the email Lambda logs and no ack in the submitter's inbox. Admin notifications
are unaffected — they go to a verified identity.

See `plans/drift/DRL-2026-04-07-ses-sandbox-ack.md` for the historical record of
this exact failure mode, the interim mitigation, and the resolution evidence.

---

## 3f. CloudFront SPA Serving

The React SPA is served via CloudFront with S3 Origin Access Control (OAC). See [ADR-019](../plans/adr/ADR-019-spa-replaces-askama.md).

- **S3 bucket** (`infra/s3-spa.tf`): stores built SPA assets (`web/dist/`)
- **CloudFront distribution** (`infra/cdn.tf`): two origins — S3 for static assets, Lambda Function URL for `/api/*`
- **OAC**: CloudFront signs requests to S3 — the bucket is not publicly accessible
- **SPA routing**: CloudFront custom error responses return `index.html` for 403/404 on non-API paths, enabling client-side routing

CI deploys SPA assets via `aws s3 sync web/dist/ s3://$SPA_BUCKET` followed by a CloudFront invalidation.

## 3g. EventBridge Backup

A daily EventBridge rule triggers a SQLite backup to S3 (`infra/eventbridge.tf`):

1. EventBridge fires the backup target on the UI Lambda
2. The handler runs `VACUUM INTO` to create a consistent snapshot
3. The snapshot is uploaded to the S3 backup bucket with a date-stamped key

No manual setup needed — this is fully managed by OpenTofu.

## 3h. API Versioning

API endpoints use URL-based versioning: `/api/v1/jobs`, `/api/v1/competencies`, etc. See [ADR-024](../plans/adr/ADR-024-api-versioning-strategy.md).

The versioning middleware adds deprecation headers when an API version is scheduled for removal. The Function URL routes all `/api/*` traffic to the UI Lambda, which handles version dispatch internally.

## 3i. LLM Proxy Lambda

The `llm-proxy` Lambda (`infra/llm-proxy-lambda.tf`) routes LLM requests to the Anthropic API.

- **Secrets**: the Anthropic API key is stored in Secrets Manager (`deploy-baba/prod/anthropic-api-key`)
- **No VPC**: needs direct internet access for the Anthropic Messages API
- **Invocation**: called by the UI Lambda via `aws_sdk_lambda::Client::invoke()`

To set up the API key:
```bash
just secret-put anthropic-api-key <your-key> deploy-baba
```

---

## 4. Bootstrap (First Time Only)

```bash
just infra-bootstrap deploy-baba
```

This creates the S3 state bucket for OpenTofu and writes the SSM sentinel
parameter that `just aws-check` uses for validation.

## 5. Validate Setup

```bash
just aws-check deploy-baba
```

Expected output:
```
✓ AWS profile 'deploy-baba' is configured correctly
✓ Account: 123456789012, Region: us-east-1
```

## 6. Deploy

```bash
just infra-apply deploy-baba    # Provision Lambda, EFS, S3
just deploy deploy-baba         # Build + push + update Lambda
just ui-open deploy-baba        # Open the live site
```
