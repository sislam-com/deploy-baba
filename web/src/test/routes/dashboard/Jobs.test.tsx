import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import Jobs from '../../../routes/dashboard/Jobs'
import DashboardLayout from '../../../routes/dashboard/Layout'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('Jobs', () => {
  it('renders jobs heading', () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )
    expect(screen.getByRole('heading', { name: 'Jobs' })).toBeInTheDocument()
  })

  it('renders new job button', () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )
    const newButton = screen.getByText('+ New job')
    expect(newButton).toBeInTheDocument()
    expect(newButton).toHaveAttribute('href', '/dashboard/jobs/new')
  })

  it('renders loading state initially', () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders jobs list', async () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText('Senior Software Engineer')).toBeInTheDocument()
    expect(screen.getByText(/Tech Corp/)).toBeInTheDocument()
  })

  it('renders job title', async () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      expect(screen.getByText('Senior Software Engineer')).toBeInTheDocument()
    })
  })

  it('renders company name', async () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      expect(screen.getByText(/Tech Corp/)).toBeInTheDocument()
    })
  })

  it('renders location', async () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      expect(screen.getByText(/San Francisco, CA/)).toBeInTheDocument()
    })
  })

  it('renders date range', async () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      expect(screen.getByText('2020-01-01 – Present')).toBeInTheDocument()
    })
  })

  it('renders job cards as links', async () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      const jobLink = screen.getByText('Senior Software Engineer').closest('a')
      expect(jobLink).toHaveAttribute('href', '/dashboard/jobs/1')
    })
  })

  it('applies correct styling to job cards', async () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      const card = screen.getByText('Senior Software Engineer').closest('a')
      expect(card).toHaveClass('bg-gray-800', 'border', 'border-gray-700', 'hover:border-gray-500', 'rounded-xl')
    })
  })

  it('applies correct styling to new job button', () => {
    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    const newButton = screen.getByText('+ New job')
    expect(newButton).toHaveClass('bg-cyan-600', 'hover:bg-cyan-500', 'text-white', 'font-semibold', 'px-4', 'py-2', 'rounded-lg')
  })

  it('shows error state on fetch failure', async () => {
    global.fetch = vi.fn(() =>
      Promise.reject(new Error('API Error'))
    )

    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      expect(screen.getByText('Failed to load jobs')).toBeInTheDocument()
    })
  })

  it('renders empty state when no jobs', async () => {
    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve([]),
      })
    )

    render(
      <DashboardLayout>
        <Jobs />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs' }
    )

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    // Should not show any job cards
    expect(screen.queryByText('Senior Software Engineer')).not.toBeInTheDocument()
  })
})