-- 028_resume_consolidation.sql
-- Consolidate job_details into fewer, higher-impact bullets for resume conciseness.
-- Updates bio text for broader platform engineering positioning.
-- Updates Scala Computing tech_stack to include Python/SAM/Step Functions.
-- Hides Logistics.com (job 8).

-- Hide Logistics.com
UPDATE jobs SET resume_display = 'hidden' WHERE id = 8;

-- Update Scala Computing tech stack
UPDATE jobs SET tech_stack = 'Go,Rust,Python,React,Redux,AWS SAM,AWS Step Functions,Lambda,CloudFormation,SSM,SES,IndexedDB,Cypress,Cursor,Claude' WHERE id = 2;

-- Update bio for broader platform engineering positioning
UPDATE about_sections SET body = 'Platform engineer with deep experience building and scaling enterprise SaaS products across the full stack — from Rust backend services and SQLite-backed data layers to AWS infrastructure provisioned as code. Specializes in zero-cost cloud architectures (Lambda, EFS, CloudFront, EventBridge) that minimize operational overhead while maintaining production resilience. Currently focused on integrating LLM capabilities — retrieval-augmented generation, tool-executing agents, and multi-provider model routing — into deployment automation and developer tooling.' WHERE slug = 'me-bio';

-- Delete all existing job_details
DELETE FROM job_details;

-- Job 1 — sislam.com (4 consolidated bullets)
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible) VALUES
(1, 'Architected a modular Rust workspace (10+ crates) with trait-based composition and compile-time monomorphization, eliminating runtime overhead while enabling independent crate evolution', 'achievement', 0, 1),
(1, 'Shipped a production web platform (Axum, SQLite/EFS, Cognito JWT, REST APIs) at $0/month on a fully serverless stack (Lambda, EFS, S3, CloudFront, EventBridge) defined in OpenTofu', 'achievement', 1, 1),
(1, 'Built an AI-powered portfolio assistant with hybrid RAG (SQLite FTS5 + vector search), agentic tool execution, and a pluggable multi-provider LLM abstraction (Anthropic, OpenAI)', 'achievement', 2, 1),
(1, 'Engineered a 35+ command developer experience layer and AI-friendly project conventions (CLAUDE.md, machine-readable manifests, custom skills) enabling efficient human-AI pair programming', 'achievement', 3, 1);

-- Job 2 — Scala Computing (4 consolidated bullets)
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible) VALUES
(2, 'Designed multi-tenant backend APIs (Go) and consolidated fragmented services into a Rust monorepo with unified public API surface, accelerating third-party integration velocity', 'achievement', 0, 1),
(2, 'Architected token-based authentication and middleware enabling secure, self-service onboarding of third-party emulation partners into the platform ecosystem', 'achievement', 1, 1),
(2, 'Owned full-stack delivery — React/Redux UI with client-side caching (IndexedDB), Python-based Step Functions orchestration (AWS SAM, Lambda, CloudFormation), automated deployment workflows (SSM, SES), and Cypress-based QA automation across consortium releases', 'achievement', 2, 1),
(2, 'Informed company AI strategy by prototyping LLM/RAG integrations (Cursor, Claude) for AI-augmented SaaS feature development', 'achievement', 3, 1);

-- Job 3 — Sunbird DCIM (2 consolidated bullets)
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible) VALUES
(3, 'Led SaaS UI migration from ExtJS/AngularJS to ES6/Angular 2.x, establishing a company-wide component library and rebuilding desktop-era interfaces as responsive modern SPAs', 'achievement', 0, 1),
(3, 'Co-authored RESTful API contracts (OpenAPI), implemented CSRF protection across multi-server backends (RoR, Java), and built a cross-platform mobile companion app (Cordova)', 'achievement', 1, 1);

-- Job 4 — FalconStor (2 consolidated bullets)
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible) VALUES
(4, 'Set the technical foundation for FreeStor SaaS console — selected AngularJS stack and architected multi-tiered REST proxy layer decoupling UI from storage services', 'achievement', 0, 1),
(4, 'Delivered real-time monitoring dashboard, i18n framework for multi-region deployment, and responsive mobile apps (Cordova) for iOS and Android', 'achievement', 1, 1);

-- Job 5 — GalaxE.Solutions (2 consolidated bullets)
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible) VALUES
(5, 'Led distributed onshore/offshore teams across concurrent enterprise engagements, earning repeat business through on-time, on-scope deliveries', 'achievement', 0, 1),
(5, 'Owned front-end delivery for e-commerce platforms serving MLB, Dick''s, NASCAR, and Toys R Us — achieving 1+ second page-load improvements and integrating SaaS features across multi-tenant clients', 'achievement', 1, 1);

-- Job 6 — Independent Contractor (1 consolidated bullet)
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible) VALUES
(6, 'Built custom CMS and e-commerce solutions for private clients — art collection management, talent casting workflows, and revenue channel integrations (Google AdSense, Amazon)', 'achievement', 0, 1);

-- Job 7 — WBGO (1 consolidated bullet)
INSERT INTO job_details (job_id, detail_text, category, sort_order, resume_visible) VALUES
(7, 'Delivered the station''s award-winning web presence using dynamic data-driven development (ASP, PHP5, SQL) while providing IT operations support for ~100 employees', 'achievement', 0, 1);
