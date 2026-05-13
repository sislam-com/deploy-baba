-- 025_resume_content_polish.sql
-- Resume content cleanup: hide process bullets, strip ADR refs, merge bullets,
-- update titles, collapse sub-engagements, update professional summary
-- ADR-010 upsert pattern where applicable

-- ── 1. Hide internal/process bullets for sislam.com ─────────────────────────
-- sort_order 0: agent cache (internal dev tooling)
-- sort_order 1: plan system (internal process)
-- sort_order 2: AI conventions (internal process)
-- sort_order 3: human-AI pair programming (methodology, not achievement)
-- sort_order 8: 35+ command DX layer (solo-project tooling)
-- sort_order 9: dual-mode Lambda (not top-6)
-- sort_order 10: custom skills for plans/ADRs (internal tooling)
UPDATE job_details SET resume_visible = 0
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects')
  AND sort_order IN (0, 1, 2, 3, 8, 9, 10);

-- ── 2. Merge zero-cost infra (6) + OpenTofu (7) into one bullet ─────────────
-- Update sort_order 6 with merged text, hide sort_order 7
UPDATE job_details
SET detail_text = 'Engineered zero-cost serverless infrastructure (Lambda Function URLs, EFS, S3, CloudFront, EventBridge) defined in 12 OpenTofu files with automated bootstrap and remote state'
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects')
  AND sort_order = 6;

UPDATE job_details SET resume_visible = 0
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects')
  AND sort_order = 7;

-- ── 3. Merge RAG pipeline (11) + /api/ask (13) into one bullet ──────────────
-- Update sort_order 11 with merged text, hide sort_order 13
UPDATE job_details
SET detail_text = 'Built hybrid RAG pipeline (SQLite FTS5 + vector search) with multi-corpus retrieval, powering a public /api/ask endpoint with rate limiting and portfolio-aware prompt assembly'
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects')
  AND sort_order = 11;

UPDATE job_details SET resume_visible = 0
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects')
  AND sort_order = 13;

-- ── 4. Strip ADR references from remaining visible bullets ──────────────────
-- sort_order 12: remove "(ADR-023)"
UPDATE job_details
SET detail_text = 'Implemented agentic LLM loop with tool execution, enabling dynamic backend function calls via HTTP from model outputs'
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects')
  AND sort_order = 12;

-- sort_order 14: remove "(ADR-015)"
UPDATE job_details
SET detail_text = 'Designed LLM provider abstraction layer with pluggable adapters — Anthropic implemented, extensible to OpenAI and local models'
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'personal-projects')
  AND sort_order = 14;

-- ── 5. Update Scala Computing title (merge to avoid demotion red flag) ──────
UPDATE jobs
SET title = 'Senior Engineer / Director of Platform Operations'
WHERE slug = 'scala-computing';

-- ── 6. Collapse GalaxE sub-engagements from 8 to 2 bullets ─────────────────
-- Hide all existing sub-engagement bullets (sort_orders 3-10)
UPDATE job_details SET resume_visible = 0
WHERE job_id = (SELECT id FROM jobs WHERE slug = 'galaxe-solutions')
  AND category = 'sub-engagement';

-- Insert 2 consolidated sub-engagement bullets
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'galaxe-solutions'),
    'Led front-end teams (5-10 developers) across e-commerce platform engagements for GSI Commerce, Coach, and TrueAction, serving brands including MLB, Dicks Sporting Goods, NASCAR, and Toys R Us',
    'achievement',
    20,
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'galaxe-solutions'),
    'Achieved 1+ second page-load improvements and integrated SaaS features (Bazaarvoice ratings, product recommendations) across multi-tenant client portfolio',
    'achievement',
    21,
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- ── 7. Update me-bio: drop "20+ years" ─────────────────────────────────────
INSERT INTO about_sections (page, slug, heading, body, icon, sort_order)
VALUES (
    'me',
    'me-bio',
    'Who I Am',
    'AI Systems & Platform Engineer with deep experience building and scaling SaaS products, now focused on Rust-based LLM systems, retrieval-augmented generation (RAG), and agentic workflows. Architected and deployed a production AI platform (deploy-baba) featuring multi-corpus RAG, tool-executing LLM agents, and full AWS infrastructure using zero-cost Rust abstractions.',
    NULL,
    1
)
ON CONFLICT(slug) DO UPDATE SET
    body = EXCLUDED.body;
