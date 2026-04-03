INSERT OR IGNORE INTO about_sections (page, slug, heading, body, icon, sort_order) VALUES
('me', 'me-bio', 'Who I Am',
 'I''m Sharful Islam — a full-stack SaaS engineer who specialises in zero-cost, serverless systems built with Rust and AWS. I care deeply about correctness, performance, and making deployment as boring as possible.',
 NULL, 1),

('me', 'me-background', 'Background',
 'I''ve spent years building production systems across fintech and enterprise SaaS — from IBM OpenPages risk-management workflows to real-time compliance pipelines. I''m most at home when the problem involves distributed systems, type-safe APIs, and infrastructure-as-code.',
 NULL, 2),

('me', 'me-skills', 'Core Skills',
 'Rust · TypeScript · Python · AWS (Lambda, EFS, S3, Cognito, CloudFront) · SQLite · OpenTofu · Axum · Tokio · Askama · OpenAPI · GraphQL · gRPC',
 NULL, 3),

('me', 'me-philosophy', 'Engineering Philosophy',
 'Zero-cost first — pay only for what you use, scale only when the data demands it. Prefer boring infrastructure, expressive type systems, and compile-time guarantees over runtime magic. Every abstraction should earn its place.',
 NULL, 4),

('repo', 'repo-overview', 'Project Overview',
 'deploy-baba is a zero-cost Rust portfolio and deployment-automation platform hosted on AWS Lambda. It serves this portfolio site, exposes live API demos in OpenAPI / GraphQL / gRPC, and ships its own infra via OpenTofu — all from a single Rust workspace.',
 NULL, 1),

('repo', 'repo-stack', 'Tech Stack',
 'Rust 2021 · Axum 0.7 · Askama (compile-time templates) · Rusqlite (SQLite on EFS) · AWS Lambda (aarch64) · CloudFront CDN · Cognito (auth) · OpenTofu · EventBridge · S3',
 NULL, 2),

('repo', 'repo-architecture', 'Architecture',
 'The workspace contains 10 pure-library crates and one Lambda binary (services/ui). A dual-mode entry point (ADR-004) lets the same binary run locally or on Lambda. SQLite lives on EFS for zero-cold-start persistence, with S3 backup on an EventBridge schedule. Auth uses Cognito implicit grant with RS256 JWT validation; a dev-mode bypass skips auth when COGNITO_POOL_ID is unset.',
 NULL, 3),

('repo', 'repo-crates', 'Library Crates',
 'config-core · config-toml · config-yaml · config-json · api-core · api-openapi · api-graphql · api-grpc · api-merger · infra-types. Each crate is a zero-cost abstraction: implement the trait, get the output format for free.',
 NULL, 4);
