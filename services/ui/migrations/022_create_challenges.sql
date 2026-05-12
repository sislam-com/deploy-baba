CREATE TABLE IF NOT EXISTS challenges (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    slug              TEXT    NOT NULL UNIQUE,
    title             TEXT    NOT NULL,
    job_id            INTEGER REFERENCES jobs(id),
    description       TEXT    NOT NULL,
    short_description TEXT,
    tech_stack        TEXT,
    category          TEXT,
    url               TEXT,
    image_url         TEXT,
    featured          INTEGER NOT NULL DEFAULT 0,
    sort_order        INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_challenges_job_id ON challenges(job_id);

-- Seed data: ADR-010 upsert convention
INSERT INTO challenges (slug, title, job_id, description, short_description, tech_stack, category, url, featured, sort_order)
VALUES (
    'deploy-baba-portfolio',
    'deploy-baba Portfolio Platform',
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Full-stack portfolio and resume platform built with Rust/Axum backend, React/Vite SPA, SQLite on EFS, deployed to AWS Lambda at zero recurring cost. Features RAG-powered AI Q&A, admin dashboard with CRUD, OpenAPI spec generation, and CI/CD via GitHub Actions.',
    'Zero-cost Rust portfolio platform on AWS Lambda',
    'Rust,Axum,React,Vite,TypeScript,SQLite,AWS Lambda,EFS,CloudFront,OpenTofu,Claude',
    'fullstack',
    'https://github.com/shantopagla/deploy-baba',
    1,
    0
)
ON CONFLICT(slug) DO UPDATE SET
    title = EXCLUDED.title,
    job_id = EXCLUDED.job_id,
    description = EXCLUDED.description,
    short_description = EXCLUDED.short_description,
    tech_stack = EXCLUDED.tech_stack,
    category = EXCLUDED.category,
    url = EXCLUDED.url,
    featured = EXCLUDED.featured,
    sort_order = EXCLUDED.sort_order;

INSERT INTO challenges (slug, title, job_id, description, short_description, tech_stack, category, url, featured, sort_order)
VALUES (
    '27-step-platform-deployment',
    '27-Step Platform Deployment Automation',
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Designed and implemented a 27-step automated deployment pipeline for a cloud-native simulation platform (SaaS/PaaS). Covered infrastructure provisioning, service deployment, database migrations, health checks, and rollback procedures across multiple AWS accounts and regions.',
    'End-to-end deployment automation for cloud simulation platform',
    'Go,AWS CDK,AWS SSM,Step Functions,CodePipeline,Docker,ECS',
    'platform',
    NULL,
    1,
    1
)
ON CONFLICT(slug) DO UPDATE SET
    title = EXCLUDED.title,
    job_id = EXCLUDED.job_id,
    description = EXCLUDED.description,
    short_description = EXCLUDED.short_description,
    tech_stack = EXCLUDED.tech_stack,
    category = EXCLUDED.category,
    url = EXCLUDED.url,
    featured = EXCLUDED.featured,
    sort_order = EXCLUDED.sort_order;

INSERT INTO challenges (slug, title, job_id, description, short_description, tech_stack, category, url, featured, sort_order)
VALUES (
    'scala-multi-tenancy',
    'Scala Multi-Tenancy Architecture',
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Architected a hierarchical multi-tenancy system for a SaaS simulation platform supporting organization-level, team-level, and user-level resource isolation. Implemented role-based access control, tenant-scoped data partitioning, and cross-tenant sharing capabilities.',
    'Hierarchical multi-tenant SaaS architecture with RBAC',
    'Go,React,Redux,PostgreSQL,AWS,REST API',
    'fullstack',
    NULL,
    1,
    2
)
ON CONFLICT(slug) DO UPDATE SET
    title = EXCLUDED.title,
    job_id = EXCLUDED.job_id,
    description = EXCLUDED.description,
    short_description = EXCLUDED.short_description,
    tech_stack = EXCLUDED.tech_stack,
    category = EXCLUDED.category,
    url = EXCLUDED.url,
    featured = EXCLUDED.featured,
    sort_order = EXCLUDED.sort_order;

INSERT INTO challenges (slug, title, job_id, description, short_description, tech_stack, category, url, featured, sort_order)
VALUES (
    'rag-grounding-citation',
    'RAG Grounding & Citation Verification System',
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Designed and implemented an LLM output quality system for a portfolio AI assistant. Built a grounding contract (ADR-016) that constrains Claude responses to verified resume data via structured prompt assembly. The DefaultPromptAssembler injects live portfolio chunks with citation tags, and entity_to_prose converters transform raw DB rows into prose the LLM can ground against. HybridRetriever combines SQLite FTS5 full-text search with keyword-triggered live data injection, ensuring architecture and auth questions surface code chunks instead of being crowded out by portfolio metadata.',
    'LLM grounding contract with citation tags and hybrid retrieval',
    'Rust,Claude API,SQLite FTS5,RAG,Prompt Engineering,Axum',
    'ai',
    NULL,
    1,
    3
)
ON CONFLICT(slug) DO UPDATE SET
    title = EXCLUDED.title,
    job_id = EXCLUDED.job_id,
    description = EXCLUDED.description,
    short_description = EXCLUDED.short_description,
    tech_stack = EXCLUDED.tech_stack,
    category = EXCLUDED.category,
    url = EXCLUDED.url,
    featured = EXCLUDED.featured,
    sort_order = EXCLUDED.sort_order;
