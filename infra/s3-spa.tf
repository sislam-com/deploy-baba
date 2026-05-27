# S3 bucket for SPA build artifacts keyed by git SHA
resource "aws_s3_bucket" "spa" {
  bucket = "deploy-baba-${var.environment}-spa-${data.aws_caller_identity.current.account_id}"

  tags = {
    Name = "deploy-baba-${var.environment}-spa"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "spa" {
  bucket = aws_s3_bucket.spa.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "spa" {
  bucket                  = aws_s3_bucket.spa.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Retain only the 5 most recent SHA prefixes; prune older builds
resource "aws_s3_bucket_lifecycle_configuration" "spa" {
  bucket = aws_s3_bucket.spa.id

  rule {
    id     = "prune-old-sha-prefixes"
    status = "Enabled"

    filter {}

    expiration {
      days = 14
    }

    noncurrent_version_expiration {
      noncurrent_days = 1
    }
  }
}

# Allow CloudFront OAC to read SPA assets from this bucket (prod only)
resource "aws_s3_bucket_policy" "spa_cloudfront" {
  count  = var.environment == "prod" ? 1 : 0
  bucket = aws_s3_bucket.spa.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowCloudFrontOAC"
        Effect = "Allow"
        Principal = {
          Service = "cloudfront.amazonaws.com"
        }
        Action   = "s3:GetObject"
        Resource = "${aws_s3_bucket.spa.arn}/*"
        Condition = {
          StringEquals = {
            "AWS:SourceArn" = aws_cloudfront_distribution.main[0].arn
          }
        }
      }
    ]
  })
}

# IAM: allow CI to upload SPA assets to this bucket (prod-only singleton roles)
resource "aws_iam_role_policy" "ci_s3_spa_dev" {
  count = local.is_prod ? 1 : 0
  name  = "${var.project_name}-ci-s3-spa-dev-policy"
  role  = aws_iam_role.ci_deploy_dev[0].id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid      = "S3SpaReadWrite"
        Effect   = "Allow"
        Action   = ["s3:PutObject", "s3:DeleteObject", "s3:ListBucket", "s3:GetObject"]
        Resource = [aws_s3_bucket.spa.arn, "${aws_s3_bucket.spa.arn}/*"]
      }
    ]
  })
}

resource "aws_iam_role_policy" "ci_s3_spa_prod" {
  count = local.is_prod ? 1 : 0
  name  = "${var.project_name}-ci-s3-spa-prod-policy"
  role  = aws_iam_role.ci_deploy_prod[0].id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid      = "S3SpaReadWrite"
        Effect   = "Allow"
        Action   = ["s3:PutObject", "s3:DeleteObject", "s3:ListBucket", "s3:GetObject"]
        Resource = [aws_s3_bucket.spa.arn, "${aws_s3_bucket.spa.arn}/*"]
      }
    ]
  })
}

output "spa_bucket_name" {
  description = "S3 bucket that holds SPA build artifacts"
  value       = aws_s3_bucket.spa.id
}
