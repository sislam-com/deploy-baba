output "function_url" {
  description = "Lambda Function URL for accessing the portfolio site"
  value       = aws_lambda_function_url.baba.function_url
}

output "lambda_arn" {
  description = "ARN of the Lambda function"
  value       = aws_lambda_function.baba.arn
}

output "lambda_function_name" {
  description = "Name of the Lambda function"
  value       = aws_lambda_function.baba.function_name
}

output "s3_backup_bucket" {
  description = "S3 bucket for database backups"
  value       = aws_s3_bucket.backups.id
}

output "efs_file_system_id" {
  description = "EFS file system ID for database storage"
  value       = aws_efs_file_system.baba_db.id
}

output "efs_mount_targets" {
  description = "EFS mount target IDs"
  value       = aws_efs_mount_target.baba_db[*].id
}

output "cloudwatch_log_group" {
  description = "CloudWatch Logs group for Lambda"
  value       = aws_cloudwatch_log_group.lambda.name
}

output "eventbridge_rule_name" {
  description = "EventBridge rule name for backup schedule"
  value       = aws_cloudwatch_event_rule.backup_schedule.name
}

output "backup_schedule" {
  description = "Backup schedule expression"
  value       = var.backup_schedule
}

output "cloudfront_distribution_id" {
  description = "CloudFront distribution ID"
  value       = aws_cloudfront_distribution.main.id
}

output "cloudfront_domain_name" {
  description = "CloudFront distribution domain name"
  value       = aws_cloudfront_distribution.main.domain_name
}

output "site_url" {
  description = "Public URL of the portfolio site"
  value       = "https://${var.domain_name}"
}

output "cognito_user_pool_id" {
  description = "Cognito User Pool ID"
  value       = aws_cognito_user_pool.baba.id
}

output "cognito_client_id" {
  description = "Cognito app client ID"
  value       = aws_cognito_user_pool_client.baba_web.id
}

output "cognito_domain" {
  description = "Cognito hosted UI domain (FQDN)"
  value       = "${aws_cognito_user_pool_domain.baba.domain}.auth.${var.region}.amazoncognito.com"
}
