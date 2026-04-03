use anyhow::Result;
use rusqlite::Connection;
use std::sync::Mutex;

/// Compile-time embedded migrations — order matters.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_create_jobs",
        include_str!("../migrations/001_create_jobs.sql"),
    ),
    (
        "002_create_competencies",
        include_str!("../migrations/002_create_competencies.sql"),
    ),
    (
        "003_seed_jobs",
        include_str!("../migrations/003_seed_jobs.sql"),
    ),
    (
        "004_seed_job_details",
        include_str!("../migrations/004_seed_job_details.sql"),
    ),
    (
        "005_seed_competencies",
        include_str!("../migrations/005_seed_competencies.sql"),
    ),
    (
        "006_seed_competency_evidence",
        include_str!("../migrations/006_seed_competency_evidence.sql"),
    ),
    (
        "007_seed_personal_projects",
        include_str!("../migrations/007_seed_personal_projects.sql"),
    ),
    (
        "008_create_about_sections",
        include_str!("../migrations/008_create_about_sections.sql"),
    ),
    (
        "009_seed_about_sections",
        include_str!("../migrations/009_seed_about_sections.sql"),
    ),
];

pub struct Db {
    pub conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT    NOT NULL UNIQUE,
                applied_at TEXT    NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        for (name, sql) in MIGRATIONS {
            let already_applied: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
                    rusqlite::params![name],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if already_applied {
                continue;
            }

            tracing::info!("Applying migration: {}", name);
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                rusqlite::params![name],
            )?;
        }

        Ok(())
    }
}
