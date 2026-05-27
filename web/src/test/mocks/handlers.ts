import { http, HttpResponse } from 'msw'

// Mock data for testing - matches OpenAPI types from types.gen.ts
export const mockJobs = [
  {
    id: 1,
    slug: 'senior-engineer',
    company: 'Tech Corp',
    title: 'Senior Software Engineer',
    location: 'San Francisco, CA',
    start_date: '2020-01-01',
    end_date: null,
    summary: 'Leading backend development',
    tech_stack: ['Rust', 'AWS', 'PostgreSQL'],
    sort_order: 1,
  },
]

export const mockCompetencies = [
  {
    id: 1,
    slug: 'rust-systems',
    name: 'Rust Systems Programming',
    description: 'Expert in Rust systems programming',
    icon: '🦀',
    sort_order: 1,
  },
]

export const mockAboutSections = [
  {
    id: 1,
    page: 'me',
    slug: 'background',
    heading: 'Background',
    body: 'I am a software engineer with 10 years of experience.',
    icon: null,
    sort_order: 1,
  },
]

export const mockSocialLinks = [
  {
    id: 1,
    platform: 'linkedin',
    url: 'https://linkedin.com/in/shantopagla',
    label: 'LinkedIn',
    icon: null,
    visible: true,
    sort_order: 1,
  },
  {
    id: 2,
    platform: 'github',
    url: 'https://github.com/shantopagla',
    label: 'GitHub',
    icon: null,
    visible: false,
    sort_order: 2,
  },
]

export const mockChallenges = [
  {
    id: 1,
    slug: 'portfolio-rag',
    title: 'Portfolio RAG System',
    job_id: 1,
    description: 'Built a RAG-powered portfolio',
    short_description: 'RAG portfolio',
    tech_stack: ['Rust', 'AI', 'SQLite'],
    category: 'technical',
    url: 'https://github.com/shantopagla/portfolio',
    featured: true,
    image_url: null,
    sort_order: 1,
  },
]

