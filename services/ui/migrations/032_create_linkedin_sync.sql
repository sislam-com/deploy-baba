CREATE TABLE IF NOT EXISTS linkedin_positions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  linkedin_id TEXT,
  company TEXT NOT NULL,
  title TEXT NOT NULL,
  location TEXT,
  start_date TEXT NOT NULL,
  end_date TEXT,
  description TEXT,
  mapped_job_id INTEGER REFERENCES jobs(id),
  sync_status TEXT NOT NULL DEFAULT 'unreviewed',
  imported_at TEXT NOT NULL DEFAULT (datetime('now')),
  reviewed_at TEXT
);

CREATE TABLE IF NOT EXISTS linkedin_projects (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  linkedin_id TEXT,
  title TEXT NOT NULL,
  description TEXT,
  url TEXT,
  start_date TEXT,
  end_date TEXT,
  associated_position TEXT,
  mapped_challenge_id INTEGER REFERENCES challenges(id),
  sync_status TEXT NOT NULL DEFAULT 'unreviewed',
  imported_at TEXT NOT NULL DEFAULT (datetime('now')),
  reviewed_at TEXT
);

CREATE TABLE IF NOT EXISTS linkedin_sync_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source TEXT NOT NULL,
  positions_count INTEGER NOT NULL DEFAULT 0,
  projects_count INTEGER NOT NULL DEFAULT 0,
  imported_at TEXT NOT NULL DEFAULT (datetime('now'))
);
