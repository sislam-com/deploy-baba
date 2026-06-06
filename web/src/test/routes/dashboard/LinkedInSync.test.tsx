import { describe, it, expect, vi } from 'vitest'
import { fireEvent } from '@testing-library/react'
import { render, screen, waitFor, userEvent } from '../../utils/test-render'
// fireEvent is used for textarea value changes (userEvent treats [ and { as special keys)
import { http, HttpResponse } from 'msw'
import { server } from '../../mocks/server'
import LinkedInSync from '../../../routes/dashboard/LinkedInSync'
import DashboardLayout from '../../../routes/dashboard/Layout'

vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

function renderSync(route = '/dashboard/linkedin') {
  return render(
    <DashboardLayout>
      <LinkedInSync />
    </DashboardLayout>,
    { router: 'memory', route }
  )
}

describe('LinkedInSync', () => {
  it('renders page heading', async () => {
    renderSync()
    expect(screen.getByRole('heading', { name: 'LinkedIn Sync' })).toBeInTheDocument()
  })

  it('renders LinkedIn Connection section', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.getByText('LinkedIn Connection')).toBeInTheDocument()
    })
  })

  it('shows Connect LinkedIn button when disconnected', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Connect LinkedIn')).toBeInTheDocument()
    })
  })

  it('shows description text when disconnected', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Sign in with LinkedIn to verify your profile identity.')).toBeInTheDocument()
    })
  })

  it('shows connected state with profile info', async () => {
    server.use(
      http.get('/api/v1/agent/linkedin/status', () => {
        return HttpResponse.json({
          connected: true,
          name: 'Test User',
          email: 'test@linkedin.com',
          picture_url: null,
          token_expires_at: String(Math.floor(Date.now() / 1000) + 3600),
        })
      })
    )

    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Test User')).toBeInTheDocument()
      expect(screen.getByText('test@linkedin.com')).toBeInTheDocument()
      expect(screen.getByText('Connected')).toBeInTheDocument()
      expect(screen.getByText('Disconnect')).toBeInTheDocument()
    })
  })

  it('renders Import LinkedIn Data section', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Import LinkedIn Data')).toBeInTheDocument()
    })
  })

  it('renders Upload CSV button', () => {
    renderSync()
    expect(screen.getByText('Upload CSV')).toBeInTheDocument()
  })

  it('renders JSON textarea', () => {
    renderSync()
    const textarea = screen.getByPlaceholderText('{"positions": [...], "projects": [...]}')
    expect(textarea).toBeInTheDocument()
  })

  it('renders Import JSON button disabled when empty', () => {
    renderSync()
    const btn = screen.getByText('Import JSON')
    expect(btn).toBeDisabled()
  })

  it('renders Positions and Projects tabs', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Positions (0)')).toBeInTheDocument()
      expect(screen.getByText('Projects (0)')).toBeInTheDocument()
    })
  })

  it('shows empty positions message', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.getByText('No LinkedIn positions imported yet.')).toBeInTheDocument()
    })
  })

  it('shows empty projects message on tab switch', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument()
    })

    const user = userEvent.setup()
    await user.click(screen.getByText('Projects (0)'))
    expect(screen.getByText('No LinkedIn projects imported yet.')).toBeInTheDocument()
  })

  it('renders positions when data exists', async () => {
    server.use(
      http.get('/api/v1/admin/linkedin/positions', () => {
        return HttpResponse.json([
          {
            id: 1,
            company: 'Acme Corp',
            title: 'Lead Dev',
            location: 'NYC',
            start_date: '2022-01',
            end_date: null,
            sync_status: 'synced',
            mapped_job_id: 1,
            imported_at: '2026-05-25',
          },
        ])
      })
    )

    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Lead Dev')).toBeInTheDocument()
      expect(screen.getByText(/Acme Corp/)).toBeInTheDocument()
      expect(screen.getByText('synced')).toBeInTheDocument()
    })
  })

  it('handles disconnect', async () => {
    server.use(
      http.get('/api/v1/agent/linkedin/status', () => {
        return HttpResponse.json({
          connected: true,
          name: 'Test User',
          email: 'test@linkedin.com',
          picture_url: null,
          token_expires_at: String(Math.floor(Date.now() / 1000) + 3600),
        })
      })
    )

    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Disconnect')).toBeInTheDocument()
    })

    const user = userEvent.setup()
    await user.click(screen.getByText('Disconnect'))
    await waitFor(() => {
      expect(screen.getByText('Connect LinkedIn')).toBeInTheDocument()
    })
  })

  it('handles JSON import success', async () => {
    renderSync()
    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument()
    })

    const textarea = screen.getByPlaceholderText('{"positions": [...], "projects": [...]}')
    fireEvent.change(textarea, { target: { value: '{"positions":[],"projects":[]}' } })

    const user = userEvent.setup()
    const btn = screen.getByText('Import JSON')
    expect(btn).not.toBeDisabled()
    await user.click(btn)

    await waitFor(() => {
      expect(screen.getByText(/Imported 2 positions/)).toBeInTheDocument()
    })
  })

  it('shows error on fetch failure', async () => {
    server.use(
      http.get('/api/v1/admin/linkedin/positions', () => {
        return HttpResponse.error()
      }),
      http.get('/api/v1/admin/linkedin/projects', () => {
        return HttpResponse.error()
      }),
      http.get('/api/v1/admin/linkedin/sync-log', () => {
        return HttpResponse.error()
      })
    )

    renderSync()
    await waitFor(() => {
      expect(screen.getByText('Failed to load LinkedIn data')).toBeInTheDocument()
    })
  })
})
