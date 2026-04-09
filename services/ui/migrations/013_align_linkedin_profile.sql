INSERT INTO social_links (platform, url, icon, label, visible, sort_order)
VALUES ('linkedin', 'https://www.linkedin.com/in/sharfulislam/', NULL, 'LinkedIn', 1, 0)
ON CONFLICT (platform)
DO UPDATE SET url = EXCLUDED.url;