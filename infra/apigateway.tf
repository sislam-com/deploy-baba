# ─── API Gateway HTTP API for POST /api/* that need OAC-bypass ─────────────────
#
# CloudFront OAC (AWS_IAM signing) rejects POST bodies because CloudFront sends
# UNSIGNED-PAYLOAD instead of the actual body hash (DRL-FUA-3, W-AUTH.POST-FIX).
# This API Gateway HTTP API sits as a second CloudFront origin that receives POST
# requests WITHOUT OAC signing, forwarding them to the UI Lambda as a proxy.
#
# Routes served here: POST /api/contact, POST /api/ask.
# All other paths use the Lambda Function URL origin with OAC as usual.

# HTTP API (V2) — simpler + cheaper than REST API, supports Lambda proxy
resource "aws_apigatewayv2_api" "contact" {
  name          = "${var.project_name}-contact-api"
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins = ["https://${var.domain_name}", "https://www.${var.domain_name}"]
    allow_methods = ["POST", "OPTIONS"]
    allow_headers = ["Content-Type"]
    max_age       = 300
  }

  tags = {
    Name = "${var.project_name}-contact-api"
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

# ─── Locals ────────────────────────────────────────────────────────────────────

locals {
  # Strip "https://" and trailing "/" from API Gateway endpoint URL
  apigw_contact_domain = replace(replace(aws_apigatewayv2_api.contact.api_endpoint, "https://", ""), "/", "")
}
