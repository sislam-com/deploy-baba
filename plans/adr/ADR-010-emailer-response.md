# ADR-010: Synchronous Email Lambda Invocation with Typed Response Propagation

**Date:** 2026-04-07
**Status:** Accepted
**Affected modules:** W-CTF, W-UI

---

## Context

The contact form pipeline involves two Lambdas:

1. **UI Lambda** (`services/ui`) — validates PoW, rate-limits, invokes email Lambda
2. **Email Lambda** (`services/email`) — validates input, calls SES, returns a result

A key design decision is how the UI Lambda invokes the email Lambda and what it does
with the response. The options are:

- **Synchronous (RequestResponse):** wait for the email Lambda to finish, relay its result as HTTP status
- **Asynchronous (Event):** fire-and-forget; return 202 Accepted immediately; user gets no confirmation
- **Queue-mediated (SQS/SNS):** fully decoupled; email Lambda polls the queue

## Decision

> Invoke the email Lambda synchronously (RequestResponse) and propagate its typed `ContactResponse` as HTTP status codes back to the browser.

The email Lambda returns `ContactResponse { success: bool, message: String }`. The UI Lambda:
1. Deserializes this response from the Lambda invoke payload blob
2. Maps `success: true` → HTTP 200, `success: false` → HTTP 500
3. Proxies the `message` string verbatim so the browser can display it

Both Lambdas share the same `ContactResponse` shape (though the types are defined independently per service — no shared crate).

## Acknowledgement Email

After the admin notification email is accepted by SES, the email Lambda sends a
**second** email — the acknowledgement — to the submitter's address (the `email`
field from the form).

**Acknowledgement properties:**
- **From:** `it@sislam.com` — a **separately-verified SES email identity**, distinct
  from the domain identity (`mail.sislam.com`) used for the admin notification. Using
  a personal address as the sender makes the acknowledgement feel human rather than
  automated and improves inbox placement for a reply-friendly touchpoint
- **To:** the submitter's `email` field
- **Subject:** `Thanks for reaching out — sislam.com`
- **Body (plain text):** thank-you message + a verbatim copy of the original
  `subject` and `message` so the submitter has a record of what they sent
- **Env var:** introduce `SES_ACK_FROM_EMAIL` (distinct from the existing
  `SES_FROM_EMAIL` used for admin notifications) so the two identities can evolve
  independently. Falls back to skipping the ack if unset (same pattern as the
  existing `SES_FROM_EMAIL`/`CONTACT_TO_EMAIL` dev-mode skip)

**Send order:** sequential. The admin notification is sent first; the ack only fires
if the admin send succeeded.

**Failure handling:** the ack email is best-effort. If the second `send_email()` call
fails, the email Lambda logs the error at `error` level and **still returns
`success: true`**. Rationale: the admin notification (the primary purpose) succeeded,
so from the user's perspective the contact form worked. Failing the whole request
because of a courtesy email would be a regression in observable behaviour.

**Abuse considerations:**
- A bot could supply an arbitrary submitter `email`, causing us to send an ack to a
  third party. This is mitigated by the existing honeypot field, the 3/hour/IP rate
  limit, and the PoW challenge — all of which already gate the request before it
  reaches the email Lambda
- Submitter `email` is already validated for length (≤254) and `@` presence in
  `services/email/src/main.rs`; no additional validation is required by this ADR

## Consequences

### Positive

- User receives real send confirmation: the browser knows whether SES actually accepted the email
- Errors from SES surface as 500s rather than silent failures — observable in CloudWatch and in the UI
- The typed response contract between Lambdas is explicit and checked at deserialization time
- No additional infrastructure (no SQS, no SNS, no polling)
- Submitters get immediate confirmation that their message was received, plus a
  copy for their records — small UX improvement and trust signal

### Negative / Trade-offs

- The UI Lambda is blocked for the duration of the SES API call (~100–400 ms). For a portfolio site with minimal traffic, this is acceptable
- Lambda-to-Lambda synchronous invocation increases cold-start latency on the first contact form submission
- If the email Lambda times out, the UI Lambda also surfaces a 500 — no partial success state
- Two SES API calls per submission instead of one — roughly doubles the SES portion
  of contact-form latency (still well within Lambda timeout budget)
- Requires SES production access (out of sandbox) so the email Lambda can send to
  arbitrary unverified submitter addresses. If the account is in sandbox, the ack
  will silently fail and be logged. **(Resolved 2026-04-08: production access
  granted for `us-east-1`; see `DRL-2026-04-07-ses-sandbox-ack` and
  `docs/aws-setup.md` §SES Manual Setup for the reproducible steps.)**
- A bot supplying a third-party email address causes us to send unsolicited mail
  to that address. Mitigated but not eliminated by honeypot + rate limit + PoW
- Two verified SES identities now sit on the contact path (`mail.sislam.com`
  domain identity for admin notifications, `it@sislam.com` email identity for
  acknowledgements). Both must remain verified in the target account; losing
  verification on either breaks a distinct part of the flow

### Neutral

- The `ContactResponse` type is duplicated across both services (not in a shared crate). This is intentional — the email Lambda is independently deployable and should not share a library dependency with the UI Lambda

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Async Event invocation | User gets no confirmation. "Message sent!" could be a lie if SES fails. Breaks the feedback loop principle. |
| SQS queue between Lambdas | Adds cost and operational complexity. Zero-cost philosophy (ADR-005) favors direct invocation for portfolio-scale traffic. |
| Inline SES call in UI Lambda | Couples SES to the UI Lambda. Email Lambda exists specifically to isolate SES permissions and keep the UI Lambda focused on serving HTTP. |
| HTTP call via API Gateway | Adds another network hop and API Gateway dependency. Lambda invoke SDK is direct and authenticated. |

## Open Questions

- Should the ack body be HTML (richer formatting) or plain text only? Default: plain
  text, matching the admin notification format
- Should we set `Reply-To:` on the ack? Since the From is already a real human
  mailbox (`it@sislam.com`), a dedicated Reply-To is likely unnecessary — replies
  will land directly in the `it@` inbox. Default: no Reply-To header
- Should the admin notification also migrate to `it@sislam.com` for consistency, or
  keep the `noreply@mail.sislam.com` domain identity? Default: keep them split —
  admin notifications stay on the domain identity, acknowledgements use the
  personal identity

## Cross-References

- → ADR-003 (Lambda Function URL — email Lambda has no Function URL; it is invoke-only)
- → ADR-005 (Zero-Cost Philosophy — justifies synchronous over queue-mediated)
- → ADR-009 (API Gateway for POST /api/contact — the route that triggers this pipeline)
- → W-CTF (contact-form module)
- → W-UI (UI service — `services/ui/src/routes/contact.rs`)
- → DRL-2026-04-03-contact-form (contact form implementation drift log)
