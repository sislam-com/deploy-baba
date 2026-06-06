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

variable "contact_email" {
  description = "Email address that receives contact form submissions"
  type        = string
  default     = "contact-sislam@shantopagla.com"
}

variable "email_lambda_code_path" {
  description = "Path to the email Lambda zip file (built by just email-build)"
  type        = string
  default     = "./build/email-lambda.zip"
}

variable "llm_proxy_lambda_code_path" {
  description = "Path to the LLM-proxy Lambda zip file (built by just llm-proxy-build)"
  type        = string
  default     = "./build/llm-proxy-lambda.zip"
}

variable "mcp_gateway_lambda_code_path" {
  description = "Path to the private MCP gateway Lambda zip file (built by just mcp-cloud-build)"
  type        = string
  default     = "./build/mcp-gateway-lambda.zip"
}

variable "auth_lambda_code_path" {
  description = "Path to the auth Lambda zip file (built by just auth-build)"
  type        = string
  default     = "./build/auth-lambda.zip"
}

variable "agent_lambda_code_path" {
  description = "Path to the agent Lambda zip file (built by just agent-build)"
  type        = string
  default     = "./build/agent-lambda.zip"
}

variable "portfolio_lambda_code_path" {
  description = "Path to the portfolio Lambda zip file"
  type        = string
  default     = "./build/portfolio-lambda.zip"
}

variable "admin_lambda_code_path" {
  description = "Path to the admin Lambda zip file"
  type        = string
  default     = "./build/admin-lambda.zip"
}

variable "contact_lambda_code_path" {
  description = "Path to the contact Lambda zip file"
  type        = string
  default     = "./build/contact-lambda.zip"
}

variable "rag_lambda_code_path" {
  description = "Path to the RAG Lambda zip file"
  type        = string
  default     = "./build/rag-lambda.zip"
}

variable "pdf_lambda_image_uri" {
  description = "ECR image URI for PDF Lambda (format: ACCOUNT.dkr.ecr.REGION.amazonaws.com/REPO:TAG)"
  type        = string
  default     = "" # Set via justfile or tfvars after building image
}
