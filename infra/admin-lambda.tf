# ─── Admin Lambda (VPC, dashboard CRUD — migration owner) ────────────────────

resource "aws_cloudwatch_log_group" "admin" {
  name              = "/aws/lambda/${local.lambda_function_name}-admin"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-admin-logs"
  }
}

resource "aws_lambda_function" "admin" {
  filename      = var.admin_lambda_code_path
  function_name = "${local.lambda_function_name}-admin"
  role          = aws_iam_role.admin_lambda_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  memory_size   = 256
  timeout       = 30
  architectures = ["arm64"]

  source_code_hash = fileexists(var.admin_lambda_code_path) ? filebase64sha256(var.admin_lambda_code_path) : null

  reserved_concurrent_executions = 5

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
    aws_cloudwatch_log_group.admin,
    aws_iam_role_policy_attachment.admin_lambda_logs,
    aws_iam_role_policy_attachment.admin_lambda_vpc,
    aws_iam_role_policy.admin_lambda_efs,
    aws_iam_role_policy.admin_lambda_s3,
  ]

  tags = {
    Name = "${local.lambda_function_name}-admin"
  }
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "admin_lambda_execution" {
  name = "${local.lambda_function_name}-admin-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-admin-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "admin_lambda_logs" {
  role       = aws_iam_role.admin_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "admin_lambda_vpc" {
  role       = aws_iam_role.admin_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

resource "aws_iam_role_policy" "admin_lambda_efs" {
  name = "${local.lambda_function_name}-admin-efs-policy"
  role = aws_iam_role.admin_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid    = "EFSAccess"
      Effect = "Allow"
      Action = [
        "elasticfilesystem:ClientMount",
        "elasticfilesystem:ClientWrite",
        "elasticfilesystem:ClientRootAccess"
      ]
      Resource = aws_efs_file_system.baba_db.arn
    }]
  })
}

resource "aws_iam_role_policy" "admin_lambda_s3" {
  name = "${local.lambda_function_name}-admin-s3-policy"
  role = aws_iam_role.admin_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid    = "S3BackupAccess"
      Effect = "Allow"
      Action = [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ]
      Resource = [
        aws_s3_bucket.backups.arn,
        "${aws_s3_bucket.backups.arn}/*"
      ]
    }]
  })
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "admin_lambda_high_invocations" {
  alarm_name          = "${local.lambda_function_name}-admin-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 100
  alarm_description   = "Admin Lambda invocations exceeded 100/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.admin.function_name
  }
}
