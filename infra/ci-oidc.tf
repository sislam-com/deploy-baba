# GitHub Actions OIDC authentication — ADR-020
# Allows CI to assume deploy roles without long-lived IAM keys.
#
# All resources in this file are per-account singletons managed only in the
# prod workspace (count guard). The dev workspace skips them entirely.

locals {
  is_prod = var.environment == "prod"
}

# GitHub's OIDC provider (one per account, shared across repos)
resource "aws_iam_openid_connect_provider" "github" {
  count = local.is_prod ? 1 : 0

  url = "https://token.actions.githubusercontent.com"

  client_id_list = ["sts.amazonaws.com"]

  thumbprint_list = ["6938fd4d98bab03faadb97b34396831e3780aea1"]

  tags = {
    Name      = "github-actions-oidc"
    ManagedBy = "OpenTofu"
  }

  lifecycle {
    ignore_changes = [thumbprint_list]
  }
}

# ── Dev deploy role (triggered by pushes to main) ─────────────────────────────

resource "aws_iam_role" "ci_deploy_dev" {
  count = local.is_prod ? 1 : 0
  name  = "${var.project_name}-ci-deploy-dev"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.github[0].arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com"
        }
        StringLike = {
          "token.actions.githubusercontent.com:sub" = "repo:sislam-com/deploy-baba:*"
        }
      }
    }]
  })

  tags = {
    Name      = "${var.project_name}-ci-deploy-dev"
    ManagedBy = "OpenTofu"
  }
}

resource "aws_iam_role_policy" "ci_deploy_dev" {
  count = local.is_prod ? 1 : 0
  name  = "deploy-dev-policy"
  role  = aws_iam_role.ci_deploy_dev[0].id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "UpdateLambda"
        Effect = "Allow"
        Action = [
          "lambda:UpdateFunctionCode",
          "lambda:InvokeFunction",
          "lambda:GetFunction",
          "lambda:GetFunctionConfiguration",
          "lambda:WaitFunctionActive"
        ]
        Resource = "arn:aws:lambda:${var.region}:${data.aws_caller_identity.current.account_id}:function:${var.project_name}-prod*"
      },
      {
        Sid      = "ReadDeployConfig"
        Effect   = "Allow"
        Action   = "secretsmanager:GetSecretValue"
        Resource = "arn:aws:secretsmanager:${var.region}:${data.aws_caller_identity.current.account_id}:secret:${var.project_name}/prod/deploy-config*"
      },
      {
        Sid      = "InvalidateCDN"
        Effect   = "Allow"
        Action   = "cloudfront:CreateInvalidation"
        Resource = "arn:aws:cloudfront::${data.aws_caller_identity.current.account_id}:distribution/${aws_cloudfront_distribution.main[0].id}"
      },
      {
        Sid    = "SyncSPA"
        Effect = "Allow"
        Action = ["s3:PutObject", "s3:DeleteObject", "s3:ListBucket"]
        Resource = [
          "arn:aws:s3:::${var.project_name}-prod-spa-*",
          "arn:aws:s3:::${var.project_name}-prod-spa-*/*"
        ]
      }
    ]
  })
}

# ── Prod deploy role (triggered by vX.Y.Z tag pushes, gated by production env) ──

resource "aws_iam_role" "ci_deploy_prod" {
  count = local.is_prod ? 1 : 0
  name  = "${var.project_name}-ci-deploy-prod"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.github[0].arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com"
        }
        StringLike = {
          "token.actions.githubusercontent.com:sub" = "repo:sislam-com/deploy-baba:ref:refs/tags/v*"
        }
      }
    }]
  })

  tags = {
    Name      = "${var.project_name}-ci-deploy-prod"
    ManagedBy = "OpenTofu"
  }
}

resource "aws_iam_role_policy" "ci_deploy_prod" {
  count = local.is_prod ? 1 : 0
  name  = "deploy-prod-policy"
  role  = aws_iam_role.ci_deploy_prod[0].id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "UpdateLambda"
        Effect = "Allow"
        Action = [
          "lambda:UpdateFunctionCode",
          "lambda:InvokeFunction",
          "lambda:GetFunction",
          "lambda:GetFunctionConfiguration",
          "lambda:WaitFunctionActive"
        ]
        Resource = "arn:aws:lambda:${var.region}:${data.aws_caller_identity.current.account_id}:function:${var.project_name}-prod*"
      },
      {
        Sid      = "ReadDeployConfig"
        Effect   = "Allow"
        Action   = "secretsmanager:GetSecretValue"
        Resource = "arn:aws:secretsmanager:${var.region}:${data.aws_caller_identity.current.account_id}:secret:${var.project_name}/prod/deploy-config*"
      },
      {
        Sid      = "InvalidateCDN"
        Effect   = "Allow"
        Action   = "cloudfront:CreateInvalidation"
        Resource = "arn:aws:cloudfront::${data.aws_caller_identity.current.account_id}:distribution/${aws_cloudfront_distribution.main[0].id}"
      },
      {
        Sid    = "SyncSPA"
        Effect = "Allow"
        Action = ["s3:PutObject", "s3:DeleteObject", "s3:ListBucket"]
        Resource = [
          "arn:aws:s3:::${var.project_name}-prod-spa-*",
          "arn:aws:s3:::${var.project_name}-prod-spa-*/*"
        ]
      }
    ]
  })
}

# ── Outputs ──────────────────────────────────────────────────────────────────

output "ci_deploy_dev_role_arn" {
  description = "Set as CI_DEPLOY_DEV_ROLE_ARN in GitHub Actions variables"
  value       = local.is_prod ? aws_iam_role.ci_deploy_dev[0].arn : ""
}

output "ci_deploy_prod_role_arn" {
  description = "Set as CI_DEPLOY_PROD_ROLE_ARN in GitHub Actions variables"
  value       = local.is_prod ? aws_iam_role.ci_deploy_prod[0].arn : ""
}
