# S3 Bucket for static assets (resume files, etc.)
# Served exclusively via CloudFront OAC — no public access.

resource "aws_s3_bucket" "assets" {
  bucket = "${local.lambda_function_name}-assets-${data.aws_caller_identity.current.account_id}"

  tags = {
    Name = "${local.lambda_function_name}-assets"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "assets" {
  bucket = aws_s3_bucket.assets.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "assets" {
  bucket = aws_s3_bucket.assets.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Grant CloudFront OAC read access to the assets bucket (prod only — dev
# buckets are accessed directly during promote, not via CloudFront)
resource "aws_s3_bucket_policy" "assets_cloudfront" {
  count  = var.environment == "prod" ? 1 : 0
  bucket = aws_s3_bucket.assets.id

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
        Resource = "${aws_s3_bucket.assets.arn}/*"
        Condition = {
          StringEquals = {
            "AWS:SourceArn" = aws_cloudfront_distribution.main[0].arn
          }
        }
      }
    ]
  })
}

output "assets_bucket_name" {
  description = "S3 assets bucket name"
  value       = aws_s3_bucket.assets.id
}
