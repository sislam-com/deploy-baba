terraform {
  required_version = ">= 1.6" # OpenTofu first stable release

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    http = {
      source  = "hashicorp/http"
      version = "~> 3.0"
    }
  }

  # Backend configuration for remote state management
  # Ensure the S3 bucket and DynamoDB table exist before applying
  backend "s3" {
    bucket         = "deploy-baba-tfstate"
    key            = "deploy-baba/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-lock"
  }
}

# AWS Provider configuration
provider "aws" {
  region = var.region

  default_tags {
    tags = local.common_tags
  }
}

# Local values for common tags and configuration
locals {
  common_tags = {
    Project     = var.project_name
    Environment = var.environment
    Terraform   = "true"
    ManagedBy   = "OpenTofu"
  }

  lambda_function_name = "${var.project_name}-${var.environment}"
}
