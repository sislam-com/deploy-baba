# ─── AWS Secrets Manager — managed secrets ────────────────────────────────────
#
# All sensitive values live here. Use `just secret-put NAME VALUE PROFILE` to
# write real values after `just infra-apply`. Never store secrets in tfvars,
# env vars, or source code.

# --- pow-secret ---

resource "aws_secretsmanager_secret" "pow_secret" {
  name = "${var.project_name}/${var.environment}/pow-secret"
  tags = { Name = "${var.project_name}-pow-secret" }
}

resource "aws_secretsmanager_secret_version" "pow_secret_initial" {
  secret_id     = aws_secretsmanager_secret.pow_secret.id
  secret_string = "placeholder-set-via-just-secret-put"

  lifecycle {
    ignore_changes = [secret_string]
  }
}

# --- cognito-temp-password ---

resource "aws_secretsmanager_secret" "cognito_temp_password" {
  name = "${var.project_name}/${var.environment}/cognito-temp-password"
  tags = { Name = "${var.project_name}-cognito-temp-password" }
}

resource "aws_secretsmanager_secret_version" "cognito_temp_password_initial" {
  secret_id     = aws_secretsmanager_secret.cognito_temp_password.id
  secret_string = "Placeholder-set-via-just-secret-put-1!"

  lifecycle {
    ignore_changes = [secret_string]
  }
}

# --- anthropic-api-key ---

resource "aws_secretsmanager_secret" "anthropic_api_key" {
  name = "${var.project_name}/${var.environment}/anthropic-api-key"
  tags = { Name = "${var.project_name}-anthropic-api-key" }
}

resource "aws_secretsmanager_secret_version" "anthropic_api_key_placeholder" {
  secret_id     = aws_secretsmanager_secret.anthropic_api_key.id
  secret_string = "placeholder-set-via-just-secret-put"

  lifecycle {
    ignore_changes = [secret_string]
  }
}

# --- openai-api-key ---

resource "aws_secretsmanager_secret" "openai_api_key" {
  name = "${var.project_name}/${var.environment}/openai-api-key"
  tags = { Name = "${var.project_name}-openai-api-key" }
}

resource "aws_secretsmanager_secret_version" "openai_api_key_placeholder" {
  secret_id     = aws_secretsmanager_secret.openai_api_key.id
  secret_string = "placeholder-set-via-just-secret-put"

  lifecycle {
    ignore_changes = [secret_string]
  }
}

# --- ses-config: email sender/recipient addresses ---
# Backing secret for SES_FROM_EMAIL, SES_ACK_FROM_EMAIL, CONTACT_TO_EMAIL.
# The email Lambda reads these from env vars set by lambda.tf, but this secret
# provides a durable, auditable backup per W-SEC policy.

resource "aws_secretsmanager_secret" "ses_config" {
  name = "${var.project_name}/${var.environment}/ses-config"
  tags = { Name = "${var.project_name}-ses-config" }
}

resource "aws_secretsmanager_secret_version" "ses_config" {
  secret_id = aws_secretsmanager_secret.ses_config.id
  secret_string = jsonencode({
    ses_from_email     = "noreply@${local.ses_domain}"
    ses_ack_from_email = "it@sislam.com"
    contact_to_email   = var.contact_email
  })
}

# --- deploy-config: CI/CD deploy identifiers, self-populated from infra outputs ---
# Re-populated on every `just infra-apply`. CI reads this to avoid storing
# bucket/distribution IDs in GitHub Variables (W-SEC alignment).

resource "aws_secretsmanager_secret" "deploy_config" {
  name        = "${var.project_name}/${var.environment}/deploy-config"
  description = "CI/CD deploy identifiers — auto-populated by tofu apply"
  tags        = { Name = "${var.project_name}-deploy-config" }
}

resource "aws_secretsmanager_secret_version" "deploy_config" {
  secret_id = aws_secretsmanager_secret.deploy_config.id
  secret_string = jsonencode({
    spa_bucket    = aws_s3_bucket.spa.id
    cloudfront_id = local.is_prod_cdn ? aws_cloudfront_distribution.main[0].id : ""
    ui_fn_name    = aws_lambda_function.baba.function_name
    fn_url        = "https://${local.effective_domain}"
  })
}

# --- IAM: Lambda can read managed secrets ---

resource "aws_iam_role_policy" "lambda_secretsmanager" {
  name = "${local.lambda_function_name}-secretsmanager-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = "secretsmanager:GetSecretValue"
      Resource = [
        aws_secretsmanager_secret.pow_secret.arn,
        aws_secretsmanager_secret.cognito_temp_password.arn,
        aws_secretsmanager_secret.anthropic_api_key.arn,
        aws_secretsmanager_secret.openai_api_key.arn,
        aws_secretsmanager_secret.ses_config.arn,
        aws_secretsmanager_secret.deploy_config.arn,
      ]
    }]
  })
}
