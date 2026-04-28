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
  secret_string = "placeholder-set-via-just-secret-put"

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
      ]
    }]
  })
}
