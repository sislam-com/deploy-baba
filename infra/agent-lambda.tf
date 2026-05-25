# ─── Agent Lambda (Python/LangGraph, no VPC) ────────────────────────────────────
#
# Cover letter generation agent (ADR-033/034). Runs LangGraph ReAct loop,
# calls back into UI Lambda for resume/matcher data, stores artifacts in S3.
# No VPC — reaches Anthropic API and other Lambdas via SDK.

resource "aws_cloudwatch_log_group" "agent" {
  name              = "/aws/lambda/${local.lambda_function_name}-agent"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-agent-logs"
  }
}

resource "aws_lambda_function" "agent" {
  filename      = var.agent_lambda_code_path
  function_name = "${local.lambda_function_name}-agent"
  role          = aws_iam_role.agent_lambda_execution.arn
  handler       = "handler.handler"
  runtime       = "python3.13"
  memory_size   = 512
  timeout       = 120
  architectures = ["arm64"]

  source_code_hash = fileexists(var.agent_lambda_code_path) ? filebase64sha256(var.agent_lambda_code_path) : null

  reserved_concurrent_executions = 5

  environment {
    variables = {
      ANTHROPIC_API_KEY_ARN = aws_secretsmanager_secret.anthropic_api_key.arn
      UI_LAMBDA_NAME        = aws_lambda_function.baba.function_name
      ARTIFACTS_BUCKET      = aws_s3_bucket.assets.id
      AWS_REGION_OVERRIDE   = var.region
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.agent,
    aws_iam_role_policy_attachment.agent_lambda_logs,
    aws_iam_role_policy.agent_lambda_permissions,
  ]

  tags = {
    Name = "${local.lambda_function_name}-agent"
  }
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "agent_lambda_execution" {
  name = "${local.lambda_function_name}-agent-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-agent-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "agent_lambda_logs" {
  role       = aws_iam_role.agent_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "agent_lambda_permissions" {
  name = "${local.lambda_function_name}-agent-permissions"
  role = aws_iam_role.agent_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid      = "InvokeUILambda"
        Effect   = "Allow"
        Action   = ["lambda:InvokeFunction"]
        Resource = aws_lambda_function.baba.arn
      },
      {
        Sid      = "ReadSecrets"
        Effect   = "Allow"
        Action   = ["secretsmanager:GetSecretValue"]
        Resource = aws_secretsmanager_secret.anthropic_api_key.arn
      },
      {
        Sid    = "S3Artifacts"
        Effect = "Allow"
        Action = [
          "s3:PutObject",
          "s3:GetObject"
        ]
        Resource = "${aws_s3_bucket.assets.arn}/cover-letters/*"
      }
    ]
  })
}

# ─── S3 Lifecycle Rule for cover-letters prefix (30-day expiry) ───────────────

resource "aws_s3_bucket_lifecycle_configuration" "assets_cover_letters" {
  bucket = aws_s3_bucket.assets.id

  rule {
    id     = "expire-cover-letters"
    status = "Enabled"

    filter {
      prefix = "cover-letters/"
    }

    expiration {
      days = 30
    }
  }
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "agent_lambda_high_invocations" {
  alarm_name          = "${local.lambda_function_name}-agent-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 50
  alarm_description   = "Agent Lambda invocations exceeded 50/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.agent.function_name
  }
}
