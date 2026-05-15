-- 027_outcome_focused_descriptions.sql
-- Rewrite job summaries and detail bullets to be outcome-focused
-- ADR-010 upsert pattern; safe on fresh and existing DBs
--
-- Scope: updates summary on jobs table and detail_text on job_details table
-- Does NOT change: titles, tech_stack, sort_order, resume_display, resume_visible, category

-- ── 1. personal-projects (sislam.com) ──────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'personal-projects',
    'sislam.com',
    'AI Systems Engineer & Founder',
    NULL,
    '2025-01',
    NULL,
    'Built and deployed a production AI platform in Rust on AWS Lambda — live at $0/month with RAG-powered Q&A, agentic LLM execution, and full serverless infrastructure — demonstrating end-to-end ownership from architecture through cloud deployment.',
    'Rust,Tokio,Axum,SQLite,FTS5,OpenTofu,AWS Lambda,EFS,S3,CloudFront,Cognito,EventBridge,Anthropic,RAG,LLM,GitHub Actions',
    0
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 4: modular Rust workspace
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Eliminated runtime overhead and enabled independent crate evolution by architecting a modular Rust workspace (10+ crates) with trait-based composition and compile-time monomorphization',
    'achievement',
    4
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 5: full-stack production app
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Shipped a production web platform (Axum, SQLite/EFS, Cognito JWT, admin dashboard, REST APIs) at zero ongoing cost, proving Rust viability for full-stack SaaS on Lambda',
    'achievement',
    5
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 6: zero-cost infra
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Achieved $0/month infrastructure baseline by engineering a fully serverless stack (Lambda Function URLs, EFS, S3, CloudFront, EventBridge) defined in 12 OpenTofu files with automated bootstrap',
    'achievement',
    6
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 11: RAG pipeline
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Enabled natural-language querying of portfolio content by building a hybrid RAG pipeline (SQLite FTS5 + vector search) powering a public /api/ask endpoint with rate limiting and multi-corpus retrieval',
    'achievement',
    11
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 12: agentic LLM loop
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Extended LLM capabilities beyond text generation by implementing an agentic execution loop that dynamically invokes backend functions based on model outputs',
    'achievement',
    12
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 14: LLM provider abstraction
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'personal-projects'),
    'Reduced vendor lock-in risk by designing a pluggable LLM provider abstraction — Anthropic implemented, extensible to OpenAI and local models without application code changes',
    'achievement',
    14
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- ── 2. scala-computing ─────────────────────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'scala-computing',
    'Scala Computing',
    'Senior Engineer / Director of Platform Operations',
    NULL,
    '2019-07',
    '2026-03',
    'Drove platform scalability and third-party ecosystem growth for a cloud-native simulation SaaS — owning backend services, public API strategy, and infrastructure automation through roles spanning IC to Director of Platform Operations.',
    'Go,Rust,React,Redux,AWS CDK,AWS SSM,SES,Cypress,IndexedDB,Cursor,Claude',
    1
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: multi-tenant backend APIs
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Powered the core simulation platform by designing multi-tenant backend APIs and distributed services in Go, supporting concurrent consortium customer workloads',
    'responsibility',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 1: Rust monorepo consolidation
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Accelerated third-party integration velocity by consolidating fragmented platform services into a Rust monorepo and establishing a unified public API surface',
    'achievement',
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 2: token-based auth
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Unlocked partner ecosystem growth by architecting token-based authentication and middleware enabling secure, self-service onboarding of third-party emulation partners',
    'achievement',
    2
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 3: browser-side caching
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Reduced server round-trips and improved dashboard responsiveness by implementing browser-side caching (IndexedDB) and client-rendered dynamic charting',
    'achievement',
    3
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 4: infrastructure automation
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Eliminated manual coordination overhead by automating tenant notifications, deployment workflows, and platform communications (AWS SSM, SES, Google Groups)',
    'achievement',
    4
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 5: platform UI
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Ensured platform feature continuity by owning and evolving the React/Redux UI — auth flows, API integrations, and role-based access controls supporting the full consortium user base',
    'responsibility',
    5
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 6: QA automation
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Shortened release-acceptance cycles by driving Cypress-based QA automation for consortium-level acceptance testing across platform updates',
    'achievement',
    6
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 7: legacy modernization
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Improved release velocity and cut operational burden by consolidating legacy services into shared, reusable platform infrastructure',
    'achievement',
    7
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 8: release management
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Maintained release stability across multi-repository integration through branch management, conflict resolution, and coordinated deployments',
    'responsibility',
    8
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 9: AI adoption
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Informed company AI adoption strategy by evaluating and prototyping LLM-driven workflows (Cursor, Claude) for AI-enhanced SaaS feature development',
    'achievement',
    9
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 10: LLM/RAG prototyping
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'scala-computing'),
    'Validated AI-augmented product direction by prototyping LLM/RAG integrations for the simulation platform, demonstrating feasibility of AI-driven feature augmentation',
    'achievement',
    10
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- ── 3. sunbird-dcim ────────────────────────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'sunbird-dcim',
    'Sunbird DCIM',
    'Senior Software Developer',
    NULL,
    '2016-03',
    '2019-01',
    'Unified three legacy UI frameworks into a single modern Angular SPA for a SaaS DCIM platform, establishing the company-wide design system, hardening security, and delivering mobile access for field technicians.',
    'Angular,AngularJS,ES6,OpenAPI,Cordova,Ruby on Rails,Java,CSRF',
    2
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: UI migration
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'sunbird-dcim'),
    'Accelerated feature delivery by planning and executing SaaS UI migration from ExtJS/AngularJS 1.x to ES6/Angular 2.x, eliminating cross-framework technical debt',
    'achievement',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 1: design system
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'sunbird-dcim'),
    'Reduced UI development time across teams by establishing a company-wide component library and UX standards adopted as the design system for the full product suite',
    'achievement',
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 2: SPA rebuild
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'sunbird-dcim'),
    'Improved user onboarding and reduced support tickets by rebuilding desktop-era DCIM interfaces as a responsive modern SPA',
    'achievement',
    2
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 3: API contracts
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'sunbird-dcim'),
    'Enabled parallel front-end and back-end development by co-authoring RESTful API contracts (YAML/OpenAPI) that standardized client-server communication patterns',
    'achievement',
    3
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 4: security hardening
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'sunbird-dcim'),
    'Closed cross-origin vulnerability gaps by implementing CSRF protection and session management hardening across the multi-server (RoR, Java) SaaS backend',
    'achievement',
    4
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 5: mobile app
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'sunbird-dcim'),
    'Extended DCIM platform reach to field technicians by building a cross-platform mobile companion app (Cordova/AngularJS) for on-site data center management',
    'achievement',
    5
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- ── 4. falconstor ──────────────────────────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'falconstor',
    'FalconStor Software',
    'GUI Manager',
    NULL,
    '2014-04',
    '2016-01',
    'Shipped the FreeStor SaaS management console from greenfield to production — leading front-end architecture, real-time monitoring, i18n, and mobile delivery for a Software-defined Storage platform.',
    'AngularJS,Highcharts,Bootstrap,SASS/SCSS,Cordova,i18n,RESTful API',
    3
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: stack selection
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'falconstor'),
    'Set the technical foundation for FreeStor by evaluating and selecting the front-end stack (AngularJS SPA), balancing team velocity with long-term maintainability',
    'achievement',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 1: proxy API layer
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'falconstor'),
    'Enabled independent UI and backend release cycles by architecting a multi-tiered RESTful proxy API layer decoupling the SaaS console from storage services',
    'achievement',
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 2: monitoring dashboard
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'falconstor'),
    'Gave SaaS customers real-time visibility into storage health and performance by building a widget-based monitoring dashboard (Highcharts, Bootstrap)',
    'achievement',
    2
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 3: i18n framework
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'falconstor'),
    'Unlocked multi-region SaaS deployment by designing an i18n framework (JSON-based label/error-code mapping) supporting localized UI without code changes',
    'achievement',
    3
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 4: mobile delivery
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'falconstor'),
    'Expanded platform reach to mobile users by delivering responsive SaaS UI (SASS/SCSS) and hybrid mobile apps (Cordova) for iOS and Android',
    'achievement',
    4
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- ── 5. galaxe-solutions ────────────────────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'galaxe-solutions',
    'GalaxE.Solutions',
    'Technical Lead (Consulting)',
    NULL,
    '2010-03',
    '2014-04',
    'Delivered front-end solutions across concurrent enterprise e-commerce engagements, leading distributed teams of 5-10 developers and driving measurable performance improvements for major retail brands.',
    'jQuery,JSP/JSTL,Spring,JavaScript,e-commerce,SEO,Bazaarvoice',
    4
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: candidate screening
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'galaxe-solutions'),
    'Built high-performing client delivery teams by screening and selecting UI/UXD candidates across onshore and offshore talent pools',
    'responsibility',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 1: optimization strategies
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'galaxe-solutions'),
    'Improved delivery quality across concurrent projects by defining optimization strategies and architectural solutions that reduced rework and shortened timelines',
    'responsibility',
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 2: team leadership
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'galaxe-solutions'),
    'Earned repeat engagements by leading distributed onshore/offshore teams to on-time, on-scope deliveries while building lasting client relationships',
    'achievement',
    2
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 20: consolidated e-commerce engagements
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'galaxe-solutions'),
    'Owned front-end delivery for enterprise e-commerce platforms (GSI Commerce, Coach, TrueAction), leading teams of 5-10 developers serving brands including MLB, Dick''s Sporting Goods, NASCAR, and Toys R Us',
    'achievement',
    20
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 21: no change — already outcome-focused with metric
-- "Achieved 1+ second page-load improvements..." is kept as-is

