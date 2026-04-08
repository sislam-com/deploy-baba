-- Update existing records with new content
UPDATE about_sections SET 
  heading = 'Who I Am',
  body = 'I''m Sharful Islam - a seasoned software engineer with a passion for building custom, scalable systems. I thrive on solving complex problems and delivering high-impact solutions that drive business value. Over my career, as I developed tiered, role-based applications, I continually came across having to build hierarchies in applications - and I found that it was a common problem that had no good solutions. So I built Hierarchy, a Rust library for building hierarchies in applications, and open-sourced it to share with the community. As I continue to learn the role of AI in software development, I am excited to explore how it can be used to build even more efficient and scalable systems.'
WHERE slug = 'me-bio';

UPDATE about_sections SET 
  heading = 'Background',
  body = 'I''ve spent decades building production systems focused on enterprise SaaS and content management solutions from workflows to real-time compliance pipelines. Among the numerous custom solutions in various verticals are fundraiser management (non-profit), artwork management (custom), talent management (custom), productization of third party integrations into existing solutions (recommendations, reviews), end-to-end PaaS deployment using AWS step-functions and SES with S3 trigger from a web form, AI augmented deployment workflow (this repo). At this point in my career, I''m most comfortable working with AI Augmented workflow in any programming language at and organization level for a small to mid-size company where my efforts include exposure to AI Planning and execution pipeline that involves distributed systems, type-safe APIs, and infrastructure-as-code.'
WHERE slug = 'me-background';

UPDATE about_sections SET 
  heading = 'Core Skills',
  body = 'Rust · TypeScript · Python · AWS (Lambda, EFS, S3, Cognito, CloudFormation, IAM, SES, RDS) · SQLite · OpenTofu · Axum · Tokio · Askama · OpenAPI · SaaS · PaaS · SDLC'
WHERE slug = 'me-skills';

UPDATE about_sections SET 
  heading = 'Engineering Philosophy',
  body = 'Plan deliberately, but assume plans will evolve. Validate dependencies early, and continuously refine through execution. Prefer boring infrastructure, explicit systems, and strong type guarantees over hidden complexity. Optimize for clarity, debuggability, and long-term maintainability. Treat AI as a collaborator, not an oracle—design systems that verify, constrain, and observe its outputs. Build feedback loops into every layer: from compile-time guarantees to runtime observability to user outcomes. Every abstraction should earn its place—and justify its cost over time.'
WHERE slug = 'me-philosophy';

-- Insert any missing records (idempotent)
INSERT OR IGNORE INTO about_sections (page, slug, heading, body, icon, sort_order) VALUES
('me', 'me-bio', 'Who I Am',
 'I''m Sharful Islam - a seasoned software engineer with a passion for building custom, scalable systems. I thrive on solving complex problems and delivering high-impact solutions that drive business value. Over my career, as I developed tiered, role-based applications, I continually came across having to build hierarchies in applications - and I found that it was a common problem that had no good solutions. So I built Hierarchy, a Rust library for building hierarchies in applications, and open-sourced it to share with the community. As I continue to learn the role of AI in software development, I am excited to explore how it can be used to build even more efficient and scalable systems.',
 NULL, 1),

('me', 'me-background', 'Background',
 'I''ve spent decades building production systems focused on enterprise SaaS and content management solutions from workflows to real-time compliance pipelines. Among the numerous custom solutions in various verticals are fundraiser management (non-profit), artwork management (custom), talent management (custom), productization of third party integrations into existing solutions (recommendations, reviews), end-to-end PaaS deployment using AWS step-functions and SES with S3 trigger from a web form, AI augmented deployment workflow (this repo). At this point in my career, I''m most comfortable working with AI Augmented workflow in any programming language at and organization level for a small to mid-size company where my efforts include exposure to AI Planning and execution pipeline that involves distributed systems, type-safe APIs, and infrastructure-as-code.',
 NULL, 2),

('me', 'me-skills', 'Core Skills',
 'Rust · TypeScript · Python · AWS (Lambda, EFS, S3, Cognito, CloudFormation, IAM, SES, RDS) · SQLite · OpenTofu · Axum · Tokio · Askama · OpenAPI · SaaS · PaaS · SDLC',
 NULL, 3),

('me', 'me-philosophy', 'Engineering Philosophy',
 'Plan deliberately, but assume plans will evolve. Validate dependencies early, and continuously refine through execution. Prefer boring infrastructure, explicit systems, and strong type guarantees over hidden complexity. Optimize for clarity, debuggability, and long-term maintainability. Treat AI as a collaborator, not an oracle—design systems that verify, constrain, and observe its outputs. Build feedback loops into every layer: from compile-time guarantees to runtime observability to user outcomes. Every abstraction should earn its place—and justify its cost over time.',
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
