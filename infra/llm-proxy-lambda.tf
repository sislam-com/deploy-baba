# ─── LLM-proxy Lambda (no VPC, internet access for api.anthropic.com) ─────────
#
# The main UI Lambda is VPC-attached (for EFS/SQLite) and cannot reach the
# internet without a NAT Gateway. This non-VPC Lambda handles the Anthropic API
# call and is invoked synchronously by the UI Lambda via SDK over the existing
# Lambda VPC interface endpoint (vpc-endpoints.tf).
#
# No Function URL — the VPC endpoint lets the UI Lambda reach the Lambda API
# internally without NAT or public exposure. Follows the same pattern as the
# email Lambda (email-lambda.tf).

resource "aws_cloudwatch_log_group" "llm_proxy" {
  name              = "/aws/lambda/${local.lambda_function_name}-llm-proxy"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-llm-proxy-logs"
  }
}

resource "aws_lambda_function" "llm_proxy" {
  filename      = var.llm_proxy_lambda_code_path
  function_name = "${local.lambda_function_name}-llm-proxy"
  role          = aws_iam_role.llm_proxy_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  memory_size   = 256
  timeout       = 30
  architectures = ["arm64"]

  source_code_hash = fileexists(var.llm_proxy_lambda_code_path) ? filebase64sha256(var.llm_proxy_lambda_code_path) : null

  # Hard cap — limits cost exposure and prevents a flood of requests from
  # exhausting account Lambda concurrency.
  reserved_concurrent_executions = 5

  environment {
    variables = {
      RUST_LOG              = "info"
      ANTHROPIC_API_KEY_ARN = aws_secretsmanager_secret.anthropic_api_key.arn
      OPENAI_API_KEY_ARN    = aws_secretsmanager_secret.openai_api_key.arn
    }
  }

  # NO vpc_config — this Lambda has direct internet access for Anthropic API calls.
  # The UI Lambda (deploy-baba-ui) is VPC-attached for EFS.

  depends_on = [
    aws_cloudwatch_log_group.llm_proxy,
    aws_iam_role_policy_attachment.llm_proxy_logs,
    aws_iam_role_policy.llm_proxy_secretsmanager,
  ]

  tags = {
    Name = "${local.lambda_function_name}-llm-proxy"
  }
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "llm_proxy_execution" {
  name = "${local.lambda_function_name}-llm-proxy-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-llm-proxy-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "llm_proxy_logs" {
  role       = aws_iam_role.llm_proxy_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "llm_proxy_secretsmanager" {
  name = "${local.lambda_function_name}-llm-proxy-secretsmanager-policy"
  role = aws_iam_role.llm_proxy_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = "secretsmanager:GetSecretValue"
      Resource = [
        aws_secretsmanager_secret.anthropic_api_key.arn,
        aws_secretsmanager_secret.openai_api_key.arn,
      ]
    }]
  })
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "llm_proxy_high_invocations" {
  alarm_name          = "${local.lambda_function_name}-llm-proxy-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 100
  alarm_description   = "LLM-proxy Lambda invocations exceeded 100/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.llm_proxy.function_name
  }
}
