-- Add source classification columns to eval cases and top_k tracking to results.
-- Uses ALTER TABLE ADD COLUMN (simpler and more robust than create-copy-drop-rename).
-- The _migrations table ensures this only runs once per database.

ALTER TABLE rag_eval_cases ADD COLUMN expected_source_kind TEXT;
ALTER TABLE rag_eval_cases ADD COLUMN expected_entity_type TEXT;

ALTER TABLE rag_eval_results ADD COLUMN top_k_hit INTEGER;

-- Backfill source classification metadata

UPDATE rag_eval_cases
SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
    expected_entity_type = COALESCE(expected_entity_type, 'competency')
WHERE question IN (
  'What are your primary skills and technical expertise?',
  'Tell me about your experience with AI/LLM systems and RAG pipelines',
  'What is your experience with cloud infrastructure and AWS?',
  'How many competencies does the portfolio list?'
);

UPDATE rag_eval_cases
SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
    expected_entity_type = COALESCE(expected_entity_type, 'job')
WHERE question IN (
  'Describe your technical leadership and team management experience',
  'What platforms and products have you built end-to-end?',
  'Compare the jobs at Scala Computing and the personal projects'
);

UPDATE rag_eval_cases
SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
    expected_entity_type = COALESCE(expected_entity_type, 'challenge')
WHERE question IN (
  'How does the RAG pipeline in this portfolio project work?',
  'Tell me about the 27-step deployment challenge'
);

UPDATE rag_eval_cases
SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
    expected_entity_type = COALESCE(expected_entity_type, 'about')
WHERE question IN (
  'What are the key architecture decisions in this portfolio?'
);

INSERT INTO rag_eval_cases (
  question, expected_hit, source_path, category, difficulty, expected_source_kind, expected_entity_type
)
VALUES
  (
    'Which challenge explains key constraints and tradeoffs?',
    'constraint',
    'portfolio://challenge',
    'challenge',
    'medium',
    'portfolio',
    'challenge'
  ),
  (
    'Which challenge documents measurable outcomes and metrics?',
    'metric',
    'portfolio://challenge',
    'challenge',
    'medium',
    'portfolio',
    'challenge'
  ),
  (
    'Which challenge references ADR or module alignment?',
    'ADR',
    'portfolio://challenge',
    'challenge',
    'hard',
    'portfolio',
    'challenge'
  )
ON CONFLICT(question) DO UPDATE SET
  expected_hit = EXCLUDED.expected_hit,
  source_path = EXCLUDED.source_path,
  category = EXCLUDED.category,
  difficulty = EXCLUDED.difficulty,
  expected_source_kind = EXCLUDED.expected_source_kind,
  expected_entity_type = EXCLUDED.expected_entity_type;
