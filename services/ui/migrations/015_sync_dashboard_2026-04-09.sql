-- W-SYNC: Capture dashboard edit to about_sections.me-bio (2026-04-09)
-- ADR-010 upsert — safe to run on both fresh and existing DBs

INSERT INTO about_sections (page, slug, heading, body, icon, sort_order)
VALUES (
    'me',
    'me-bio',
    'Who I Am',
    'Experienced SaaS/PaaS engineer with a passion for building custom, scalable systems solving complex problems and delivering high-impact solutions that drive business value. Over my career, as I developed tiered, role-based applications, I continually came across having to build hierarchies in applications - and I found that it was a common problem that had no good solutions. This portfolio portal implemented using AI augmented workflow plans is my playground to work with AI on an entire SaaS lifecycle. As I continue to learn the role of AI in software development, I am excited to explore how it can be used to build even more efficient and scalable systems.',
    NULL,
    1
)
ON CONFLICT(slug)
DO UPDATE SET
    body = EXCLUDED.body;
