CREATE TABLE IF NOT EXISTS about_sections (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    page       TEXT    NOT NULL,
    slug       TEXT    NOT NULL UNIQUE,
    heading    TEXT    NOT NULL,
    body       TEXT    NOT NULL,
    icon       TEXT,
    sort_order INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_about_sections_page ON about_sections(page);
