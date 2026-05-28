# IAM role for Lambda execution
resource "aws_iam_role" "lambda_execution" {
  name = "${local.lambda_function_name}-execution-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Name = "${local.lambda_function_name}-execution-role"
  }
}

# Managed policy: CloudWatch Logs write access
resource "aws_iam_role_policy_attachment" "lambda_logs" {
  role       = aws_iam_role.lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/CloudWatchLogsFullAccess"
}

# Managed policy: VPC execution (for EFS access)
resource "aws_iam_role_policy_attachment" "lambda_vpc" {
  role       = aws_iam_role.lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

# Inline policy: EFS access
resource "aws_iam_role_policy" "lambda_efs" {
  name = "${local.lambda_function_name}-efs-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "EFSAccess"
        Effect = "Allow"
        Action = [
          "elasticfilesystem:ClientMount",
          "elasticfilesystem:ClientWrite",
          "elasticfilesystem:ClientRootAccess"
        ]
        Resource = aws_efs_file_system.baba_db.arn
      }
    ]
  })
}

# Inline policy: S3 backup bucket access
resource "aws_iam_role_policy" "lambda_s3" {
  name = "${local.lambda_function_name}-s3-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
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
      }
    ]
  })
}

# Inline policy: invoke email Lambda
resource "aws_iam_role_policy" "lambda_invoke_email" {
  name = "${local.lambda_function_name}-invoke-email-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid      = "InvokeEmailLambda"
      Effect   = "Allow"
      Action   = "lambda:InvokeFunction"
      Resource = aws_lambda_function.email.arn
    }]
  })
}

# Inline policy: invoke LLM-proxy Lambda (non-VPC, reaches api.anthropic.com)
resource "aws_iam_role_policy" "lambda_invoke_llm_proxy" {
  name = "${local.lambda_function_name}-invoke-llm-proxy-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid      = "InvokeLlmProxyLambda"
      Effect   = "Allow"
      Action   = "lambda:InvokeFunction"
      Resource = aws_lambda_function.llm_proxy.arn
    }]
  })
}

# Inline policy: invoke portfolio Lambda
resource "aws_iam_role_policy" "lambda_invoke_portfolio" {
  name = "${local.lambda_function_name}-invoke-portfolio-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid      = "InvokePortfolioLambda"
      Effect   = "Allow"
      Action   = "lambda:InvokeFunction"
      Resource = aws_lambda_function.portfolio.arn
    }]
  })
}

# Inline policy: invoke admin Lambda
resource "aws_iam_role_policy" "lambda_invoke_admin" {
  name = "${local.lambda_function_name}-invoke-admin-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid      = "InvokeAdminLambda"
      Effect   = "Allow"
      Action   = "lambda:InvokeFunction"
      Resource = aws_lambda_function.admin.arn
    }]
  })
}

# Inline policy: invoke contact Lambda
resource "aws_iam_role_policy" "lambda_invoke_contact" {
  name = "${local.lambda_function_name}-invoke-contact-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid      = "InvokeContactLambda"
      Effect   = "Allow"
      Action   = "lambda:InvokeFunction"
      Resource = aws_lambda_function.contact.arn
    }]
  })
}

# Inline policy: invoke RAG Lambda
resource "aws_iam_role_policy" "lambda_invoke_rag" {
  name = "${local.lambda_function_name}-invoke-rag-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid      = "InvokeRagLambda"
      Effect   = "Allow"
      Action   = "lambda:InvokeFunction"
      Resource = aws_lambda_function.rag.arn
    }]
  })
}

# Inline policy: SSM parameter read access
resource "aws_iam_role_policy" "lambda_ssm" {
  name = "${local.lambda_function_name}-ssm-policy"
  role = aws_iam_role.lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "SSMParameterAccess"
        Effect = "Allow"
        Action = [
          "ssm:GetParameter",
          "ssm:GetParameters"
        ]
        Resource = [
          "arn:aws:ssm:${var.region}:${data.aws_caller_identity.current.account_id}:parameter/${var.project_name}/${var.environment}/*"
        ]
      }
    ]
  })
}
