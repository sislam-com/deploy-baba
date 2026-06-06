# ─── API Gateway HTTP API for POST /api/* that need OAC-bypass ─────────────────
#
# CloudFront OAC (AWS_IAM signing) rejects POST bodies because CloudFront sends
# UNSIGNED-PAYLOAD instead of the actual body hash (DRL-FUA-3, W-AUTH.POST-FIX).
# This API Gateway HTTP API sits as a second CloudFront origin that receives POST
# requests WITHOUT OAC signing, forwarding them to the UI Lambda as a proxy.
#
# Routes served here: POST /api/contact, POST /api/ask, POST /mcp, GET /mcp/health.
# All other paths use the Lambda Function URL origin with OAC as usual.

# HTTP API (V2) — simpler + cheaper than REST API, supports Lambda proxy
resource "aws_apigatewayv2_api" "contact" {
  name          = "${local.lambda_function_name}-contact-api"
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins = ["https://${var.domain_name}", "https://www.${var.domain_name}", "https://dev.${var.domain_name}"]
    allow_methods = ["GET", "POST", "OPTIONS"]
    allow_headers = ["Authorization", "Content-Type"]
    max_age       = 300
  }

  tags = {
    Name = "${local.lambda_function_name}-contact-api"
  }
}

# Lambda proxy integration → UI Lambda
resource "aws_apigatewayv2_integration" "contact" {
  api_id                 = aws_apigatewayv2_api.contact.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.baba.invoke_arn
  payload_format_version = "2.0"
}

# Route: POST /api/contact
resource "aws_apigatewayv2_route" "contact_post" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/contact"
  target    = "integrations/${aws_apigatewayv2_integration.contact.id}"
}

# Route: POST /api/ask (same OAC-bypass workaround as /api/contact)
resource "aws_apigatewayv2_route" "ask_post" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/ask"
  target    = "integrations/${aws_apigatewayv2_integration.contact.id}"
}

# Lambda proxy integration → Auth Lambda (Cognito IDP proxy, no VPC)
resource "aws_apigatewayv2_integration" "auth" {
  api_id                 = aws_apigatewayv2_api.contact.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.auth.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "auth_signin" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/auth/signin"
  target    = "integrations/${aws_apigatewayv2_integration.auth.id}"
}

resource "aws_apigatewayv2_route" "auth_forgot_password" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/auth/forgot-password"
  target    = "integrations/${aws_apigatewayv2_integration.auth.id}"
}

resource "aws_apigatewayv2_route" "auth_confirm_forgot_password" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/auth/confirm-forgot-password"
  target    = "integrations/${aws_apigatewayv2_integration.auth.id}"
}

resource "aws_apigatewayv2_route" "auth_respond_to_challenge" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/auth/respond-to-challenge"
  target    = "integrations/${aws_apigatewayv2_integration.auth.id}"
}

resource "aws_apigatewayv2_route" "auth_signout" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/auth/signout"
  target    = "integrations/${aws_apigatewayv2_integration.auth.id}"
}

resource "aws_apigatewayv2_route" "auth_health" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "GET /health"
  target    = "integrations/${aws_apigatewayv2_integration.auth.id}"
}

# GET /api/auth/me → UI Lambda (not auth Lambda) — needs cookie middleware
resource "aws_apigatewayv2_route" "auth_me" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "GET /api/auth/me"
  target    = "integrations/${aws_apigatewayv2_integration.contact.id}"
}

# Allow API Gateway to invoke the Auth Lambda
resource "aws_lambda_permission" "apigw_auth" {
  statement_id  = "AllowAPIGatewayAuthInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.auth.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.contact.execution_arn}/*/*/api/auth/*"
}

# Lambda proxy integration -> private MCP gateway Lambda
resource "aws_apigatewayv2_integration" "mcp_gateway" {
  api_id                 = aws_apigatewayv2_api.contact.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.mcp_gateway.invoke_arn
  payload_format_version = "2.0"
}

# Route: POST /mcp
resource "aws_apigatewayv2_route" "mcp_post" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /mcp"
  target    = "integrations/${aws_apigatewayv2_integration.mcp_gateway.id}"
}

# Route: GET /mcp/health
resource "aws_apigatewayv2_route" "mcp_health_get" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "GET /mcp/health"
  target    = "integrations/${aws_apigatewayv2_integration.mcp_gateway.id}"
}

# $default stage with auto-deploy
resource "aws_apigatewayv2_stage" "contact" {
  api_id      = aws_apigatewayv2_api.contact.id
  name        = "$default"
  auto_deploy = true

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.apigw_contact.arn
    format          = jsonencode({ requestId = "$context.requestId", ip = "$context.identity.sourceIp", routeKey = "$context.routeKey", status = "$context.status", responseLength = "$context.responseLength", integrationError = "$context.integrationErrorMessage" })
  }

  tags = {
    Name = "${var.project_name}-contact-api-stage"
  }
}

# CloudWatch Log Group for API Gateway access logs
resource "aws_cloudwatch_log_group" "apigw_contact" {
  name              = "/aws/apigateway/${var.project_name}-contact-api"
  retention_in_days = var.logs_retention_days

  tags = {
    Name = "${var.project_name}-contact-api-logs"
  }
}

# Allow API Gateway to invoke the UI Lambda (covers all POST /api/* routes)
resource "aws_lambda_permission" "apigw_contact" {
  statement_id  = "AllowAPIGatewayPostInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.baba.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.contact.execution_arn}/*/*/api/*"
}

# Allow API Gateway to invoke the private MCP gateway Lambda
resource "aws_lambda_permission" "apigw_mcp_gateway" {
  statement_id  = "AllowAPIGatewayMcpInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.mcp_gateway.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.contact.execution_arn}/*/*/mcp*"
}

# ─── Agent Lambda Integration ──────────────────────────────────────────────────
# Routes POST /api/v1/agent/* through API Gateway for OAC body hash workaround

resource "aws_apigatewayv2_integration" "agent" {
  api_id                 = aws_apigatewayv2_api.contact.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.agent.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "agent_cover_letter" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/v1/agent/cover-letter"
  target    = "integrations/${aws_apigatewayv2_integration.agent.id}"
}

resource "aws_apigatewayv2_route" "agent_cover_letter_stream" {
  api_id    = aws_apigatewayv2_api.contact.id
  route_key = "POST /api/v1/agent/cover-letter/stream"
  target    = "integrations/${aws_apigatewayv2_integration.agent.id}"
}

resource "aws_lambda_permission" "apigw_agent" {
  statement_id  = "AllowAPIGatewayAgentInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.agent.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.contact.execution_arn}/*/*/api/v1/agent/*"
}

# ─── Locals ────────────────────────────────────────────────────────────────────

locals {
  # Strip "https://" and trailing "/" from API Gateway endpoint URL
  apigw_contact_domain = replace(replace(aws_apigatewayv2_api.contact.api_endpoint, "https://", ""), "/", "")
}
