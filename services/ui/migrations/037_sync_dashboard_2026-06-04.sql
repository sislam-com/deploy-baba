-- Migration 037: Sync dashboard edit to jobs.scala-computing tech_stack formatting
-- Captured from _change_log: UPDATE jobs WHERE slug='scala-computing' (2026-06-04 14:51:12)

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order, resume_display)
VALUES (
    'scala-computing',
    'Scala Computing',
    'Senior Engineer / Director of Platform Operations',
    NULL,
    '2019-07',
    '2026-03',
    'Drove platform scalability and third-party ecosystem growth for a cloud-native simulation SaaS — owning backend services, public API strategy, and infrastructure automation through roles spanning IC to Director of Platform Operations.',
    'Go, Rust, Python, React, Redux, AWS SAM, AWS Step Functions, Lambda, CloudFormation, SSM, SES, IndexedDB, Cypress, Cursor, Claude',
    1,
    'full'
)
ON CONFLICT(slug)
DO UPDATE SET
    company = EXCLUDED.company,
    title = EXCLUDED.title,
    location = EXCLUDED.location,
    start_date = EXCLUDED.start_date,
    end_date = EXCLUDED.end_date,
    summary = EXCLUDED.summary,
    tech_stack = EXCLUDED.tech_stack,
    sort_order = EXCLUDED.sort_order,
    resume_display = EXCLUDED.resume_display;
