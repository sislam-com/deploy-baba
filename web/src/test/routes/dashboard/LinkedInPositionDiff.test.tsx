import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor, userEvent } from '../../utils/test-render'
import { http, HttpResponse } from 'msw'
import { server } from '../../mocks/server'
import LinkedInPositionDiff from '../../../routes/dashboard/LinkedInPositionDiff'

vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

function renderDiff(id = '1') {
  return render(<LinkedInPositionDiff />, {
    router: 'memory',
    route: `/dashboard/linkedin/positions/${id}`,
    routes: [
      { path: '/dashboard/linkedin/positions/:id' },
      { path: '/dashboard/linkedin' },
    ],
  })
}

describe('LinkedInPositionDiff', () => {
  it('renders loading state initially', () => {
    renderDiff()
    expect(screen.getByText('Loading...')).toBeInTheDocument()
  })

  it('renders position title and company', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Senior Engineer' })).toBeInTheDocument()
      const corps = screen.getAllByText('Tech Corp')
      expect(corps.length).toBeGreaterThanOrEqual(1)
    })
  })

  it('renders back link', async () => {
    renderDiff()
    await waitFor(() => {
      const link = screen.getByText(/Back to LinkedIn Sync/)
      expect(link).toBeInTheDocument()
      expect(link).toHaveAttribute('href', '/dashboard/linkedin')
    })
  })

  it('renders field diff table with differences', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('title')).toBeInTheDocument()
      expect(screen.getByText('Sr. Engineer')).toBeInTheDocument()
      const seniors = screen.getAllByText('Senior Engineer')
      expect(seniors.length).toBeGreaterThanOrEqual(2)
    })
  })

  it('renders mapping section with job dropdown', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Map to Internal Job')).toBeInTheDocument()
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
      http.get('/api/v1/admin/linkedin/positions/:id/diff', () => {
        return HttpResponse.json(null, { status: 404 })
      })
    )

    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('Position not found')).toBeInTheDocument()
    })
  })

  it('populates job dropdown from /api/jobs', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText(/Senior Software Engineer/)).toBeInTheDocument()
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

  it('handles linkedin only click', async () => {
    renderDiff()
    await waitFor(() => {
      expect(screen.getByText('LinkedIn Only')).toBeInTheDocument()
    })
    const user = userEvent.setup()
    await user.click(screen.getByText('LinkedIn Only'))
  })

  it('shows unmapped message when no job mapped', async () => {
    server.use(
      http.get('/api/v1/admin/linkedin/positions/:id/diff', () => {
        return HttpResponse.json({
          position: {
            id: 2,
            company: 'Startup Inc',
            title: 'Dev',
            location: null,
            start_date: '2024-01',
            end_date: null,
            description: null,
            mapped_job_id: null,
            sync_status: 'unreviewed',
          },
          job_title: null,
          job_company: null,
          fields: [],
        })
      })
    )
    renderDiff('2')
    await waitFor(() => {
      expect(screen.getByText(/map this position to an internal job/i)).toBeInTheDocument()
    })
  })
})
