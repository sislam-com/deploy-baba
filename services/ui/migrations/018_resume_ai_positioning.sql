-- 018_resume_ai_positioning.sql
-- W-SYNC: Reposition resume as AI Systems + Platform Engineer
-- ADR-010 upsert — safe to run on both fresh and existing DBs
--
-- Changes:
--   about_sections(me-bio): rewrite summary to AI Systems positioning
--   jobs(personal-projects): upgrade title, summary, tech_stack to reflect RAG/LLM/agentic work
--   job_details(personal-projects): add RAG pipeline, agentic loop, LLM abstraction bullets
--   job_details(scala-computing): replace weak AI bullet with two specific LLM/RAG bullets
--   competencies(ai-augmented-dev): upgrade to ai-llm-systems with richer description

-- ── 1. Professional Summary (about_sections me-bio) ───────────────────────────
INSERT INTO about_sections (page, slug, heading, body, icon, sort_order)
VALUES (
    'me',
    'me-bio',
    'Who I Am',
    'AI Systems & Platform Engineer with 20+ years building and scaling SaaS products, now focused on Rust-based LLM systems, retrieval-augmented generation (RAG), and agentic workflows. Architected and deployed a production AI platform (deploy-baba) featuring multi-corpus RAG, tool-executing LLM agents, and full AWS infrastructure using zero-cost Rust abstractions.',
    NULL,
    1
)
ON CONFLICT(slug) DO UPDATE SET
    body = EXCLUDED.body;

-- ── 2. personal-projects job row ─────────────────────────────────────────────
INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'personal-projects',
    'sislam.com',
    'AI Systems Engineer & Founder',
    NULL,
    '2025-01',
    NULL,
    'Designed and built a production-ready AI platform in Rust integrating retrieval-augmented generation (RAG), agentic LLM execution, and full AWS deployment.',
    'Rust,Tokio,Axum,SQLite,FTS5,OpenTofu,AWS Lambda,EFS,S3,CloudFront,Cognito,EventBridge,Anthropic,RAG,LLM,GitHub Actions',
    0
)
ON CONFLICT(slug) DO UPDATE SET
    title      = EXCLUDED.title,
    summary    = EXCLUDED.summary,
    tech_stack = EXCLUDED.tech_stack,
    start_date = EXCLUDED.start_date;

-- ── 3. personal-projects job_details — new/updated AI Systems bullets ─────────

-- sort_order 4 (was: 10-crate Rust library ecosystem) — keep but clarify RAG angle
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Architected modular Rust workspace (10+ crates) using trait-based composition and zero-cost abstractions (monomorphization over dynamic dispatch)',
    'achievement',
    4
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- sort_order 11 — RAG pipeline (new)
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Built hybrid RAG pipeline (SQLite + FTS5 + vector search) supporting multi-corpus retrieval across portfolio data, OpenAPI specs, source code, architecture decisions, and structured content',
    'achievement',
    11
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- sort_order 12 — agentic LLM loop (new)
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Implemented agentic LLM loop with tool execution (ADR-023), enabling dynamic backend function calls via HTTP from model outputs',
    'achievement',
    12
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- sort_order 13 — public API + rate limiting (new)
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Developed public /api/ask endpoint with rate limiting and portfolio-aware prompt assembly for live AI-powered portfolio assistant',
    'achievement',
    13
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- sort_order 14 — LLM provider abstraction (new)
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Designed LLM provider abstraction layer (ADR-015) with pluggable adapters — Anthropic implemented, extensible to OpenAI and local models',
    'achievement',
    14
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- ── 4. scala-computing — replace weak AI bullet with two specific LLM/RAG bullets ──

-- sort_order 9 was: "Evaluated and adopted AI-assisted development tools..."
-- Replace with a stronger, specific bullet
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Evaluated and prototyped LLM-driven workflows and AI-assisted development tools (Cursor, Claude), informing internal adoption strategy for AI-enhanced SaaS features',
    'achievement',
    9
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- sort_order 10 — new: LLM/RAG integration prototyping
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Prototyped early LLM/RAG integrations to explore AI-driven feature augmentation within the SaaS simulation platform',
    'achievement',
    10
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;

-- ── 5. Upgrade ai-augmented-dev competency to reflect full AI Systems work ────
INSERT INTO competencies (slug, name, description, icon, sort_order)
VALUES (
    'ai-llm-systems',
    'AI Systems & LLM Engineering',
    'Production RAG pipelines, agentic LLM execution with tool dispatch, LLM provider abstraction (Anthropic/OpenAI), multi-corpus retrieval, prompt engineering, and AI-augmented development workflows.',
    '🤖',
    3
)
ON CONFLICT(slug) DO UPDATE SET
    name        = EXCLUDED.name,
    description = EXCLUDED.description;

-- Retire old slug by updating description to redirect, keep for DB continuity
INSERT INTO competencies (slug, name, description, icon, sort_order)
VALUES (
    'ai-augmented-dev',
    'AI Systems & LLM Engineering',
    'Production RAG pipelines, agentic LLM execution with tool dispatch, LLM provider abstraction (Anthropic/OpenAI), multi-corpus retrieval, prompt engineering, and AI-augmented development workflows.',
    '🤖',
    3
)
ON CONFLICT(slug) DO UPDATE SET
    name        = EXCLUDED.name,
    description = EXCLUDED.description;
