-- Migration 020: fix me-summary content to remove name prefix
UPDATE about_sections 
SET body = 'AI Systems Engineer'
WHERE slug = 'me-summary';
