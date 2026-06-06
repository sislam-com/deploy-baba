# ─── Contact Lambda (no VPC, PoW validation + email Lambda delegation) ───────
#
# Handles POST /api/contact and GET /api/contact/challenge.
# No VPC — invokes the email Lambda via SDK over the Lambda VPC interface
# endpoint (same pattern as llm-proxy-lambda.tf).

resource "aws_cloudwatch_log_group" "contact" {
  name              = "/aws/lambda/${local.lambda_function_name}-contact"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-contact-logs"
  }
}

resource "aws_lambda_function" "contact" {
  filename      = var.contact_lambda_code_path
  function_name = "${local.lambda_function_name}-contact"
  role          = aws_iam_role.contact_lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  memory_size   = 128
  timeout       = 10
  architectures = ["arm64"]

  source_code_hash = fileexists(var.contact_lambda_code_path) ? filebase64sha256(var.contact_lambda_code_path) : null

  reserved_concurrent_executions = 10

  environment {
    variables = {
      RUST_LOG          = "info"
      EMAIL_LAMBDA_NAME = aws_lambda_function.email.function_name
      POW_SECRET_ARN    = aws_secretsmanager_secret.pow_secret.arn
    }
  }

  # NO vpc_config — needs Lambda API access to invoke email Lambda.

  depends_on = [
    aws_cloudwatch_log_group.contact,
    aws_iam_role_policy_attachment.contact_lambda_logs,
    aws_iam_role_policy.contact_lambda_permissions,
  ]

  tags = {
    Name = "${local.lambda_function_name}-contact"
  }
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "contact_lambda_execution" {
  name = "${local.lambda_function_name}-contact-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-contact-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "contact_lambda_logs" {
  role       = aws_iam_role.contact_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "contact_lambda_permissions" {
  name = "${local.lambda_function_name}-contact-permissions"
  role = aws_iam_role.contact_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid      = "InvokeEmailLambda"
        Effect   = "Allow"
        Action   = "lambda:InvokeFunction"
        Resource = aws_lambda_function.email.arn
      },
      {
        Sid    = "ReadPowSecret"
        Effect = "Allow"
        Action = "secretsmanager:GetSecretValue"
        Resource = [
          aws_secretsmanager_secret.pow_secret.arn,
        ]
      }
    ]
  })
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "contact_lambda_high_invocations" {
  alarm_name          = "${local.lambda_function_name}-contact-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 200
  alarm_description   = "Contact Lambda invocations exceeded 200/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.contact.function_name
  }
}
