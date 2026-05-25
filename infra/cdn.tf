# Variables for CDN / custom domain
variable "domain_name" {
  description = "Primary domain name for the portfolio site"
  type        = string
  default     = "sislam.com"
}

locals {
  is_prod_cdn = var.environment == "prod"
}

# ─── Data sources ──────────────────────────────────────────────────────────────

data "aws_route53_zone" "main" {
  count        = local.is_prod_cdn ? 1 : 0
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

# ─── Custom origin request policy for Lambda ───────────────────────────────────
# Forward NO viewer headers: OAC SigV4 for Lambda Function URL (AWS_IAM) requires
# signed payload hash for POST/PUT but CloudFront always uses UNSIGNED-PAYLOAD,
# causing InvalidSignatureException. Forwarding zero viewer headers minimizes the
# signed header set, keeping GET working. POST/PUT must be avoided through this
# CloudFront→Function URL path (see W-AUTH.POST-FIX, DRL-2026-03-27-function-url-auth).
# Cookies forwarded separately via cookies_config (session auth works).
# x-forwarded-for always added by CloudFront regardless of this policy.
resource "aws_cloudfront_origin_request_policy" "lambda_oac" {
  count = local.is_prod_cdn ? 1 : 0
  name  = "deploy-baba-lambda-oac-policy"

  headers_config {
    header_behavior = "none"
  }

  cookies_config {
    cookie_behavior = "all"
  }

  query_strings_config {
    query_string_behavior = "all"
  }
}

# ─── Locals ────────────────────────────────────────────────────────────────────

locals {
  # Strip "https://" prefix and trailing "/" from the Lambda Function URL
  lambda_origin_domain      = replace(replace(aws_lambda_function_url.baba.function_url, "https://", ""), "/", "")
  auth_lambda_origin_domain = replace(replace(aws_lambda_function_url.auth.function_url, "https://", ""), "/", "")
}

# ─── CloudFront OAC for Lambda ─────────────────────────────────────────────────

resource "aws_cloudfront_origin_access_control" "lambda" {
  count                             = local.is_prod_cdn ? 1 : 0
  name                              = "deploy-baba-lambda-oac"
  origin_access_control_origin_type = "lambda"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# ─── CloudFront OAC for S3 assets ──────────────────────────────────────────────

resource "aws_cloudfront_origin_access_control" "assets" {
  count                             = local.is_prod_cdn ? 1 : 0
  name                              = "deploy-baba-assets-oac"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# ─── CloudFront OAC for S3 SPA bucket ─────────────────────────────────────────

resource "aws_cloudfront_origin_access_control" "spa" {
  count                             = local.is_prod_cdn ? 1 : 0
  name                              = "deploy-baba-spa-oac"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# ─── CloudFront Distribution ───────────────────────────────────────────────────

resource "aws_cloudfront_distribution" "main" {
  count           = local.is_prod_cdn ? 1 : 0
  enabled         = true
  is_ipv6_enabled = true
  price_class     = "PriceClass_100"
  aliases         = [var.domain_name, "www.${var.domain_name}", "dev.${var.domain_name}"]
  comment         = "Portfolio site — ${var.domain_name}"

  origin {
    domain_name              = local.lambda_origin_domain
    origin_id                = "lambda-function-url"
    origin_access_control_id = aws_cloudfront_origin_access_control.lambda[0].id

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
    origin_access_control_id = aws_cloudfront_origin_access_control.assets[0].id
  }

  origin {
    domain_name              = aws_s3_bucket.spa.bucket_regional_domain_name
    origin_id                = "s3-spa"
    origin_access_control_id = aws_cloudfront_origin_access_control.spa[0].id
  }

  # Auth Lambda origin — public Function URL for SPA login flow
  origin {
    domain_name              = local.auth_lambda_origin_domain
    origin_id                = "auth-lambda-function-url"
    origin_access_control_id = aws_cloudfront_origin_access_control.lambda[0].id

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  # API Gateway origin for POST /api/contact (no OAC — body hash works correctly)
  origin {
    domain_name = local.apigw_contact_domain
    origin_id   = "apigw-contact"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
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

  # Auth service — MUST appear before the general /api/* behavior.
  ordered_cache_behavior {
    path_pattern           = "/api/auth/*"
    target_origin_id       = "auth-lambda-function-url"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.lambda_oac[0].id
  }

  # Cache behaviors for POST /api/* routed via API Gateway (no OAC — body hash works correctly)
  # Must appear before the general /api/* behavior below so CloudFront matches the specific path first.
  ordered_cache_behavior {
    path_pattern           = "/api/contact"
    target_origin_id       = "apigw-contact"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = data.aws_cloudfront_origin_request_policy.all_viewer_except_host.id
  }

  ordered_cache_behavior {
    path_pattern           = "/api/ask"
    target_origin_id       = "apigw-contact"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = data.aws_cloudfront_origin_request_policy.all_viewer_except_host.id
  }

  ordered_cache_behavior {
    path_pattern           = "/mcp*"
    target_origin_id       = "apigw-contact"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = data.aws_cloudfront_origin_request_policy.all_viewer_except_host.id
  }

  # Cache behaviors for Lambda-served paths — API, auth, docs, health
  ordered_cache_behavior {
    path_pattern           = "/api/*"
    target_origin_id       = "lambda-function-url"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.lambda_oac[0].id
  }

  ordered_cache_behavior {
    path_pattern           = "/auth/*"
    target_origin_id       = "lambda-function-url"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.lambda_oac[0].id
  }

  ordered_cache_behavior {
    path_pattern           = "/health"
    target_origin_id       = "lambda-function-url"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.lambda_oac[0].id
  }

  ordered_cache_behavior {
    path_pattern           = "/docs"
    target_origin_id       = "lambda-function-url"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.lambda_oac[0].id
  }

  # SPA hashed assets — long-lived cache keyed by content hash in filename
  ordered_cache_behavior {
    path_pattern           = "/assets/*"
    target_origin_id       = "s3-spa"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD"]
    cached_methods  = ["GET", "HEAD"]

    cache_policy_id = data.aws_cloudfront_cache_policy.caching_optimized.id
  }

  # Default: serve SPA from S3. All unmatched paths (/, /about, /dashboard, etc.)
  # hit S3 which returns 403 for non-existent keys → custom_error_response → /index.html.
  default_cache_behavior {
    target_origin_id       = "s3-spa"
    viewer_protocol_policy = "redirect-to-https"

    allowed_methods = ["GET", "HEAD"]
    cached_methods  = ["GET", "HEAD"]

    # CachingDisabled for index.html so deploys propagate immediately
    cache_policy_id = data.aws_cloudfront_cache_policy.caching_disabled.id
  }

  # SPA history routing: S3 returns 403 for non-existent keys (private bucket).
  # Map both 403 and 404 to /index.html so client-side routes resolve.
  custom_error_response {
    error_code            = 403
    response_code         = 200
    response_page_path    = "/index.html"
    error_caching_min_ttl = 0
  }

  custom_error_response {
    error_code            = 404
    response_code         = 200
    response_page_path    = "/index.html"
    error_caching_min_ttl = 0
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn      = aws_acm_certificate_validation.wildcard[0].certificate_arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }

  tags = {
    Name = "${var.project_name}-cdn"
  }
}

# ─── Route53 Records (prod-only) ──────────────────────────────────────────────

resource "aws_route53_record" "apex_a" {
  count           = local.is_prod_cdn ? 1 : 0
  zone_id         = data.aws_route53_zone.main[0].zone_id
  name            = var.domain_name
  type            = "A"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main[0].domain_name
    zone_id                = aws_cloudfront_distribution.main[0].hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "apex_aaaa" {
  count           = local.is_prod_cdn ? 1 : 0
  zone_id         = data.aws_route53_zone.main[0].zone_id
  name            = var.domain_name
  type            = "AAAA"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main[0].domain_name
    zone_id                = aws_cloudfront_distribution.main[0].hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "www_a" {
  count           = local.is_prod_cdn ? 1 : 0
  zone_id         = data.aws_route53_zone.main[0].zone_id
  name            = "www.${var.domain_name}"
  type            = "A"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main[0].domain_name
    zone_id                = aws_cloudfront_distribution.main[0].hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "www_aaaa" {
  count           = local.is_prod_cdn ? 1 : 0
  zone_id         = data.aws_route53_zone.main[0].zone_id
  name            = "www.${var.domain_name}"
  type            = "AAAA"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main[0].domain_name
    zone_id                = aws_cloudfront_distribution.main[0].hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "dev_a" {
  count           = local.is_prod_cdn ? 1 : 0
  zone_id         = data.aws_route53_zone.main[0].zone_id
  name            = "dev.${var.domain_name}"
  type            = "A"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main[0].domain_name
    zone_id                = aws_cloudfront_distribution.main[0].hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "dev_aaaa" {
  count           = local.is_prod_cdn ? 1 : 0
  zone_id         = data.aws_route53_zone.main[0].zone_id
  name            = "dev.${var.domain_name}"
  type            = "AAAA"
  allow_overwrite = true

  alias {
    name                   = aws_cloudfront_distribution.main[0].domain_name
    zone_id                = aws_cloudfront_distribution.main[0].hosted_zone_id
    evaluate_target_health = false
  }
}
