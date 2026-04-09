-- W-SYNC: composite UNIQUE indexes required by ADR-010 upsert convention.
-- See plans/modules/dashboard-sync.md (W-SYNC.2 natural-key audit table).

CREATE UNIQUE INDEX IF NOT EXISTS ux_job_details_job_sort
    ON job_details(job_id, sort_order);

CREATE UNIQUE INDEX IF NOT EXISTS ux_competency_evidence_comp_job_sort
    ON competency_evidence(competency_id, job_id, sort_order);
