# CloudWatch Log Group for Lambda
resource "aws_cloudwatch_log_group" "lambda" {
  name              = "/aws/lambda/${local.lambda_function_name}"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-logs"
  }
}

# Lambda function for the Baba portfolio site
resource "aws_lambda_function" "baba" {
  filename         = var.lambda_code_path
  function_name    = local.lambda_function_name
  role             = aws_iam_role.lambda_execution.arn
  handler          = "index.handler"
  runtime          = "provided.al2023"
  memory_size      = var.lambda_memory
  timeout          = var.lambda_timeout
  architectures    = ["arm64"]
  source_code_hash = fileexists(var.lambda_code_path) ? filebase64sha256(var.lambda_code_path) : null

  # Environment variables passed to the Lambda function
  environment {
    variables = {
      DB_PATH               = "/mnt/db/baba.db"
      RUST_LOG              = "info"
      COGNITO_POOL_ID       = aws_cognito_user_pool.baba.id
      COGNITO_CLIENT_ID     = aws_cognito_user_pool_client.baba_web.id
      COGNITO_DOMAIN        = "${aws_cognito_user_pool_domain.baba.domain}.auth.${var.region}.amazoncognito.com"
      COGNITO_REGION        = var.region
      APP_DOMAIN            = "https://${var.domain_name}"
      EMAIL_LAMBDA_NAME     = aws_lambda_function.email.function_name
      COGNITO_JWKS          = data.http.cognito_jwks.response_body
      POW_SECRET_ARN        = aws_secretsmanager_secret.pow_secret.arn
      ANTHROPIC_API_KEY_ARN = aws_secretsmanager_secret.anthropic_api_key.arn
      SPA_ROOT              = "/mnt/spa/active"
      SPA_BUCKET            = aws_s3_bucket.spa.id
    }
  }

  # EFS mount — SQLite database
  file_system_config {
    arn              = aws_efs_access_point.db.arn
    local_mount_path = "/mnt/db"
  }

  # VPC configuration for EFS access
  vpc_config {
    subnet_ids         = data.aws_subnets.default.ids
    security_group_ids = [aws_security_group.lambda_efs.id]
  }

  # Explicit CloudWatch Logs dependency
  depends_on = [
    aws_cloudwatch_log_group.lambda,
    aws_iam_role_policy_attachment.lambda_logs,
    aws_iam_role_policy_attachment.lambda_vpc,
    aws_iam_role_policy.lambda_efs,
    aws_iam_role_policy.lambda_s3,
    aws_iam_role_policy.lambda_s3_spa,
    aws_iam_role_policy.lambda_ssm,
    aws_iam_role_policy.lambda_invoke_email,
    aws_iam_role_policy.lambda_secretsmanager,
  ]

  tags = {
    Name = local.lambda_function_name
  }
}

# Lambda Function URL for direct HTTPS invocation (no API Gateway)
resource "aws_lambda_function_url" "baba" {
  function_name      = aws_lambda_function.baba.function_name
  authorization_type = "AWS_IAM"

  depends_on = [aws_lambda_function.baba]
}

# Lambda permission — CloudFront OAC only
resource "aws_lambda_permission" "cloudfront" {
  statement_id  = "AllowCloudFrontInvoke"
  action        = "lambda:InvokeFunctionUrl"
  function_name = aws_lambda_function.baba.function_name
  principal     = "cloudfront.amazonaws.com"
  source_arn    = aws_cloudfront_distribution.main.arn
}

resource "aws_lambda_permission" "cloudfront_invoke" {
  statement_id  = "AllowCloudFrontInvokeFunction"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.baba.function_name
  principal     = "cloudfront.amazonaws.com"
  source_arn    = aws_cloudfront_distribution.main.arn
}

