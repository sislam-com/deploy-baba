# W-CTF: Contact Form + SES Email
**Service:** `services/email/` + `services/ui/` | **Status:** WIP (Secrets Manager pending)
**Depends on:** W-UI, W-AUTH (nav rendering) | **Depended on by:** —

## W-CTF.1 Purpose
Public contact page with email form. Sends via AWS SES to Google Groups.
Separate non-VPC Lambda handles email sending to avoid VPC networking costs.
Proof-of-work (PoW) challenge protects against bot submissions without CAPTCHA.

## W-CTF.2 Public API Surface
- `GET /contact` — renders contact form page (UI Lambda, VPC, OAC signed)
- `GET /api/contact/challenge` — issues HMAC-signed PoW challenge (UI Lambda, VPC, OAC signed)
- `POST /api/contact` — submits form with JSON body + PoW solution (API Gateway HTTP API → UI Lambda, **no OAC**)

## W-CTF.3 Implementation Notes

### Architecture (as deployed 2026-04-03, PoW branch deployed)

```
Browser → CloudFront → [GET /contact/challenge] → Lambda Function URL (OAC) → UI Lambda
Browser → CloudFront → [POST /api/contact] → API Gateway HTTP API (no OAC) → UI Lambda
UI Lambda → aws_sdk_lambda::invoke → Email Lambda → SES
```

**Why API Gateway for POST (DRL-FUA-3 workaround):**
CloudFront OAC + AWS_IAM Lambda Function URL rejects POST bodies — CloudFront sends
`UNSIGNED-PAYLOAD` instead of the actual body hash, causing `InvalidSignatureException`.
Solution: separate CloudFront origin pointing to an API Gateway HTTP API for just
`POST /api/contact`. The API Gateway doesn't use OAC signing, so POST bodies work.
See `infra/apigateway.tf`, `infra/cdn.tf` `/api/contact` ordered_cache_behavior.

**PoW protocol (stateless HMAC-signed challenges):**
- Server generates nonce (16 random bytes, hex), timestamp, difficulty=18, HMAC-SHA256 signature
- Browser receives challenge, loops: compute SHA256(nonce:solution_counter), check 18 leading zero bits
- Browser yields every 1000 iterations (setTimeout) to stay responsive (~1–3s solve time)
- Server verifies: HMAC signature, 5-min expiry, nonce not replayed, hash difficulty
- Used-nonce tracker: `OnceLock<Mutex<HashMap<String, Instant>>>` with auto-eviction

**`POW_SECRET` (MIGRATED — W-CTF.4.11 DONE):**
Stored in AWS Secrets Manager at `deploy-baba/prod/pow-secret`. Lambda reads
it via `POW_SECRET_ARN` env var at cold start (`init_pow_secret()` in main.rs).
Falls back to `"dev-secret-change-me"` locally when `POW_SECRET_ARN` not set.

**Email flow:**
- Main UI Lambda (`POST /api/contact`) invokes email Lambda via `aws_sdk_lambda::Client::invoke()`
- Email Lambda: separate non-VPC binary (`services/email/`), no public HTTP endpoint
- VPC Interface Endpoint for Lambda (`com.amazonaws.us-east-1.lambda`) allows VPC-bound
  UI Lambda to call email Lambda without NAT Gateway
- Sends from `noreply@mail.sislam.com` (subdomain isolates reputation from main domain)
- SES domain identity: `mail.sislam.com` with DKIM, SPF (~all softfail), DMARC (p=none)
- Reserved concurrency=5, timeout=10s, memory=128MB for cost/abuse protection
- Honeypot field `website`: silently returns success if filled (bot detection)
- In-memory per-IP rate limit: max 3 submissions/hour (keyed on `x-forwarded-for`)
- Input: name(100), email(254), subject(200), message(5000) max lengths
- `contact-sislam@shantopagla.com` is the admin inbox

**Workspace deps added:**
- `sha2 = "0.10"`, `hmac = "0.12"`, `hex = "0.4"`, `rand = "0.8"` (in root Cargo.toml + services/ui/Cargo.toml)

## W-CTF.4 Work Items
| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-CTF.4.1 | Create `infra/ses.tf` — SES identity + DKIM/SPF/DMARC for mail.sislam.com | DONE | |
| W-CTF.4.2 | Create `infra/email-lambda.tf` — Lambda + IAM + Function URL + CloudWatch alarm | DONE | |
| W-CTF.4.3 | Modify `infra/cdn.tf` — API Gateway origin + `/api/contact` cache behavior | DONE | Was originally email Lambda origin; replaced by API Gateway approach |
| W-CTF.4.4 | Add `services/email/` workspace member — Rust binary | DONE | |
| W-CTF.4.5 | Add `services/ui/src/routes/contact.rs` — contact handlers | DONE | |
| W-CTF.4.6 | Add `services/ui/templates/contact.html` — form template + PoW solver JS | DONE | |
| W-CTF.4.7 | Modify `services/ui/templates/base.html` — Contact nav link | DONE | |
| W-CTF.4.8 | Add justfile recipes: `email-build`, `email-deploy` | DONE | |
| W-CTF.4.9 | Create `infra/apigateway.tf` — HTTP API + Lambda proxy + POST route + stage | DONE | Bypasses OAC body hash issue (DRL-FUA-3) |
| W-CTF.4.10 | POST + PoW implementation — challenge/verify handlers + JS solver | DONE | Deployed 2026-04-03 |
| W-CTF.4.11 | Migrate `POW_SECRET` from Lambda env var → AWS Secrets Manager | **DONE** | W-SEC complete; `init_pow_secret()` in main.rs; SM secret + VPC endpoint + IAM policy in infra |
| W-CTF.4.12 | End-to-end test: form → PoW solve → POST → SES → contact-sislam@shantopagla.com | **OPEN** | Test after Secrets Manager migration complete |

## W-CTF.5 Test Strategy
- Local: `just ui` renders /contact, form JS works (email disabled in dev mode)
- Local PoW: open browser console, submit form, observe: "Verifying → Solving challenge → Sending"
- Post-deploy: `curl -X POST https://sislam.com/api/contact -H 'Content-Type: application/json' -d '{"name":"Test","email":"t@t.com","subject":"hi","message":"test","website":"","pow_nonce":"...","pow_timestamp":...,"pow_solution":...,"pow_signature":"..."}'`
- Manual: submit form on sislam.com/contact, verify email arrives in Google Groups

## W-CTF.6 Cross-References
→ ADR-003 (Lambda Function URL — POST exception via API Gateway, see ADR-009)
→ ADR-004 (Dual-mode entry point)
→ ADR-009 (API Gateway HTTP API for POST /api/contact — DRL-FUA-3 workaround)
→ W-SEC (Secrets Manager integration for POW_SECRET)
→ DRL-2026-04-03-contact-form (contact form implementation log)
→ DRL-2026-04-03-pow-apigateway (POST+PoW+API Gateway drift log — to be created)
