# AWS Setup Spec — deploy-baba

**Audience:** Developers setting up a new deployment | **Source:** `docs/aws-setup.md`

---

## Overview

A developer cloning this repo needs an AWS profile with a specific IAM policy.
`just aws-check` validates setup by reading a known SSM parameter (sentinel).

```
just aws-check PROFILE  →  sts:GetCallerIdentity + ssm:GetParameter /deploy-baba/sentinel
```

---

## Required IAM Permissions (least privilege)

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
    },
    {
      "Sid": "CognitoForAuth",
      "Effect": "Allow",
      "Action": [
        "cognito-idp:CreateUserPool",
        "cognito-idp:DeleteUserPool",
        "cognito-idp:DescribeUserPool",
        "cognito-idp:UpdateUserPool",
        "cognito-idp:CreateUserPoolClient",
        "cognito-idp:DeleteUserPoolClient",
        "cognito-idp:DescribeUserPoolClient",
        "cognito-idp:UpdateUserPoolClient",
        "cognito-idp:CreateUserPoolDomain",
        "cognito-idp:DeleteUserPoolDomain",
        "cognito-idp:DescribeUserPoolDomain",
        "cognito-idp:AdminCreateUser",
        "cognito-idp:AdminDeleteUser",
        "cognito-idp:AdminGetUser",
        "cognito-idp:AdminSetUserPassword",
        "cognito-idp:ListUserPools",
        "cognito-idp:ListUserPoolClients",
        "cognito-idp:TagResource",
        "cognito-idp:UntagResource"
      ],
      "Resource": "*"
    }
  ]
}
```

---

## Local `~/.aws/config` Setup

```ini
[profile deploy-baba]
region = us-east-1
output = json
# For SSO users:
sso_start_url = https://your-org.awsapps.com/start
sso_account_id = 123456789012
sso_role_name = DeployBabaDeveloper
sso_region = us-east-1
# For access key users: use ~/.aws/credentials instead
```

---

## Validation Flow (`just aws-check`)

```
1. Load profile from ~/.aws/config (or use --profile flag)
2. Call sts:GetCallerIdentity (verifies credentials are valid)
3. Call ssm:GetParameter on /deploy-baba/sentinel
4. Assert value == "deploy-baba-configured"
5. Print: ✓ AWS profile 'deploy-baba' is configured correctly
         ✓ Account: 123456789012, Region: us-east-1, User: arn:aws:iam::...
```

The sentinel parameter is created during `just infra-bootstrap`. Trying to validate
before bootstrapping gives a clear error:
```
✗ SSM parameter /deploy-baba/sentinel not found.
  Run `just infra-bootstrap --profile <profile>` first.
```

---

## SSM Parameters Used

| Parameter | Value | Set by |
|-----------|-------|--------|
| `/deploy-baba/sentinel` | `"deploy-baba-configured"` | `just infra-bootstrap` |
| `/deploy-baba/region` | e.g. `"us-east-1"` | `just infra-bootstrap` |
| `/deploy-baba/account` | e.g. `"123456789012"` | `just infra-bootstrap` |
| `/deploy-baba/prod/cognito-pool-id` | Cognito user pool ID | `just infra-apply` (W-AUTH) |
| `/deploy-baba/prod/cognito-client-id` | App client ID (public) | `just infra-apply` (W-AUTH) |
| `/deploy-baba/prod/cognito-domain` | Hosted UI domain prefix | `just infra-apply` (W-AUTH) |

---

## Cross-References
- → `plans/modules/xtask.md` — W-XT aws/validate.rs implementation
- → `plans/modules/auth.md` — W-AUTH Cognito implementation
- → `plans/cross-cutting/aws-architecture.md` — topology
- → `plans/adr/ADR-008-cognito-authentication.md` — Cognito auth decision
- → `plans/drift/DRL-2026-03-18-xtask.md` — bootstrap fixes
