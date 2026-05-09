-- Migration 021: Replace competencies with actual top skills (ADR-010)
-- Evidence will be rebuilt when Challenges CRUD is implemented.

DELETE FROM competency_evidence;
DELETE FROM competencies;

INSERT INTO competencies (slug, name, description, icon, sort_order) VALUES
('rust-systems',
 'Rust Systems Engineering',
 'Production async runtimes (Tokio/Axum), zero-cost abstractions, trait-based architecture, workspace-scale codebases, and Lambda-optimized binaries',
 NULL, 1),
('aws-cloud',
 'AWS Cloud Architecture',
 'Lambda, EFS, S3, CloudFront, Cognito, SES, Step Functions, SAM, CloudFormation, Route53, ACM, EventBridge — multi-account isolation and zero-cost deployment',
 NULL, 2),
('ai-llm',
 'AI & LLM Systems',
 'Production RAG pipelines (FTS5 + vector), agentic LLM execution with tool dispatch, provider abstraction (Anthropic/OpenAI), multi-corpus retrieval, prompt engineering',
 NULL, 3),
('platform-automation',
 'Platform Deployment Automation',
 '27-step Step Functions orchestration, IaC (Terraform/OpenTofu/SAM/CloudFormation), multi-tenant deployment pipelines, automated account provisioning',
 NULL, 4),
('fullstack-saas',
 'Full-Stack SaaS Engineering',
 'React, Angular, TypeScript, OpenAPI contracts, multi-vertical product ownership across simulation, DCIM, SDS, and e-commerce platforms',
 NULL, 5),
('multitenant-arch',
 'Multi-Tenant Architecture',
 'Go/Scala consortium multi-tenancy, OU-based AWS account isolation, global/local DB wiring, platform manager orchestration, subdomain routing',
 NULL, 6),
('technical-leadership',
 'Technical Leadership & Delivery',
 'Director-level platform operations, GUI Manager, Tech Lead — onshore/offshore teams, client relationships, architectural decision-making',
 NULL, 7);
