# ─── Portfolio Lambda (VPC, read-only data — jobs, competencies, about, social-links, resume, challenges) ───

resource "aws_cloudwatch_log_group" "portfolio" {
  name              = "/aws/lambda/${local.lambda_function_name}-portfolio"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-portfolio-logs"
  }
}

resource "aws_lambda_function" "portfolio" {
  filename      = var.portfolio_lambda_code_path
  function_name = "${local.lambda_function_name}-portfolio"
  role          = aws_iam_role.portfolio_lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  memory_size   = 128
  timeout       = 10
  architectures = ["arm64"]

  source_code_hash = fileexists(var.portfolio_lambda_code_path) ? filebase64sha256(var.portfolio_lambda_code_path) : null

  reserved_concurrent_executions = 10

  environment {
    variables = {
      DB_PATH  = "/mnt/db/baba.db"
      RUST_LOG = "info"
    }
  }

  file_system_config {
    arn              = aws_efs_access_point.db.arn
    local_mount_path = "/mnt/db"
  }

  vpc_config {
    subnet_ids         = data.aws_subnets.default.ids
    security_group_ids = [aws_security_group.lambda_efs.id]
  }

  depends_on = [
    aws_cloudwatch_log_group.portfolio,
    aws_iam_role_policy_attachment.portfolio_lambda_logs,
    aws_iam_role_policy_attachment.portfolio_lambda_vpc,
    aws_iam_role_policy.portfolio_lambda_efs,
  ]

  tags = {
    Name = "${local.lambda_function_name}-portfolio"
  }
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "portfolio_lambda_execution" {
  name = "${local.lambda_function_name}-portfolio-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-portfolio-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "portfolio_lambda_logs" {
  role       = aws_iam_role.portfolio_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "portfolio_lambda_vpc" {
  role       = aws_iam_role.portfolio_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

resource "aws_iam_role_policy" "portfolio_lambda_efs" {
  name = "${local.lambda_function_name}-portfolio-efs-policy"
  role = aws_iam_role.portfolio_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid    = "EFSAccess"
      Effect = "Allow"
      Action = [
        "elasticfilesystem:ClientMount",
        "elasticfilesystem:ClientRootAccess"
      ]
      Resource = aws_efs_file_system.baba_db.arn
    }]
  })
}

# ─── CloudFront Permission ────────────────────────────────────────────────────

resource "aws_lambda_permission" "cloudfront_portfolio" {
  count         = var.environment == "prod" ? 1 : 0
  statement_id  = "AllowCloudFrontInvokePortfolio"
  action        = "lambda:InvokeFunctionUrl"
  function_name = aws_lambda_function.portfolio.function_name
  principal     = "cloudfront.amazonaws.com"
  source_arn    = aws_cloudfront_distribution.main[0].arn
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "portfolio_lambda_high_invocations" {
  alarm_name          = "${local.lambda_function_name}-portfolio-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 500
  alarm_description   = "Portfolio Lambda invocations exceeded 500/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.portfolio.function_name
  }
}