-- ── 6. independent-contractor ──────────────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'independent-contractor',
    'Independent Contractor (sislam.com)',
    'Independent UI Developer',
    NULL,
    '2008-05',
    '2010-04',
    'Delivered custom web applications and CMS platforms for independent clients, digitizing art collection workflows, automating talent casting operations, and establishing small business online revenue channels.',
    'jQuery,PHP5,MySQL,AJAX,JSON,Flash/XML,DHTML,Google AdSense',
    5
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: art collection CMS
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'independent-contractor'),
    'Digitized art collection management for a private client by building a custom CMS (jQuery/PHP5/MySQL) with AJAX-driven workflows, media upload, and interactive DHTML carousels',
    'achievement',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 1: talent portfolio CMS
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'independent-contractor'),
    'Streamlined talent casting operations by delivering a full-stack CMS with automated casting sheet generation, email workflows, and advanced candidate search',
    'achievement',
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 2: Jersey Ice Corp
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'independent-contractor'),
    'Established online revenue channels for Jersey Ice Corp. by launching web initiatives with Google AdSense, Amazon aStore, and RESTful integrations',
    'achievement',
    2
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- ── 7. wbgo ────────────────────────────────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'wbgo',
    'Newark Public Radio — Jazz 88.3 FM (WBGO)',
    'IT Coordinator / Web Developer',
    'Newark, NJ',
    '2002-01',
    '2008-01',
    'Built WBGO''s award-winning web presence and internal tools, including a fundraising management system that streamlined donor engagement operations for the public radio station.',
    'DHTML,ASP,PHP5,SQL,XML,ActionScript,JavaScript,networking',
    6
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: stakeholder demos
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'wbgo'),
    'Secured stakeholder buy-in by creating wireframes, prototypes, and demos for Web group, Strategy Group, and Board meetings',
    'responsibility',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 1: award-winning sites
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'wbgo'),
    'Delivered the station''s award-winning web presence by building dynamic, data-driven sites using DHTML, ASP, PHP5, SQL, and ActionScript',
    'achievement',
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 2: IT support
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'wbgo'),
    'Maintained operational continuity for ~100 employees by providing help-desk, networking, and server support',
    'responsibility',
    2
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- ── 8. logistics-com ───────────────────────────────────────────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'logistics-com',
    'Logistics.com',
    'User Interface Developer',
    'Burlington, MA',
    '2001-01',
    '2001-12',
    'Standardized the front-end component library for an enterprise SaaS transportation management platform, ensuring cross-browser reach and enabling internationalization.',
    'JavaScript,DHTML,CSS,UI internationalization',
    7
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: cross-browser UI
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'logistics-com'),
    'Ensured cross-browser reach for the enterprise TMS platform by developing browser- and platform-agnostic UI components (JavaScript, DHTML, CSS)',
    'achievement',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- sort 1: design standards
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'logistics-com'),
    'Established visual consistency across the product suite by defining graphic standards, layouts, and style guides while researching internationalization strategies for global expansion',
    'achievement',
    1
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;

-- ── 9. openpages (hidden — updating for data completeness) ─────────────────────

INSERT INTO jobs (slug, company, title, location, start_date, end_date, summary, tech_stack, sort_order)
VALUES (
    'openpages',
    'Openpages Inc.',
    'Webmaster',
    'Westford, MA',
    '2000-01',
    '2001-01',
    'Supported an early-stage SaaS governance/risk/compliance platform (later acquired by IBM) by managing the web presence and internal development tools.',
    'HTML,CSS,JavaScript,web management',
    8
)
ON CONFLICT(slug) DO UPDATE SET
    summary = EXCLUDED.summary;

-- sort 0: web presence
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (
    (SELECT id FROM jobs WHERE slug = 'openpages'),
    'Maintained web presence and internal tooling for an early-stage SaaS GRC platform later acquired by IBM',
    'responsibility',
    0
)
ON CONFLICT(job_id, sort_order) DO UPDATE SET
    detail_text = EXCLUDED.detail_text;
