-- A. Update personal-projects job row
UPDATE jobs SET
    title = 'AI-Augmented Platform Engineer & Founder',
    summary = 'Building deploy-baba — a zero-cost SaaS platform in Rust on AWS Lambda, developed end-to-end using AI-augmented engineering workflows with Claude Code as a core development partner.',
    tech_stack = 'Rust,Tokio,Axum,Askama,SQLite,OpenTofu,AWS Lambda,EFS,S3,CloudFront,Cognito,EventBridge,Claude Code,Claude'
WHERE slug = 'personal-projects';

-- B. Seed job_details for personal-projects
INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 0, 'achievement', 'Designed an agent cache system that snapshots full project knowledge (crate structure, ADRs, dependencies, plan status) to maintain context across AI coding sessions'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 1, 'achievement', 'Built a modular plan system (17 module plans, 8 ADRs, drift logs) serving as structured context for AI-assisted implementation'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 2, 'achievement', 'Developed AI-friendly project conventions — justfile-only interface, CLAUDE.md instructions, machine-readable manifests — enabling agentic coding workflows'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 3, 'achievement', 'Every feature designed, reviewed, and implemented through human-AI pair programming using Claude Code'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 4, 'achievement', 'Architected a 10-crate Rust library ecosystem with trait-based abstractions for config parsing, multi-format API spec generation (OpenAPI, GraphQL, Protobuf), and cross-format merging'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 5, 'achievement', 'Built full-stack production app on Axum with SQLite on EFS, Cognito JWT auth (RS256 with deploy-time JWKS embedding), admin dashboard, REST APIs, and auto-generated OpenAPI docs'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 6, 'achievement', 'Engineered zero-cost serverless infrastructure: Lambda Function URLs, EFS, S3, CloudFront CDN, EventBridge — no API Gateway, no RDS, no always-on compute'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 7, 'achievement', 'Defined entire cloud stack in 12 OpenTofu files with automated bootstrap, remote state, and drift logging'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 8, 'achievement', 'Created 35+ command developer experience layer covering quality gates, cross-compiled Lambda builds (ARM64), infra plan/apply, DB backup/restore, and resume generation from SQLite to DOCX/PDF'
FROM jobs WHERE slug = 'personal-projects';

INSERT OR IGNORE INTO job_details (job_id, sort_order, category, detail_text)
SELECT id, 9, 'achievement', 'Single Rust binary runs identically as a local Axum server or AWS Lambda handler via dual-mode entry point (ADR-004)'
FROM jobs WHERE slug = 'personal-projects';

-- C. Add competency_evidence rows for personal-projects

-- platform-architecture
INSERT OR IGNORE INTO competency_evidence (competency_id, job_id, detail_id, highlight_text, sort_order)
SELECT
    (SELECT id FROM competencies WHERE slug = 'platform-architecture'),
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    (SELECT id FROM job_details WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects') AND sort_order = 4),
    NULL, 5;

-- frontend-engineering
INSERT OR IGNORE INTO competency_evidence (competency_id, job_id, detail_id, highlight_text, sort_order)
SELECT
    (SELECT id FROM competencies WHERE slug = 'frontend-engineering'),
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    NULL,
    'Server-rendered Askama templates with HTMX-style interactions, admin dashboard master/detail views, and auto-generated OpenAPI/RapiDoc documentation',
    5;

-- technical-leadership
INSERT OR IGNORE INTO competency_evidence (competency_id, job_id, detail_id, highlight_text, sort_order)
SELECT
    (SELECT id FROM competencies WHERE slug = 'technical-leadership'),
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    NULL,
    'Solo architect and founder — designed modular plan system, 8 ADRs, and AI-augmented development workflow driving the full project lifecycle',
    4;

-- saas-product
INSERT OR IGNORE INTO competency_evidence (competency_id, job_id, detail_id, highlight_text, sort_order)
SELECT
    (SELECT id FROM competencies WHERE slug = 'saas-product'),
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    NULL,
    'Zero-cost SaaS deployment platform demonstrating Lambda Function URLs, Cognito auth, SQLite on EFS, and CloudFront CDN — production-grade at $0/month',
    4;
