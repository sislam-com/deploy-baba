# ─── SES Domain Identity for mail.sislam.com ──────────────────────────────────
#
# Uses mail.sislam.com subdomain to isolate email-sending reputation from
# the main sislam.com domain. If the sending subdomain is flagged, main domain
# is unaffected. The existing sislam.com SES identity (manually created) is
# NOT managed here — leave it untouched.

locals {
  ses_domain = "mail.${var.domain_name}"
}

resource "aws_sesv2_email_identity" "mail" {
  count          = local.is_prod_cdn ? 1 : 0
  email_identity = local.ses_domain

  dkim_signing_attributes {
    next_signing_key_length = "RSA_2048_BIT"
  }
}

# ─── DKIM CNAME Records (3 tokens) ────────────────────────────────────────────

resource "aws_route53_record" "ses_dkim" {
  count   = local.is_prod_cdn ? 3 : 0
  zone_id = data.aws_route53_zone.main[0].zone_id
  name    = "${tolist(aws_sesv2_email_identity.mail[0].dkim_signing_attributes)[0].tokens[count.index]}._domainkey.${local.ses_domain}"
  type    = "CNAME"
  ttl     = 600
  records = ["${tolist(aws_sesv2_email_identity.mail[0].dkim_signing_attributes)[0].tokens[count.index]}.dkim.amazonses.com"]
}

# ─── SPF TXT Record ────────────────────────────────────────────────────────────
# Softfail (~all) during initial rollout. Upgrade to -all after monitoring.

resource "aws_route53_record" "ses_spf" {
  count   = local.is_prod_cdn ? 1 : 0
  zone_id = data.aws_route53_zone.main[0].zone_id
  name    = local.ses_domain
  type    = "TXT"
  ttl     = 600
  records = ["v=spf1 include:amazonses.com ~all"]
}

# ─── DMARC TXT Record ─────────────────────────────────────────────────────────
# Starts in monitor mode (p=none). Upgrade to p=quarantine after 2-4 weeks of
# clean aggregate reports at dmarc-reports@sislam.com.

resource "aws_route53_record" "ses_dmarc" {
  count   = local.is_prod_cdn ? 1 : 0
  zone_id = data.aws_route53_zone.main[0].zone_id
  name    = "_dmarc.${local.ses_domain}"
  type    = "TXT"
  ttl     = 600
  records = ["v=DMARC1; p=none; rua=mailto:dmarc-reports@${var.domain_name}"]
}
