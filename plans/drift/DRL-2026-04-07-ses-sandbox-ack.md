# DRL-2026-04-07-ses-sandbox-ack — SES Sandbox Blocks Acknowledgement Emails

**Date:** 2026-04-07
**Severity:** Medium (feature partially non-functional; admin notifications unaffected)
**Status:** **RESOLVED 2026-04-08** — SES production access granted; ack path live

---

## Observed Symptom

Acknowledgement emails (W-CTF.4.13) arrive only when the submitter's `email` address
happens to be a verified SES identity in the account. For every other address the ack
silently fails with a `MessageRejected` error in CloudWatch.

Admin notifications (to `contact-sislam@shantopagla.com`) continue to work correctly
because that recipient is a verified SES identity and satisfies the sandbox constraint.

---

## Root Cause

The SES account in `us-east-1` is in **sandbox mode**. In sandbox mode:

- `send_email` from a verified identity → allowed
- `send_email` to a verified identity → allowed
- `send_email` to any unverified address → rejected:
  `MessageRejected: Email address is not verified. The following identities failed
  the check in region us-east-1: <recipient>`

This failure mode was explicitly predicted in ADR-010:

> Requires SES production access (out of sandbox) so the email Lambda can send to
> arbitrary unverified submitter addresses. If the account is in sandbox, the ack
> will silently fail and be logged.
> — `plans/adr/ADR-010-emailer-response.md:89`

IAM policy in `infra/email-lambda.tf:59-81` is **not** the cause — it correctly
authorises `ses:SendEmail` against `it@sislam.com`. IAM has no concept of recipient
verification.

---

## Remediation

### Interim (applied 2026-04-07)

**Change 1 — `infra/email-lambda.tf`:** commented out `SES_ACK_FROM_EMAIL` env var.
`try_send_ack()` returns `Ok(())` early when the var is unset, so the `error!` log
spam stops. Admin notifications are unaffected.

**Change 2 — `services/email/src/main.rs`:** the ack-caller branch now classifies
`MessageRejected` as `warn!(code = "message_rejected", ...)` instead of `error!`,
so if the env var is ever restored before production access is granted, the known
failure is grep-able and doesn't pollute the error stream.

### Permanent Fix (out-of-band, user action required)

1. **AWS Console → SES → Account dashboard → Request production access** for
   `us-east-1`. Provide the use case: transactional contact-form acknowledgements,
   low volume (≤100/day), opt-in only (submitter initiated the request).
2. After approval: verify with
   `aws sesv2 get-account --region us-east-1 --profile personal`
   → look for `ProductionAccessEnabled: true`.
3. Restore `SES_ACK_FROM_EMAIL = "it@sislam.com"` in `infra/email-lambda.tf`
   (remove the comment block).
4. Redeploy: `just infra-apply personal` + `just email-deploy personal`.
5. Submit the contact form with an unverified Gmail address and verify the ack
   arrives. CloudWatch should show `info!(to = ..., "acknowledgement email sent")`.
6. Close this drift log with a resolution note and flip W-CTF.4.13 back to DONE
   in both `plans/modules/contact-form.md` and `plans/INDEX.md`.

---

## Affected Files

| File | Change |
|------|--------|
| `infra/email-lambda.tf` | `SES_ACK_FROM_EMAIL` commented out (restore after production access) |
| `services/email/src/main.rs` | `MessageRejected` classified as `warn` in ack-caller branch |
| `plans/modules/contact-form.md` | W-CTF.4.13 → BLOCKED |
| `plans/INDEX.md` | P2.5 item 16 → BLOCKED |

---

## Cross-References

→ ADR-010 (`plans/adr/ADR-010-emailer-response.md`) — documents this trade-off
→ W-CTF.4.13 (`plans/modules/contact-form.md:81`)
→ `infra/email-lambda.tf`
→ `services/email/src/main.rs` — `try_send_ack()`

---

## Resolution (2026-04-08)

**Date production access granted:** 2026-04-08

**Verification:**
- Tested delivery to an unverified external Gmail address — email arrived in inbox
- CloudWatch showed the expected `info!(to = ..., "acknowledgement email sent")` line
- No `warn!(code = "message_rejected", ...)` entries

**Confirmation command output:** `aws sesv2 get-account --region us-east-1`
- `ProductionAccessEnabled: true`
- `EnforcementStatus: HEALTHY`
- `SendingEnabled: true`
- `Details.MailType: TRANSACTIONAL`
- `Details.ReviewDetails.Status: GRANTED`
- `Details.ReviewDetails.CaseId: 177558013300465`
- `Max24HourSend: 50000`, `MaxSendRate: 14/s`

**File changes that restored the live path:**
- `infra/email-lambda.tf`: removed interim comment block; `SES_ACK_FROM_EMAIL = "it@sislam.com"` restored as a live env var
- `plans/modules/contact-form.md`: W-CTF.4.13 → DONE
- `plans/INDEX.md`: P2.5 item 16 → DONE; drift log index row → RESOLVED

The interim mitigation section above is retained as historical record of the symptom and the out-of-band remediation steps.
