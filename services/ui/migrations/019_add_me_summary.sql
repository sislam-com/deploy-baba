-- Migration 019: add me-summary section with name and title for homepage (ADR-010)
INSERT INTO about_sections (page, slug, heading, body, icon, sort_order)
VALUES ('me', 'me-summary', 'Summary',
        'AI Systems Engineer',
        NULL, 0)
ON CONFLICT(slug) DO UPDATE SET
    page       = EXCLUDED.page,
    heading    = EXCLUDED.heading,
    body       = EXCLUDED.body,
    icon       = EXCLUDED.icon,
    sort_order = EXCLUDED.sort_order;
