# DRL-2026-04-03-contact-form

**Date:** 2026-04-03
**Topic:** Contact Form + SES Email Lambda implementation
**Status:** In Progress — code committed, pre-apply checks and e2e test pending

---

## Entries

### DRL-CTF-1: Separate email Lambda bypasses W-AUTH.POST-FIX

**Observation:** The existing CloudFront OAC body hash mismatch (W-AUTH.POST-FIX) would block the contact form POST. The email Lambda uses `authorization_type = "NONE"` on its Function URL and a separate CloudFront origin (no OAC), so the body is forwarded intact.

**Resolution:** Implemented. `/api/contact` routes to the email Lambda origin (no OAC, no body signing). Other routes continue to use the main Lambda with OAC. W-AUTH.POST-FIX remains open for the dashboard edit forms.

### DRL-CTF-2: mail.sislam.com subdomain for sending reputation isolation

**Observation:** Using `mail.sislam.com` as the SES identity isolates email-sending reputation from the main `sislam.com` domain. If the sending domain is flagged, sislam.com is unaffected.

**Resolution:** SES identity created for `mail.sislam.com`. DKIM CNAME records, SPF TXT (`~all` softfail), and DMARC TXT (`p=none` monitor) all scoped to `mail.sislam.com`. From address: `noreply@mail.sislam.com`.

**Follow-up:** After 2-4 weeks of clean DMARC reports, upgrade to `p=quarantine`.

### DRL-CTF-3: CONVENTIONS.md was missing SL, RSM, CTF domain codes

**Observation:** `plans/CONVENTIONS.md` domain codes table was missing `SL` (social-links), `RSM` (resume), and `CTF` (contact-form). These were implemented but never recorded.

**Resolution:** Added all three codes to CONVENTIONS.md as part of this session.

### DRL-CTF-4: Option A chosen — no DynamoDB storage

**Observation:** Two options for contact form submissions: (A) email only via Google Groups, (B) email + DynamoDB storage for dashboard review.

**Resolution:** Option A selected. Google Groups inbox serves as admin review. DynamoDB adds cost and complexity not justified for v1 portfolio site. Re-evaluate if submission volume warrants admin review tooling.

---

## Open Items

| ID | Description | Blocking |
|----|-------------|---------|
| W-CTF.4.9 | Pre-apply verification checks (SES identity status, DNS, Route53 zone, SES sandbox status) | `tofu apply` |
| W-CTF.4.10 | End-to-end test: submit form on sislam.com/contact, verify Google Groups delivery | Release |
