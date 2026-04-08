# ─── Email Lambda (no VPC, internal worker) ───────────────────────────────────
#
# Invoked by the UI Lambda via SDK (not via public HTTP).
# The UI Lambda handles POST /api/contact, rate-limits, and delegates here.
# No Function URL — the Lambda VPC endpoint (vpc-endpoints.tf) lets the
# VPC-bound UI Lambda reach the Lambda API internally.

resource "aws_lambda_function" "email" {
  filename         = var.email_lambda_code_path
  function_name    = "${var.project_name}-email"
  role             = aws_iam_role.email_lambda_execution.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  memory_size      = 128
  timeout          = 10
  architectures    = ["arm64"]

  # Hard cap on concurrent invocations — limits cost exposure and prevents
  # a flood of submissions from exhausting account Lambda concurrency.
  reserved_concurrent_executions = 5

  environment {
    variables = {
      RUST_LOG             = "info"
      SES_FROM_EMAIL       = "noreply@${local.ses_domain}"
      SES_ACK_FROM_EMAIL   = "it@sislam.com"
      CONTACT_TO_EMAIL     = var.contact_email
      AWS_SES_REGION       = var.region
      ALLOWED_ORIGIN       = "https://${var.domain_name}"
    }
  }

  # NO vpc_config — this Lambda has direct internet access for SES calls.
  # The main Lambda (deploy-baba-ui) is VPC-attached for EFS.

  depends_on = [aws_iam_role_policy_attachment.email_lambda_logs]
}

# ─── IAM Role ─────────────────────────────────────────────────────────────────

resource "aws_iam_role" "email_lambda_execution" {
  name = "${var.project_name}-email-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "email_lambda_logs" {
  role       = aws_iam_role.email_lambda_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "email_lambda_ses" {
  name = "${var.project_name}-email-ses-policy"
  role = aws_iam_role.email_lambda_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = ["ses:SendEmail", "ses:SendRawEmail"]
      # SES v2 checks IAM on both FROM identity (mail.sislam.com) and the exact
      # TO identity ARN. The domain identity `shantopagla.com` does NOT cover
      # individual address checks — need a wildcard for *@shantopagla.com.
      Resource = [
        # FROM identity for admin notifications (domain identity)
        "arn:aws:ses:${var.region}:${data.aws_caller_identity.current.account_id}:identity/${local.ses_domain}",
        # TO identity for admin notifications (verified email in same account)
        "arn:aws:ses:${var.region}:${data.aws_caller_identity.current.account_id}:identity/*@shantopagla.com",
        # FROM identity for acknowledgement emails (separately-verified email identity)
        "arn:aws:ses:${var.region}:${data.aws_caller_identity.current.account_id}:identity/it@sislam.com",
      ]
    }]
  })
}

# ─── CloudWatch Alarm: High Invocations ───────────────────────────────────────
# Alert if the email Lambda receives more than 100 invocations per hour.
# Add an SNS topic ARN to alarm_actions to get notified.

resource "aws_cloudwatch_metric_alarm" "email_lambda_high_invocations" {
  alarm_name          = "${var.project_name}-email-high-invocations"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "Invocations"
  namespace           = "AWS/Lambda"
  period              = 3600
  statistic           = "Sum"
  threshold           = 100
  alarm_description   = "Email Lambda invocations exceeded 100/hour — possible abuse"
  treat_missing_data  = "notBreaching"
  alarm_actions       = []

  dimensions = {
    FunctionName = aws_lambda_function.email.function_name
  }
}
