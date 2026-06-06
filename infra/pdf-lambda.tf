# PDF Conversion Lambda — Docker-based Python Lambda using WeasyPrint
# Converts HTML cover letters to PDF via POST /convert

# ECR Repository for PDF Lambda Docker images
resource "aws_ecr_repository" "pdf" {
  name                 = "${local.lambda_function_name}-pdf"
  image_tag_mutability = "MUTABLE"

  force_delete = true

  image_scanning_configuration {
    scan_on_push = true
  }

  tags = {
    Name = "${local.lambda_function_name}-pdf"
  }
}

# CloudWatch Log Group for PDF Lambda
resource "aws_cloudwatch_log_group" "pdf" {
  name              = "/aws/lambda/${local.lambda_function_name}-pdf"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-pdf-logs"
  }
}

# PDF Lambda Function — container image based
# Only created after image is built and pushed (just pdf-build)
resource "aws_lambda_function" "pdf" {
  count         = var.pdf_lambda_image_uri != "" ? 1 : 0
  function_name = "${local.lambda_function_name}-pdf"
  role          = aws_iam_role.pdf_lambda_execution.arn
  package_type  = "Image"
  image_uri     = var.pdf_lambda_image_uri

  timeout     = 60
  memory_size = 1024

  environment {
    variables = {
      AWS_REGION_OVERRIDE = var.region
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.pdf,
    aws_iam_role_policy_attachment.pdf_lambda_logs,
  ]

  tags = {
    Name = "${local.lambda_function_name}-pdf"
  }
}

# Lambda Function URL — invoked by agent Lambda
resource "aws_lambda_function_url" "pdf" {
  count              = length(aws_lambda_function.pdf) > 0 ? 1 : 0
  function_name      = aws_lambda_function.pdf[0].function_name
  authorization_type = "AWS_IAM"

  depends_on = [aws_lambda_function.pdf]
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "pdf_lambda_execution" {
  name = "${local.lambda_function_name}-pdf-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-pdf-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "pdf_lambda_logs" {
  role       = aws_iam_role.pdf_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "pdf_lambda_permissions" {
  name = "${local.lambda_function_name}-pdf-permissions"
  role = aws_iam_role.pdf_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid      = "AllowFunctionURLInvoke"
        Effect   = "Allow"
        Action   = "lambda:InvokeFunctionUrl"
        Resource = length(aws_lambda_function.pdf) > 0 ? [aws_lambda_function.pdf[0].arn] : ["*"]
      },
    ]
  })
}

# ─── Lambda Permission: Allow Agent Lambda to invoke ───────────────────────────

resource "aws_lambda_permission" "allow_agent_invoke_pdf" {
  count         = length(aws_lambda_function.pdf) > 0 ? 1 : 0
  statement_id  = "AllowAgentInvokePDF"
  action        = "lambda:InvokeFunctionUrl"
  function_name = aws_lambda_function.pdf[0].function_name
  principal     = aws_iam_role.agent_lambda_execution.arn
  source_arn    = aws_lambda_function.agent.arn
}

# ─── CloudWatch Alarm: High Error Rate ────────────────────────────────────────

resource "aws_cloudwatch_metric_alarm" "pdf_lambda_errors" {
  alarm_name          = "${local.lambda_function_name}-pdf-errors"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = 300
  statistic           = "Sum"
  threshold           = 5
  treat_missing_data  = "notBreaching"

  alarm_actions = []

  dimensions = {
    FunctionName = length(aws_lambda_function.pdf) > 0 ? aws_lambda_function.pdf[0].function_name : "${local.lambda_function_name}-pdf"
  }
}
