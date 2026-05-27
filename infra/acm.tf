# Managed wildcard cert: covers sislam.com + *.sislam.com (prod singleton)
#
# Shared across all environments. Dev workspace skips cert creation — it
# re-uses the existing wildcard cert via the shared CloudFront distribution.

locals {
  is_prod_acm = var.environment == "prod"
}

resource "aws_acm_certificate" "wildcard" {
  count                     = local.is_prod_acm ? 1 : 0
  domain_name               = var.domain_name
  subject_alternative_names = ["*.${var.domain_name}"]
  validation_method         = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = {
    Name    = "${var.project_name}-wildcard-cert"
    Project = var.project_name
  }
}

resource "aws_route53_record" "cert_validation" {
  for_each = local.is_prod_acm ? {
    for dvo in aws_acm_certificate.wildcard[0].domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  } : {}

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = data.aws_route53_zone.main[0].zone_id
}

resource "aws_acm_certificate_validation" "wildcard" {
  count                   = local.is_prod_acm ? 1 : 0
  certificate_arn         = aws_acm_certificate.wildcard[0].arn
  validation_record_fqdns = [for record in aws_route53_record.cert_validation : record.fqdn]
}
