-- Change tracking for dashboard-editable content tables.
-- Records INSERT/UPDATE/DELETE operations so `just db-changes` can show
-- pending edits before they get lost on DB recreate or S3 sync.

CREATE TABLE IF NOT EXISTS _change_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name  TEXT    NOT NULL,
    natural_key TEXT    NOT NULL,
    operation   TEXT    NOT NULL CHECK(operation IN ('INSERT','UPDATE','DELETE')),
    changed_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    synced      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_change_log_unsynced
    ON _change_log (synced) WHERE synced = 0;

-- ── Triggers: jobs (natural key: slug) ──────────────────────────────────────

CREATE TRIGGER IF NOT EXISTS trg_jobs_insert AFTER INSERT ON jobs
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('jobs', NEW.slug, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_jobs_update AFTER UPDATE ON jobs
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('jobs', NEW.slug, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_jobs_delete AFTER DELETE ON jobs
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('jobs', OLD.slug, 'DELETE');
END;

-- ── Triggers: job_details (natural key: job_id,sort_order) ──────────────────

CREATE TRIGGER IF NOT EXISTS trg_job_details_insert AFTER INSERT ON job_details
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('job_details', NEW.job_id || ':' || NEW.sort_order, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_job_details_update AFTER UPDATE ON job_details
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('job_details', NEW.job_id || ':' || NEW.sort_order, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_job_details_delete AFTER DELETE ON job_details
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('job_details', OLD.job_id || ':' || OLD.sort_order, 'DELETE');
END;

-- ── Triggers: competencies (natural key: slug) ─────────────────────────────

CREATE TRIGGER IF NOT EXISTS trg_competencies_insert AFTER INSERT ON competencies
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('competencies', NEW.slug, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_competencies_update AFTER UPDATE ON competencies
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('competencies', NEW.slug, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_competencies_delete AFTER DELETE ON competencies
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('competencies', OLD.slug, 'DELETE');
END;

-- ── Triggers: competency_evidence (natural key: competency_id,job_id,sort_order)

CREATE TRIGGER IF NOT EXISTS trg_competency_evidence_insert AFTER INSERT ON competency_evidence
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('competency_evidence', NEW.competency_id || ':' || NEW.job_id || ':' || NEW.sort_order, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_competency_evidence_update AFTER UPDATE ON competency_evidence
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('competency_evidence', NEW.competency_id || ':' || NEW.job_id || ':' || NEW.sort_order, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_competency_evidence_delete AFTER DELETE ON competency_evidence
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('competency_evidence', OLD.competency_id || ':' || OLD.job_id || ':' || OLD.sort_order, 'DELETE');
END;

-- ── Triggers: about_sections (natural key: slug) ───────────────────────────

CREATE TRIGGER IF NOT EXISTS trg_about_sections_insert AFTER INSERT ON about_sections
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('about_sections', NEW.slug, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_about_sections_update AFTER UPDATE ON about_sections
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('about_sections', NEW.slug, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_about_sections_delete AFTER DELETE ON about_sections
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('about_sections', OLD.slug, 'DELETE');
END;

-- ── Triggers: social_links (natural key: platform) ─────────────────────────

CREATE TRIGGER IF NOT EXISTS trg_social_links_insert AFTER INSERT ON social_links
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('social_links', NEW.platform, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_social_links_update AFTER UPDATE ON social_links
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('social_links', NEW.platform, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_social_links_delete AFTER DELETE ON social_links
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('social_links', OLD.platform, 'DELETE');
END;

-- ── Triggers: challenges (natural key: slug) ───────────────────────────────

CREATE TRIGGER IF NOT EXISTS trg_challenges_insert AFTER INSERT ON challenges
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('challenges', NEW.slug, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_challenges_update AFTER UPDATE ON challenges
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('challenges', NEW.slug, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_challenges_delete AFTER DELETE ON challenges
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('challenges', OLD.slug, 'DELETE');
END;

-- ── Triggers: skill_categories (natural key: slug) ─────────────────────────

CREATE TRIGGER IF NOT EXISTS trg_skill_categories_insert AFTER INSERT ON skill_categories
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('skill_categories', NEW.slug, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_skill_categories_update AFTER UPDATE ON skill_categories
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('skill_categories', NEW.slug, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_skill_categories_delete AFTER DELETE ON skill_categories
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('skill_categories', OLD.slug, 'DELETE');
END;

-- ── Triggers: curated_skills (natural key: category_id,skill_name) ─────────

CREATE TRIGGER IF NOT EXISTS trg_curated_skills_insert AFTER INSERT ON curated_skills
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('curated_skills', NEW.category_id || ':' || NEW.skill_name, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_curated_skills_update AFTER UPDATE ON curated_skills
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('curated_skills', NEW.category_id || ':' || NEW.skill_name, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_curated_skills_delete AFTER DELETE ON curated_skills
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('curated_skills', OLD.category_id || ':' || OLD.skill_name, 'DELETE');
END;

-- ── Triggers: education (natural key: degree) ──────────────────────────────

CREATE TRIGGER IF NOT EXISTS trg_education_insert AFTER INSERT ON education
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('education', NEW.degree, 'INSERT');
END;

CREATE TRIGGER IF NOT EXISTS trg_education_update AFTER UPDATE ON education
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('education', NEW.degree, 'UPDATE');
END;

CREATE TRIGGER IF NOT EXISTS trg_education_delete AFTER DELETE ON education
BEGIN
    INSERT INTO _change_log (table_name, natural_key, operation)
    VALUES ('education', OLD.degree, 'DELETE');
END;
