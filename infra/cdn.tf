# Variables for CDN / custom domain
variable "domain_name" {
  description = "Primary domain name for the portfolio site"
  type        = string
  default     = "sislam.com"
}

variable "acm_certificate_arn" {
  description = "ACM certificate ARN (must cover domain_name and *.domain_name, in us-east-1)"
  type        = string
  default     = "arn:aws:acm:us-east-1:062513063428:certificate/431fdf38-6e82-42a9-b693-31abb64aabbb"
}

# ─── Data sources ──────────────────────────────────────────────────────────────

data "aws_route53_zone" "main" {
  name         = var.domain_name
  private_zone = false
}

data "aws_cloudfront_cache_policy" "caching_disabled" {
  name = "Managed-CachingDisabled"
}

data "aws_cloudfront_cache_policy" "caching_optimized" {
  name = "Managed-CachingOptimized"
}

data "aws_cloudfront_origin_request_policy" "all_viewer_except_host" {
  name = "Managed-AllViewerExceptHostHeader"
}

# ─── Locals ────────────────────────────────────────────────────────────────────

locals {
  # Strip "https://" prefix and trailing "/" from the Lambda Function URL
  lambda_origin_domain = replace(replace(aws_lambda_function_url.baba.function_url, "https://", ""), "/", "")
}

# ─── CloudFront OAC for Lambda ─────────────────────────────────────────────────

resource "aws_cloudfront_origin_access_control" "lambda" {
  name                              = "deploy-baba-lambda-oac"
  origin_access_control_origin_type = "lambda"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# ─── CloudFront OAC for S3 assets ──────────────────────────────────────────────

resource "aws_cloudfront_origin_access_control" "assets" {
  name                              = "deploy-baba-assets-oac"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# ─── CloudFront Distribution ───────────────────────────────────────────────────

resource "aws_cloudfront_distribution" "main" {
  enabled         = true
  is_ipv6_enabled = true
  price_class     = "PriceClass_100"
  aliases         = [var.domain_name, "www.${var.domain_name}"]
  comment         = "Portfolio site — ${var.domain_name}"

  origin {
    domain_name              = local.lambda_origin_domain
    origin_id                = "lambda-function-url"
    origin_access_control_id = aws_cloudfront_origin_access_control.lambda.id

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  origin {
    domain_name              = aws_s3_bucket.assets.bucket_regional_domain_name
    origin_id                = "s3-assets"
    origin_access_control_id = aws_cloudfront_origin_access_control.assets.id
  }

  # Cache behavior for /resume/* — served from S3 assets bucket
  ordered_cache_behavior {
    path_pattern           = "/resume/*"
    target_origin_id       = "s3-assets"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD"]
    cached_methods  = ["GET", "HEAD"]

    # CachingOptimized — honors Cache-Control headers set during upload
    cache_policy_id = data.aws_cloudfront_cache_policy.caching_optimized.id
  }

  default_cache_behavior {
    target_origin_id       = "lambda-function-url"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"]
    cached_methods  = ["GET", "HEAD"]

    # CachingDisabled — every request forwarded to origin (dynamic app)
    cache_policy_id = data.aws_cloudfront_cache_policy.caching_disabled.id

    # AllViewerExceptHostHeader — forwards all headers/cookies/query strings
    # but replaces Host with the Lambda origin domain so Lambda accepts the request
    origin_request_policy_id = data.aws_cloudfront_origin_request_policy.all_viewer_except_host.id
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn      = var.acm_certificate_arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }

  tags = {
    Name = "${var.project_name}-cdn"
  }
}

# ─── Route53 Records ───────────────────────────────────────────────────────────

resource "aws_route53_record" "apex_a" {
  zone_id         = data.aws_route53_zone.main.zone_id
  name            = var.domain_name
  type            = "A"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main.domain_name
    zone_id                = aws_cloudfront_distribution.main.hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "apex_aaaa" {
  zone_id         = data.aws_route53_zone.main.zone_id
  name            = var.domain_name
  type            = "AAAA"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main.domain_name
    zone_id                = aws_cloudfront_distribution.main.hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "www_a" {
  zone_id         = data.aws_route53_zone.main.zone_id
  name            = "www.${var.domain_name}"
  type            = "A"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main.domain_name
    zone_id                = aws_cloudfront_distribution.main.hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "www_aaaa" {
  zone_id         = data.aws_route53_zone.main.zone_id
  name            = "www.${var.domain_name}"
  type            = "AAAA"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main.domain_name
    zone_id                = aws_cloudfront_distribution.main.hosted_zone_id
    evaluate_target_health = false
  }
}
