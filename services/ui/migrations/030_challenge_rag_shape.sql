ALTER TABLE challenges ADD COLUMN problem TEXT;
ALTER TABLE challenges ADD COLUMN constraints TEXT;
ALTER TABLE challenges ADD COLUMN decisions TEXT;
ALTER TABLE challenges ADD COLUMN implementation TEXT;
ALTER TABLE challenges ADD COLUMN outcomes TEXT;
ALTER TABLE challenges ADD COLUMN metrics TEXT;
ALTER TABLE challenges ADD COLUMN related_job_slug TEXT;
ALTER TABLE challenges ADD COLUMN related_plan_module TEXT;
ALTER TABLE challenges ADD COLUMN related_adr TEXT;

-- Seed structured defaults for existing featured challenge entries.
UPDATE challenges
SET
  problem = COALESCE(problem, short_description),
  constraints = COALESCE(constraints, 'Zero recurring cost, practical maintainability, and verifiable grounding.'),
  decisions = COALESCE(decisions, 'Prioritize local-first MCP context and explicit plan/ADR alignment.'),
  implementation = COALESCE(implementation, description),
  outcomes = COALESCE(outcomes, short_description),
  metrics = COALESCE(metrics, 'See challenge description for measurable impact.'),
  related_job_slug = COALESCE(related_job_slug, CASE
    WHEN job_id IS NULL THEN NULL
    ELSE (SELECT slug FROM jobs WHERE jobs.id = challenges.job_id)
  END),
  related_plan_module = COALESCE(related_plan_module, 'W-RAG'),
  related_adr = COALESCE(related_adr, 'ADR-016')
WHERE featured = 1;
