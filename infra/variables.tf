variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "deploy-baba"

  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.project_name))
    error_message = "Project name must contain only lowercase letters, numbers, and hyphens."
  }
}

variable "region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Deployment environment (prod, staging, dev)"
  type        = string
  default     = "prod"

  validation {
    condition     = contains(["prod", "staging", "dev"], var.environment)
    error_message = "Environment must be one of: prod, staging, dev."
  }
}

variable "lambda_memory" {
  description = "Lambda function memory in MB"
  type        = number
  default     = 256

  validation {
    condition     = var.lambda_memory >= 128 && var.lambda_memory <= 10240
    error_message = "Lambda memory must be between 128 and 10240 MB."
  }
}

variable "lambda_timeout" {
  description = "Lambda function timeout in seconds"
  type        = number
  default     = 30

  validation {
    condition     = var.lambda_timeout >= 1 && var.lambda_timeout <= 900
    error_message = "Lambda timeout must be between 1 and 900 seconds."
  }
}

variable "backup_schedule" {
  description = "EventBridge rule schedule expression for backups (e.g., 'rate(1 day)' or 'cron(0 2 * * ? *)')"
  type        = string
  default     = "rate(1 day)"
}

variable "backup_retain_versions" {
  description = "Number of backup versions to retain in S3"
  type        = number
  default     = 7

  validation {
    condition     = var.backup_retain_versions >= 1 && var.backup_retain_versions <= 365
    error_message = "Backup retention must be between 1 and 365 versions."
  }
}

variable "lambda_code_path" {
  description = "Path to the Lambda function code (zip file)"
  type        = string
  default     = "./build/lambda.zip"
}

variable "logs_retention_days" {
  description = "CloudWatch Logs retention period in days"
  type        = number
  default     = 14
}

variable "admin_email" {
  description = "Email address for the Cognito admin user (baba-admin)"
  type        = string
  default     = "it@shantopagla.com"
}

variable "cognito_temp_password" {
  description = "Temporary password for baba-admin (must be changed on first login). Ignored after initial user creation (lifecycle.ignore_changes)."
  type        = string
  sensitive   = true
  default     = "unused-after-first-apply"
}