// MSW handlers for API endpoints
export const handlers = [
  // Auth endpoints
  http.get('/api/auth/me', () => {
    return HttpResponse.json({
      authenticated: true,
      email: 'test@example.com',
    })
  }),

  http.post('/api/auth/signin', () => {
    return HttpResponse.json({ success: true })
  }),

  http.post('/api/auth/forgot-password', () => {
    return HttpResponse.json({ success: true })
  }),

  http.post('/api/auth/confirm-forgot-password', () => {
    return HttpResponse.json({ success: true })
  }),

  http.post('/api/auth/respond-to-challenge', () => {
    return HttpResponse.json({ success: true })
  }),

  // Resume endpoint
  http.get('/api/resume', () => {
    return HttpResponse.json({
      name: 'Sharful Islam',
      title: 'AI Systems Engineer',
      bio: 'Software engineer specializing in AI systems',
      summary: '10 years of experience building scalable systems',
      jobs: mockJobs,
      competencies: mockCompetencies,
      challenges: mockChallenges,
      social_links: mockSocialLinks,
    })
  }),

  http.get('/api/jobs', () => {
    return HttpResponse.json(mockJobs)
  }),

  http.get('/api/jobs/:slug', ({ params }) => {
    const job = mockJobs.find(j => j.slug === params.slug)
    if (job) {
      return HttpResponse.json({
        ...job,
        details: [
          {
            id: 1,
            detail_text: 'Led backend development',
            category: 'responsibility',
            sort_order: 1,
          },
        ],
      })
    }
    return HttpResponse.json(null, { status: 404 })
  }),

  http.get('/api/competencies', () => {
    return HttpResponse.json(mockCompetencies)
  }),

  http.get('/api/competencies/:slug', ({ params }) => {
    const comp = mockCompetencies.find(c => c.slug === params.slug)
    if (comp) {
      return HttpResponse.json({
        ...comp,
        evidence: [
          {
            id: 1,
            job_id: 1,
            job_slug: 'senior-engineer',
            company: 'Tech Corp',
            highlight_text: 'Built Rust systems',
            detail_text: 'Developed high-performance Rust systems',
            detail_id: 1,
            sort_order: 1,
          },
        ],
      })
    }
    return HttpResponse.json(null, { status: 404 })
  }),

  // About sections
  http.get('/api/about/sections', ({ request }) => {
    const url = new URL(request.url)
    const page = url.searchParams.get('page')
    if (!page || page === 'me' || page === 'repo') {
      return HttpResponse.json(mockAboutSections)
    }
    return HttpResponse.json([])
  }),

  // Social links
  http.get('/api/social-links', () => {
    return HttpResponse.json(mockSocialLinks)
  }),

  // Challenges
  http.get('/api/challenges', () => {
    return HttpResponse.json(mockChallenges)
  }),

  http.get('/api/challenges/:slug', ({ params }) => {
    const challenge = mockChallenges.find(c => c.slug === params.slug)
    if (challenge) {
      return HttpResponse.json(challenge)
    }
    return HttpResponse.json(null, { status: 404 })
  }),

  // Contact form
  http.get('/api/contact/challenge', () => {
    return HttpResponse.json({
      nonce: 'test-nonce',
      difficulty: 1,
      timestamp: Date.now(),
      signature: 'test-signature',
    })
  }),

  http.post('/api/contact', () => {
    return HttpResponse.json({
      success: true,
      message: 'Message sent successfully',
    })
  }),

  // LinkedIn sync (admin)
  http.get('/api/v1/admin/linkedin/positions', () => {
    return HttpResponse.json([])
  }),

  http.get('/api/v1/admin/linkedin/projects', () => {
    return HttpResponse.json([])
  }),

  http.get('/api/v1/admin/linkedin/sync-log', () => {
    return HttpResponse.json([])
  }),

  // LinkedIn admin endpoints
  http.post('/api/v1/admin/linkedin/import', () => {
    return HttpResponse.json({
      positions_imported: 2,
      projects_imported: 1,
      positions_matched: 1,
      projects_matched: 0,
    })
  }),

  http.get('/api/v1/admin/linkedin/positions/:id/diff', () => {
    return HttpResponse.json({
      position: {
        id: 1,
        company: 'Tech Corp',
        title: 'Senior Engineer',
        location: 'SF',
        start_date: '2020-01',
        end_date: null,
        description: 'Backend work',
        mapped_job_id: 1,
        sync_status: 'diverged',
      },
      job_title: 'Sr. Engineer',
      job_company: 'Tech Corp',
      fields: [
        { field: 'title', linkedin_value: 'Senior Engineer', db_value: 'Sr. Engineer', differs: true },
        { field: 'company', linkedin_value: 'Tech Corp', db_value: 'Tech Corp', differs: false },
      ],
    })
  }),

  http.get('/api/v1/admin/linkedin/projects/:id/diff', () => {
    return HttpResponse.json({
      project: {
        id: 1,
        title: 'Portfolio RAG',
        description: 'Built a RAG system',
        url: null,
        start_date: '2026-01',
        end_date: null,
        associated_position: 'Tech Corp',
        mapped_challenge_id: 1,
        sync_status: 'diverged',
      },
      challenge_title: 'Portfolio RAG System',
      fields: [
        { field: 'title', linkedin_value: 'Portfolio RAG', db_value: 'Portfolio RAG System', differs: true },
      ],
    })
  }),

  http.put('/api/v1/admin/linkedin/positions/:id/map', () => {
    return new HttpResponse(null, { status: 200 })
  }),

  http.put('/api/v1/admin/linkedin/projects/:id/map', () => {
    return new HttpResponse(null, { status: 200 })
  }),

  http.put('/api/v1/admin/linkedin/positions/:id/status', () => {
    return new HttpResponse(null, { status: 200 })
  }),

  http.put('/api/v1/admin/linkedin/projects/:id/status', () => {
    return new HttpResponse(null, { status: 200 })
  }),

  // LinkedIn OAuth (agent service)
  http.get('/api/v1/agent/linkedin/status', () => {
    return HttpResponse.json({
      connected: false,
      name: null,
      email: null,
      picture_url: null,
      token_expires_at: null,
    })
  }),

  http.get('/api/v1/agent/linkedin/auth-url', () => {
    return HttpResponse.json({
      url: 'https://www.linkedin.com/oauth/v2/authorization?client_id=test&state=abc123',
      state: 'abc123',
    })
  }),

  http.post('/api/v1/agent/linkedin/disconnect', () => {
    return HttpResponse.json({ status: 'disconnected' })
  }),

  // Ask endpoint
  http.post('/api/ask', () => {
    return HttpResponse.json({
      answer: 'This is a test answer based on the portfolio.',
      citations: [
        {
          path: 'portfolio://README.md',
          kind: 'portfolio',
          sha: 'abc123',
          url: 'https://github.com/shantopagla/portfolio/blob/main/README.md',
          ord: 1,
        },
      ],
      model: 'claude-3-haiku-20240307',
      input_tokens: 50,
      output_tokens: 100,
    })
  }),
]