# ─── RAG Lambda (VPC, FTS5 retrieval + grounded generation) ──────────────────
#
# Handles POST /api/ask and POST /api/v1/rag/ask.
# VPC-attached for EFS/SQLite read access. Reaches the LLM-proxy Lambda via
# the Lambda VPC interface endpoint (vpc-endpoints.tf) for generation.

resource "aws_cloudwatch_log_group" "rag" {
  name              = "/aws/lambda/${local.lambda_function_name}-rag"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-rag-logs"
  }
}

resource "aws_lambda_function" "rag" {
  filename      = var.rag_lambda_code_path
  function_name = "${local.lambda_function_name}-rag"
  role          = aws_iam_role.rag_lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  memory_size   = 256
  timeout       = 30
  architectures = ["arm64"]

  source_code_hash = fileexists(var.rag_lambda_code_path) ? filebase64sha256(var.rag_lambda_code_path) : null

  reserved_concurrent_executions = 5

  environment {
    variables = {
      DB_PATH               = "/mnt/db/baba.db"
      RUST_LOG              = "info"
      ANTHROPIC_API_KEY_ARN = aws_secretsmanager_secret.anthropic_api_key.arn
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
    aws_cloudwatch_log_group.rag,
    aws_iam_role_policy_attachment.rag_lambda_logs,
    aws_iam_role_policy_attachment.rag_lambda_vpc,
    aws_iam_role_policy.rag_lambda_efs,
    aws_iam_role_policy.rag_lambda_secretsmanager,
  ]

  tags = {
    Name = "${local.lambda_function_name}-rag"
  }
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "rag_lambda_execution" {
  name = "${local.lambda_function_name}-rag-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-rag-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "rag_lambda_logs" {
  role       = aws_iam_role.rag_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "rag_lambda_vpc" {
  role       = aws_iam_role.rag_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

resource "aws_iam_role_policy" "rag_lambda_efs" {
  name = "${local.lambda_function_name}-rag-efs-policy"
  role = aws_iam_role.rag_lambda_execution.id

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

resource "aws_iam_role_policy" "rag_lambda_secretsmanager" {
  name = "${local.lambda_function_name}-rag-secretsmanager-policy"
  role = aws_iam_role.rag_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = "secretsmanager:GetSecretValue"
      Resource = [
        aws_secretsmanager_secret.anthropic_api_key.arn,
      ]
    }]
  })
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "rag_lambda_high_invocations" {
  alarm_name          = "${local.lambda_function_name}-rag-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 100
  alarm_description   = "RAG Lambda invocations exceeded 100/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.rag.function_name
  }
}
