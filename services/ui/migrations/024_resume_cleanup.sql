-- 024_resume_cleanup.sql
-- Resume generation infrastructure: display control, curated skills, education table
-- ADR-010 upsert pattern; safe on fresh and existing DBs

-- ── 1. Add resume_display column to jobs ────────────────────────────────────
-- Values: 'full' (default) | 'condensed' (one-liner) | 'hidden' (omitted)
ALTER TABLE jobs ADD COLUMN resume_display TEXT NOT NULL DEFAULT 'full';

-- ── 2. Add resume_visible flag to job_details ───────────────────────────────
-- 1 = show on resume (default), 0 = hide from resume output
ALTER TABLE job_details ADD COLUMN resume_visible INTEGER NOT NULL DEFAULT 1;

-- ── 3. Create skill_categories table ────────────────────────────────────────
CREATE TABLE IF NOT EXISTS skill_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

-- ── 4. Create curated_skills table ──────────────────────────────────────────
CREATE TABLE IF NOT EXISTS curated_skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL REFERENCES skill_categories(id),
    skill_name TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE(category_id, skill_name)
);

-- ── 5. Create education table ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS education (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    degree TEXT NOT NULL UNIQUE,
    institution TEXT NOT NULL,
    location TEXT,
    sort_order INTEGER NOT NULL
);

-- ── 6. Seed education ───────────────────────────────────────────────────────
INSERT OR IGNORE INTO education (degree, institution, location, sort_order) VALUES
    ('B.S. Computing Sciences & Graphic Design', 'University of Central Oklahoma', 'Edmond, OK', 0),
    ('Certificate, Management & Leadership Skills', 'NST, Rockhurst University Continuing Education Center', NULL, 1);

-- ── 7. Seed skill categories ────────────────────────────────────────────────
INSERT INTO skill_categories (slug, name, sort_order) VALUES
    ('languages',      'Languages & Runtimes', 0),
    ('frameworks',     'Frameworks',           1),
    ('ai-ml',          'AI / ML',              2),
    ('aws',            'AWS',                  3),
    ('infrastructure', 'Infrastructure',       4),
    ('practices',      'Practices',            5);

-- ── 8. Seed curated skills ──────────────────────────────────────────────────
-- Languages & Runtimes
INSERT OR IGNORE INTO curated_skills (category_id, skill_name, sort_order) VALUES
    ((SELECT id FROM skill_categories WHERE slug = 'languages'), 'Rust', 0),
    ((SELECT id FROM skill_categories WHERE slug = 'languages'), 'Go', 1),
    ((SELECT id FROM skill_categories WHERE slug = 'languages'), 'TypeScript', 2),
    ((SELECT id FROM skill_categories WHERE slug = 'languages'), 'JavaScript', 3),
    ((SELECT id FROM skill_categories WHERE slug = 'languages'), 'SQL', 4),
    ((SELECT id FROM skill_categories WHERE slug = 'languages'), 'Python', 5);

-- Frameworks
INSERT OR IGNORE INTO curated_skills (category_id, skill_name, sort_order) VALUES
    ((SELECT id FROM skill_categories WHERE slug = 'frameworks'), 'Tokio', 0),
    ((SELECT id FROM skill_categories WHERE slug = 'frameworks'), 'Axum', 1),
    ((SELECT id FROM skill_categories WHERE slug = 'frameworks'), 'React', 2),
    ((SELECT id FROM skill_categories WHERE slug = 'frameworks'), 'Redux', 3),
    ((SELECT id FROM skill_categories WHERE slug = 'frameworks'), 'Angular', 4),
    ((SELECT id FROM skill_categories WHERE slug = 'frameworks'), 'Node.js', 5);

-- AI / ML
INSERT OR IGNORE INTO curated_skills (category_id, skill_name, sort_order) VALUES
    ((SELECT id FROM skill_categories WHERE slug = 'ai-ml'), 'RAG (FTS5 + vector)', 0),
    ((SELECT id FROM skill_categories WHERE slug = 'ai-ml'), 'LLM orchestration', 1),
    ((SELECT id FROM skill_categories WHERE slug = 'ai-ml'), 'Anthropic Claude', 2),
    ((SELECT id FROM skill_categories WHERE slug = 'ai-ml'), 'Agentic tool dispatch', 3),
    ((SELECT id FROM skill_categories WHERE slug = 'ai-ml'), 'Prompt engineering', 4);

-- AWS
INSERT OR IGNORE INTO curated_skills (category_id, skill_name, sort_order) VALUES
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'Lambda', 0),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'EFS', 1),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'S3', 2),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'CloudFront', 3),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'Cognito', 4),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'SES', 5),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'EventBridge', 6),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'CDK', 7),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'SSM', 8),
    ((SELECT id FROM skill_categories WHERE slug = 'aws'), 'Secrets Manager', 9);

-- Infrastructure
INSERT OR IGNORE INTO curated_skills (category_id, skill_name, sort_order) VALUES
    ((SELECT id FROM skill_categories WHERE slug = 'infrastructure'), 'OpenTofu', 0),
    ((SELECT id FROM skill_categories WHERE slug = 'infrastructure'), 'GitHub Actions CI/CD', 1),
    ((SELECT id FROM skill_categories WHERE slug = 'infrastructure'), 'Docker', 2),
    ((SELECT id FROM skill_categories WHERE slug = 'infrastructure'), 'SQLite', 3),
    ((SELECT id FROM skill_categories WHERE slug = 'infrastructure'), 'MySQL', 4);

-- Practices
INSERT OR IGNORE INTO curated_skills (category_id, skill_name, sort_order) VALUES
    ((SELECT id FROM skill_categories WHERE slug = 'practices'), 'OpenAPI / REST design', 0),
    ((SELECT id FROM skill_categories WHERE slug = 'practices'), 'Zero-cost architecture', 1),
    ((SELECT id FROM skill_categories WHERE slug = 'practices'), 'Multi-tenant SaaS', 2),
    ((SELECT id FROM skill_categories WHERE slug = 'practices'), 'Cross-compiled deploys (ARM64)', 3);

-- ── 9. Set resume_display for old/thin roles ────────────────────────────────
UPDATE jobs SET resume_display = 'condensed' WHERE slug IN ('independent-contractor', 'wbgo', 'logistics-com');
UPDATE jobs SET resume_display = 'hidden' WHERE slug = 'openpages';
