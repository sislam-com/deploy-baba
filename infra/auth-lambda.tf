# ─── Auth Lambda (no VPC, public Function URL) ─────────────────────────────────
#
# Custom login flow for the React SPA. Proxies Cognito IDP via AWS SDK.
# This Lambda is public (no VPC) so it can reach the Cognito public endpoint.
# The SPA calls it via CloudFront origin routing on /api/auth/*.
#
# Flow:
#   SPA → POST /api/auth/signin → auth Lambda → cognito-idp:InitiateAuth
#   On success: SPA → GET /auth/set-session → UI Lambda (HttpOnly cookie)

resource "aws_cloudwatch_log_group" "auth" {
  name              = "/aws/lambda/${local.lambda_function_name}-auth"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-auth-logs"
  }
}

resource "aws_lambda_function" "auth" {
  filename      = var.auth_lambda_code_path
  function_name = "${local.lambda_function_name}-auth"
  role          = aws_iam_role.auth_lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  memory_size   = 128
  timeout       = 10
  architectures = ["arm64"]

  source_code_hash = fileexists(var.auth_lambda_code_path) ? filebase64sha256(var.auth_lambda_code_path) : null

  # Hard cap on concurrent invocations — limits cost exposure.
  reserved_concurrent_executions = 10

  environment {
    variables = {
      RUST_LOG          = "info"
      COGNITO_POOL_ID   = aws_cognito_user_pool.baba.id
      COGNITO_CLIENT_ID = aws_cognito_user_pool_client.baba_web.id
      COGNITO_REGION    = var.region
      ALLOWED_ORIGIN    = "https://${local.effective_domain}"
    }
  }

  # NO vpc_config — direct internet access for Cognito IDP calls.

  depends_on = [
    aws_cloudwatch_log_group.auth,
    aws_iam_role_policy_attachment.auth_lambda_logs,
    aws_iam_role_policy.auth_lambda_cognito_idp,
  ]

  tags = {
    Name = "${local.lambda_function_name}-auth"
  }
}

# Lambda Function URL — public HTTPS endpoint for SPA calls via CloudFront
resource "aws_lambda_function_url" "auth" {
  function_name      = aws_lambda_function.auth.function_name
  authorization_type = "AWS_IAM"

  depends_on = [aws_lambda_function.auth]
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "auth_lambda_execution" {
  name = "${local.lambda_function_name}-auth-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-auth-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "auth_lambda_logs" {
  role       = aws_iam_role.auth_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "auth_lambda_cognito_idp" {
  name = "${local.lambda_function_name}-auth-cognito-idp-policy"
  role = aws_iam_role.auth_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "cognito-idp:InitiateAuth",
        "cognito-idp:RespondToAuthChallenge",
        "cognito-idp:ForgotPassword",
        "cognito-idp:ConfirmForgotPassword",
        "cognito-idp:GlobalSignOut"
      ]
      Resource = aws_cognito_user_pool.baba.arn
    }]
  })
}

# ─── CloudFront Permission ────────────────────────────────────────────────────

resource "aws_lambda_permission" "cloudfront_auth" {
  count         = var.environment == "prod" ? 1 : 0
  statement_id  = "AllowCloudFrontInvokeAuth"
  action        = "lambda:InvokeFunctionUrl"
  function_name = aws_lambda_function.auth.function_name
  principal     = "cloudfront.amazonaws.com"
  source_arn    = aws_cloudfront_distribution.main[0].arn
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "auth_lambda_high_invocations" {
  alarm_name          = "${local.lambda_function_name}-auth-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 200
  alarm_description   = "Auth Lambda invocations exceeded 200/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.auth.function_name
  }
}
