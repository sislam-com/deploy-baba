# GitHub Actions OIDC authentication — ADR-020
# Allows CI to assume deploy roles without long-lived IAM keys.

# GitHub's OIDC provider (one per account, shared across repos)
resource "aws_iam_openid_connect_provider" "github" {
  url = "https://token.actions.githubusercontent.com"

  client_id_list = ["sts.amazonaws.com"]

  # Thumbprint list: GitHub's OIDC TLS cert thumbprint (stable, verified 2024)
  thumbprint_list = ["6938fd4d98bab03faadb97b34396831e3780aea1"]

  tags = {
    Name      = "github-actions-oidc"
    ManagedBy = "OpenTofu"
  }

  lifecycle {
    # A second apply must not re-create if already present
    ignore_changes = [thumbprint_list]
  }
}

# ── Dev deploy role (triggered by pushes to main) ─────────────────────────────

resource "aws_iam_role" "ci_deploy_dev" {
  name = "${var.project_name}-ci-deploy-dev"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.github.arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com"
        }
        StringLike = {
          # Allows any ref on any branch (workflow_run fires after CI passes on main)
          "token.actions.githubusercontent.com:sub" = "repo:shantopagla/deploy-baba:*"
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
  name = "deploy-dev-policy"
  role = aws_iam_role.ci_deploy_dev.id

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
        Resource = "arn:aws:lambda:${var.region}:${data.aws_caller_identity.current.account_id}:function:${var.project_name}-prod"
      },
      {
        Sid      = "ReadDeployConfig"
        Effect   = "Allow"
        Action   = "secretsmanager:GetSecretValue"
        Resource = "arn:aws:secretsmanager:${var.region}:${data.aws_caller_identity.current.account_id}:secret:${var.project_name}/${var.environment}/deploy-config*"
      },
      {
        Sid      = "InvalidateCDN"
        Effect   = "Allow"
        Action   = "cloudfront:CreateInvalidation"
        Resource = "arn:aws:cloudfront::${data.aws_caller_identity.current.account_id}:distribution/${aws_cloudfront_distribution.main.id}"
      }
    ]
  })
}

# ── Prod deploy role (triggered by vX.Y.Z tag pushes, gated by production env) ──

resource "aws_iam_role" "ci_deploy_prod" {
  name = "${var.project_name}-ci-deploy-prod"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.github.arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com"
        }
        StringLike = {
          # Only v* tags (release-promote output)
          "token.actions.githubusercontent.com:sub" = "repo:shantopagla/deploy-baba:ref:refs/tags/v*"
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
  name = "deploy-prod-policy"
  role = aws_iam_role.ci_deploy_prod.id

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
        Resource = "arn:aws:lambda:${var.region}:${data.aws_caller_identity.current.account_id}:function:${var.project_name}-prod"
      },
      {
        Sid      = "ReadDeployConfig"
        Effect   = "Allow"
        Action   = "secretsmanager:GetSecretValue"
        Resource = "arn:aws:secretsmanager:${var.region}:${data.aws_caller_identity.current.account_id}:secret:${var.project_name}/${var.environment}/deploy-config*"
      },
      {
        Sid      = "InvalidateCDN"
        Effect   = "Allow"
        Action   = "cloudfront:CreateInvalidation"
        Resource = "arn:aws:cloudfront::${data.aws_caller_identity.current.account_id}:distribution/${aws_cloudfront_distribution.main.id}"
      }
    ]
  })
}

# ── Outputs (copy these into GitHub Actions Variables) ────────────────────────

output "ci_deploy_dev_role_arn" {
  description = "Set as CI_DEPLOY_DEV_ROLE_ARN in GitHub Actions variables"
  value       = aws_iam_role.ci_deploy_dev.arn
}

output "ci_deploy_prod_role_arn" {
  description = "Set as CI_DEPLOY_PROD_ROLE_ARN in GitHub Actions variables"
  value       = aws_iam_role.ci_deploy_prod.arn
}
