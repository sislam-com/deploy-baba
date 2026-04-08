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

For SSO users, configure SSO fields in the profile section instead.

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
