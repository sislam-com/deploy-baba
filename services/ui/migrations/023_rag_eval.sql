-- RAG evaluation infrastructure: eval cases, run tracking, and query logging.

-- ── Eval dataset ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS rag_eval_cases (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    question      TEXT NOT NULL,
    expected_hit  TEXT NOT NULL,        -- substring that must appear in a correct answer
    source_path   TEXT,                 -- expected source path in retrieval results
    category      TEXT NOT NULL,        -- portfolio | architecture | code | edge-case
    difficulty    TEXT NOT NULL DEFAULT 'medium',
    UNIQUE(question)
);

CREATE TABLE IF NOT EXISTS rag_eval_runs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    run_at          TEXT NOT NULL DEFAULT (datetime('now')),
    git_sha         TEXT NOT NULL,
    prompt_version  TEXT NOT NULL,
    total_cases     INTEGER NOT NULL,
    pass_count      INTEGER NOT NULL,
    avg_groundedness REAL,
    avg_correctness  REAL,
    notes           TEXT
);

CREATE TABLE IF NOT EXISTS rag_eval_results (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id            INTEGER NOT NULL REFERENCES rag_eval_runs(id),
    case_id           INTEGER NOT NULL REFERENCES rag_eval_cases(id),
    answer            TEXT NOT NULL,
    citations_json    TEXT,
    chunks_json       TEXT,
    retrieval_hit     INTEGER NOT NULL DEFAULT 0,
    groundedness      REAL,
    correctness       REAL,
    citation_accuracy REAL,
    latency_ms        INTEGER,
    failure_type      TEXT,
    UNIQUE(run_id, case_id)
);

-- ── Query log (lightweight production telemetry) ─────────────────────────────

CREATE TABLE IF NOT EXISTS rag_query_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    query         TEXT NOT NULL,
    answer        TEXT,
    citations_json TEXT,
    chunks_json   TEXT,
    model         TEXT,
    input_tokens  INTEGER,
    output_tokens INTEGER,
    latency_ms    INTEGER,
    groundedness  REAL,
    ip_hash       TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Seed eval cases (ADR-010 upsert convention) ─────────────────────────────

-- Portfolio questions (from RECRUITER_QUESTIONS + variations)
INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What are your primary skills and technical expertise?', 'Rust', 'portfolio://competency', 'portfolio', 'easy')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('Tell me about your experience with AI/LLM systems and RAG pipelines', 'RAG', 'portfolio://competency', 'portfolio', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What is your experience with cloud infrastructure and AWS?', 'AWS', 'portfolio://competency', 'portfolio', 'easy')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('Describe your technical leadership and team management experience', 'team', 'portfolio://job', 'portfolio', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What platforms and products have you built end-to-end?', 'platform', 'portfolio://job', 'portfolio', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('How does the RAG pipeline in this portfolio project work?', 'FTS5', 'portfolio://challenge', 'portfolio', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What are the key architecture decisions in this portfolio?', 'ADR', 'portfolio://about', 'portfolio', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

-- Architecture questions
INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('Why was SQLite chosen over PostgreSQL for this project?', 'SQLite', NULL, 'architecture', 'easy')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('How is authentication implemented in this portfolio?', 'Cognito', NULL, 'architecture', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What is the grounding contract and how does it prevent hallucinations?', 'citation', NULL, 'architecture', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('How is the Lambda deployment configured?', 'Lambda', NULL, 'architecture', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What is ADR-016 about?', 'RAG', NULL, 'architecture', 'easy')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

-- Code questions
INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('How does HybridRetriever combine FTS and live portfolio data?', 'PORTFOLIO_KEYWORDS', NULL, 'code', 'hard')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What does the DefaultPromptAssembler do?', 'source', NULL, 'code', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('How does error handling work in the API routes?', 'StatusCode', NULL, 'code', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What is the PortfolioDataProvider trait?', 'PortfolioDataProvider', NULL, 'code', 'easy')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('How does the contact form proof-of-work challenge verification work?', 'SHA', NULL, 'code', 'hard')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

-- Edge cases
INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What is the meaning of life?', 'source', NULL, 'edge-case', 'hard')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('Compare the jobs at Scala Computing and the personal projects', 'Scala', 'portfolio://job', 'edge-case', 'hard')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('What technologies does this project NOT use?', 'source', NULL, 'edge-case', 'hard')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('How many competencies does the portfolio list?', 'competenc', 'portfolio://competency', 'edge-case', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;

INSERT INTO rag_eval_cases (question, expected_hit, source_path, category, difficulty)
VALUES ('Tell me about the 27-step deployment challenge', '27', 'portfolio://challenge', 'edge-case', 'medium')
ON CONFLICT(question) DO UPDATE SET expected_hit = EXCLUDED.expected_hit, source_path = EXCLUDED.source_path, category = EXCLUDED.category;
