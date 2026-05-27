import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor, userEvent } from '../../utils/test-render'
import { http, HttpResponse } from 'msw'
import { server } from '../../mocks/server'
import LinkedInProjectDiff from '../../../routes/dashboard/LinkedInProjectDiff'

vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

function renderDiff(id = '1') {
  return render(<LinkedInProjectDiff />, {
    router: 'memory',
    route: `/dashboard/linkedin/projects/${id}`,
    routes: [
      { path: '/dashboard/linkedin/projects/:id' },
      { path: '/dashboard/linkedin' },
    ],
  })
}

describe('LinkedInProjectDiff', () => {
  it('renders loading state initially', () => {
    renderDiff()
    expect(screen.getByText('Loading...')).toBeInTheDocument()
  })

  it('renders project title and associated position', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Portfolio RAG' })).toBeInTheDocument()
      expect(screen.getByText('Tech Corp')).toBeInTheDocument()
    })
  })

  it('renders back link', async () => {
    renderDiff()
    await waitFor(() => {
      const link = screen.getByText(/Back to LinkedIn Sync/)
      expect(link).toHaveAttribute('href', '/dashboard/linkedin')
    })
  })

  it('renders field diff table', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('title')).toBeInTheDocument()
      const rags = screen.getAllByText('Portfolio RAG')
      expect(rags.length).toBeGreaterThanOrEqual(2)
      const systems = screen.getAllByText('Portfolio RAG System')
      expect(systems.length).toBeGreaterThanOrEqual(1)
    })
  })

  it('renders mapping section with challenge dropdown', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Map to Internal Challenge')).toBeInTheDocument()
      expect(screen.getByText('Save Mapping')).toBeInTheDocument()
    })
  })

  it('renders status action buttons', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Mark as Synced')).toBeInTheDocument()
      expect(screen.getByText('Mark as Diverged')).toBeInTheDocument()
      expect(screen.getByText('LinkedIn Only')).toBeInTheDocument()
    })
  })

  it('shows error on fetch failure', async () => {
    server.use(
      http.get('/api/v1/admin/linkedin/projects/:id/diff', () => {
        return HttpResponse.json(null, { status: 404 })
      })
    )

    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Project not found')).toBeInTheDocument()
    })
  })

  it('populates challenge dropdown from /api/challenges', async () => {
    renderDiff()
    await waitFor(() => {
      const options = screen.getAllByText('Portfolio RAG System')
      expect(options.length).toBeGreaterThanOrEqual(1)
    })
  })

  it('handles save mapping click', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Save Mapping')).toBeInTheDocument()
    })
    const user = userEvent.setup()
    await user.click(screen.getByText('Save Mapping'))
  })

  it('handles mark as synced click', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Mark as Synced')).toBeInTheDocument()
    })
    const user = userEvent.setup()
    await user.click(screen.getByText('Mark as Synced'))
  })

  it('handles mark as diverged click', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Mark as Diverged')).toBeInTheDocument()
    })
    const user = userEvent.setup()
    await user.click(screen.getByText('Mark as Diverged'))
  })

  it('shows unmapped message when no challenge mapped', async () => {
    server.use(
      http.get('/api/v1/admin/linkedin/projects/:id/diff', () => {
        return HttpResponse.json({
          project: {
            id: 2,
            title: 'Side Project',
            description: null,
            url: null,
            start_date: null,
            end_date: null,
            associated_position: null,
            mapped_challenge_id: null,
            sync_status: 'unreviewed',
          },
          challenge_title: null,
          fields: [],
        })
      })
    )
    renderDiff('2')
    await waitFor(() => {
      expect(screen.getByText(/map this project to an internal challenge/i)).toBeInTheDocument()
    })
  })
})
