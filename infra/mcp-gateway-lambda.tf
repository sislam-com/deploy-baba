# Private MCP gateway Lambda.
#
# This Lambda is a thin authenticated HTTP adapter around the workspace-owned
# `crates/mcp-rs` crate. It is intentionally non-VPC and read-only: the Lambda
# package contains a static context bundle plus `mcp-rs.toml`.

resource "aws_cloudwatch_log_group" "mcp_gateway" {
  name              = "/aws/lambda/${local.lambda_function_name}-mcp-gateway"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${local.lambda_function_name}-mcp-gateway-logs"
  }
}

resource "aws_lambda_function" "mcp_gateway" {
  filename      = var.mcp_gateway_lambda_code_path
  function_name = "${local.lambda_function_name}-mcp-gateway"
  role          = aws_iam_role.mcp_gateway_execution.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  memory_size   = 256
  timeout       = 10
  architectures = ["arm64"]

  source_code_hash = fileexists(var.mcp_gateway_lambda_code_path) ? filebase64sha256(var.mcp_gateway_lambda_code_path) : null

  reserved_concurrent_executions = 2

  environment {
    variables = {
      RUST_LOG          = "info"
      MCP_RS_CONFIG     = "/var/task/mcp-rs.toml"
      COGNITO_POOL_ID   = aws_cognito_user_pool.baba.id
      COGNITO_CLIENT_ID = aws_cognito_user_pool_client.baba_web.id
      COGNITO_REGION    = var.region
      COGNITO_JWKS      = data.http.cognito_jwks.response_body
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.mcp_gateway,
    aws_iam_role_policy_attachment.mcp_gateway_logs,
  ]

  tags = {
    Name = "${local.lambda_function_name}-mcp-gateway"
  }
}

resource "aws_iam_role" "mcp_gateway_execution" {
  name = "${local.lambda_function_name}-mcp-gateway-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Name = "${local.lambda_function_name}-mcp-gateway-execution-role"
  }
}

resource "aws_iam_role_policy_attachment" "mcp_gateway_logs" {
  role       = aws_iam_role.mcp_gateway_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}
