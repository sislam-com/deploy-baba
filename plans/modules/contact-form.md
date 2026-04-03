# W-CTF: Contact Form + SES Email
**Service:** `services/email/` (new) + `services/ui/` | **Status:** WIP
**Depends on:** W-UI, W-AUTH (nav rendering) | **Depended on by:** —

## W-CTF.1 Purpose
Public contact page with email form. Sends via AWS SES to Google Groups.
Separate non-VPC Lambda handles email sending to avoid VPC networking costs.

## W-CTF.2 Public API Surface
- `GET /contact` — renders contact form page (main Lambda, VPC)
- `GET /api/contact?name=...&email=...&subject=...&message=...&website=...` — submits form via query params (main Lambda → email Lambda SDK invoke)

## W-CTF.3 Implementation Notes

### Architecture (as deployed 2026-04-03)
- Contact form uses **GET + query params** (not POST + JSON body) to bypass CloudFront OAC body hash issue (see DRL-FUA-3, DRL-FUA-4)
- Main UI Lambda (`/api/contact`) receives GET request with query params, invokes email Lambda via AWS SDK
- Email Lambda is a separate non-VPC binary (`services/email/`) invoked directly via `aws_sdk_lambda::Client::invoke()` — no public HTTP endpoint, no CloudFront involvement
- VPC Interface Endpoint for Lambda (`com.amazonaws.us-east-1.lambda`) allows the VPC-bound UI Lambda to call the email Lambda without NAT Gateway
- Sends from `noreply@mail.sislam.com` (subdomain isolates reputation from main domain)
- SES domain identity for `mail.sislam.com` with DKIM, SPF (softfail `~all`), DMARC (p=none monitor)
- Reserved concurrency=5, timeout=10s, memory=128MB for cost/abuse protection
- Honeypot field `website`: silently returns `{"success":true}` if filled (bot detection)
- In-memory per-IP rate limit: max 3 submissions/hour per IP (keyed on `x-forwarded-for`)
- Input validation: name(100), email(254), subject(200), message(5000) max lengths
- `contact-sislam@shantopagla.com` is the admin inbox

### What was dropped from original plan
- `infra/cdn.tf` email Lambda origin + `/api/contact` cache behavior → removed; not needed since UI Lambda invokes email Lambda via SDK
- `infra/email-lambda.tf` Function URL CORS config → still present but unused (email Lambda has no public URL used in production)
- Email Lambda `ALLOWED_ORIGIN` env var → still set but not checked at Lambda level (CORS not applicable for SDK invocations)

## W-CTF.4 Work Items
| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-CTF.4.1 | Create `infra/ses.tf` — SES identity + DKIM/SPF/DMARC for mail.sislam.com | DONE | |
| W-CTF.4.2 | Create `infra/email-lambda.tf` — Lambda + IAM + Function URL + CloudWatch alarm | DONE | |
| W-CTF.4.3 | Modify `infra/cdn.tf` — email Lambda origin + /api/contact cache behavior | DONE | |
| W-CTF.4.4 | Add `services/email/` workspace member — Rust binary | DONE | |
| W-CTF.4.5 | Add `services/ui/src/routes/contact.rs` — GET /contact page | DONE | |
| W-CTF.4.6 | Add `services/ui/templates/contact.html` — form template | DONE | |
| W-CTF.4.7 | Modify `services/ui/templates/base.html` — Contact nav link | DONE | |
| W-CTF.4.8 | Add justfile recipes: `email-build`, `email-deploy` | DONE | |
| W-CTF.4.9 | Pre-apply verification checks | DONE | Infra applied 2026-04-03 |
| W-CTF.4.10 | End-to-end test: form → SES → contact-sislam@shantopagla.com | DONE | Full pipeline verified 2026-04-03 |

## W-CTF.5 Test Strategy
- Local: `just ui` renders /contact, form JS works (email disabled in dev mode via `CONTACT_TO_EMAIL` not set)
- Pre-deploy: `tofu plan` review, `dig` DNS checks
- Post-deploy: submit form on sislam.com/contact, verify email arrives in Google Groups

## W-CTF.6 Cross-References
→ ADR-003 (Lambda Function URL)
→ ADR-004 (Dual-mode entry point)
→ W-AUTH.POST-FIX (CloudFront body hash — bypassed via GET+query params; see DRL-FUA-3, DRL-FUA-4)
