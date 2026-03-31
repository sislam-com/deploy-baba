# SSM Parameter for database path
resource "aws_ssm_parameter" "db_path" {
  name        = "/${var.project_name}/${var.environment}/db-path"
  description = "Path to SQLite database in EFS"
  type        = "String"
  value       = "/mnt/db/baba.db"

  tags = {
    Name = "${local.lambda_function_name}-db-path"
  }
}

# SSM Parameter for backup bucket name
resource "aws_ssm_parameter" "backup_bucket" {
  name        = "/${var.project_name}/${var.environment}/backup-bucket"
  description = "S3 bucket for database backups"
  type        = "String"
  value       = aws_s3_bucket.backups.id

  tags = {
    Name = "${local.lambda_function_name}-backup-bucket"
  }
}

# SSM Parameter for backup prefix
resource "aws_ssm_parameter" "backup_prefix" {
  name        = "/${var.project_name}/${var.environment}/backup-prefix"
  description = "S3 prefix for database backups"
  type        = "String"
  value       = "backups/"

  tags = {
    Name = "${local.lambda_function_name}-backup-prefix"
  }
}

# Cognito SSM parameters
resource "aws_ssm_parameter" "cognito_pool_id" {
  name        = "/${var.project_name}/${var.environment}/cognito-pool-id"
  description = "Cognito User Pool ID"
  type        = "String"
  value       = aws_cognito_user_pool.baba.id

  tags = {
    Name = "${local.lambda_function_name}-cognito-pool-id"
  }
}

resource "aws_ssm_parameter" "cognito_client_id" {
  name        = "/${var.project_name}/${var.environment}/cognito-client-id"
  description = "Cognito app client ID"
  type        = "String"
  value       = aws_cognito_user_pool_client.baba_web.id

  tags = {
    Name = "${local.lambda_function_name}-cognito-client-id"
  }
}

resource "aws_ssm_parameter" "cognito_domain" {
  name        = "/${var.project_name}/${var.environment}/cognito-domain"
  description = "Cognito hosted UI domain (FQDN)"
  type        = "String"
  value       = "${aws_cognito_user_pool_domain.baba.domain}.auth.${var.region}.amazoncognito.com"

  tags = {
    Name = "${local.lambda_function_name}-cognito-domain"
  }
}
