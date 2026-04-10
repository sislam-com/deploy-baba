-- W-SYNC: Capture dashboard edits (2026-04-10)
-- ADR-010 upsert — safe to run on both fresh and existing DBs
--
-- Changes captured:
--   jobs.personal-projects: company → 'sislam.com', start_date → '2026-03'
--   job_details(job_id=1, sort_order=0): trailing period added to detail_text
--   job_details(job_id=1, sort_order=10): new row — "Developed custom skills..."

-- Update personal-projects job metadata
INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'personal-projects',
    'sislam.com',
    'AI-Augmented Platform Engineer & Founder',
    NULL,
    '2026-03',
    NULL,
    'Building deploy-baba — a zero-cost SaaS platform in Rust on AWS Lambda, developed end-to-end using AI-augmented engineering workflows with Claude Code as a core development partner.',
    'Rust,Tokio,Axum,Askama,SQLite,OpenTofu,AWS Lambda,EFS,S3,CloudFront,Cognito,EventBridge,Claude Code,Claude',
    0
)
ON CONFLICT(slug)
DO UPDATE SET
    company    = EXCLUDED.company,
    start_date = EXCLUDED.start_date;

-- Update detail_text for job_id=1, sort_order=0 (trailing period)
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    1,
    'Designed an agent cache system that snapshots full project knowledge (crate structure, ADRs, dependencies, plan status) to maintain context across AI coding sessions.',
    'achievement',
    0
)
ON CONFLICT(job_id, sort_order)
DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- Insert new detail for job_id=1, sort_order=10
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    1,
    'Developed custom skills to manage plans, ADRs, resume regeneration and sql migration pipelines using standard skill schema usable via any AI.',
    'achievement',
    10
)
ON CONFLICT(job_id, sort_order)
DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;
