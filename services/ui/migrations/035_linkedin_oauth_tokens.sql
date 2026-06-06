CREATE TABLE IF NOT EXISTS linkedin_oauth_tokens (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  access_token TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  name TEXT,
  email TEXT,
  picture_url TEXT,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
