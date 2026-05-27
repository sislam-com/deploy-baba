---
name: add-migration
description: Create a numbered SQLite migration file and wire it into the MIGRATIONS array in db.rs. Follows the NNN_description.sql naming convention.
argument-hint: "[description]"
---

Add a new SQLite migration to the `services/ui` database. Current migration count: 011.

## Steps

1. **Determine the next migration number**
   - Run: `ls services/ui/migrations/ | sort | tail -5`
   - The next number is the highest `NNN` + 1, zero-padded to 3 digits

2. **Create the migration file**
   - Path: `services/ui/migrations/<NNN>_<description>.sql`
   - Use snake_case for description (e.g. `012_add_blog_posts.sql`)
   - Write idiomatic SQLite — no `CASCADE` without `PRAGMA foreign_keys`, use `INTEGER PRIMARY KEY` for rowid alias
   - Example template:
     ```sql
     -- Migration 012: add blog posts table
     CREATE TABLE IF NOT EXISTS blog_posts (
         id      INTEGER PRIMARY KEY,
         title   TEXT NOT NULL,
         body    TEXT NOT NULL,
         created TEXT NOT NULL DEFAULT (datetime('now'))
     );
     ```

3. **Wire into db.rs MIGRATIONS array**
   - File: `services/ui/src/db.rs`
   - Find the `MIGRATIONS` static array (search for `include_str!`)
   - Append at the end:
     ```rust
     include_str!("../migrations/<NNN>_<description>.sql"),
     ```
   - The array index = migration number - 1 (0-based). Order is critical — never reorder.

4. **Verify**
   - Run `just dev` — if the migration SQL is invalid, the app will panic at startup with a clear error
   - Check: `just ui-run` and confirm the DB opens without error

5. **Update agent cache**
   - Edit `.agent-cache/index.json` → increment `database.migration_count`

## Naming Convention Examples

| Number | Description | Filename |
|--------|-------------|----------|
| 001 | initial schema | `001_initial_schema.sql` |
| 008 | about section | `008_about_section.sql` |
| 012 | add blog posts | `012_add_blog_posts.sql` |

## Key Files

- `services/ui/migrations/` — all SQL files
- `services/ui/src/db.rs` — `MIGRATIONS` array and connection setup
