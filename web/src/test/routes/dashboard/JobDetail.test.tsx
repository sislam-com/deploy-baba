import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor, act } from '../../utils/test-render'
import userEvent from '@testing-library/user-event'
import JobDetail from '../../../routes/dashboard/JobDetail'
import DashboardLayout from '../../../routes/dashboard/Layout'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('JobDetail', () => {
  it('renders new job form', () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    expect(screen.getByText('New job')).toBeInTheDocument()
  })

  it('renders edit job form', async () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/1', routes: [{ path: '/dashboard/jobs/:id' }] }
    )

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText('Edit job')).toBeInTheDocument()
  })

  it('renders back link', () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    const backLink = screen.getByText('← Jobs')
    expect(backLink).toBeInTheDocument()
    expect(backLink).toHaveAttribute('href', '/dashboard/jobs')
  })

  it('renders form fields', () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    expect(screen.getByLabelText('slug')).toBeInTheDocument()
    expect(screen.getByLabelText('company')).toBeInTheDocument()
    expect(screen.getByLabelText('title')).toBeInTheDocument()
    expect(screen.getByLabelText('location')).toBeInTheDocument()
    expect(screen.getByLabelText('start date')).toBeInTheDocument()
    expect(screen.getByLabelText('end date')).toBeInTheDocument()
    expect(screen.getByLabelText('Summary')).toBeInTheDocument()
    expect(screen.getByLabelText('Tech stack (comma-separated)')).toBeInTheDocument()
    expect(screen.getByLabelText('Sort order')).toBeInTheDocument()
  })

  it('renders save button', () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
  })

  it('renders delete button for existing job', async () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/1', routes: [{ path: '/dashboard/jobs/:id' }] }
    )

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument()
  })

  it('does not render delete button for new job', () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument()
  })

  it('requires required fields', () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    const slugInput = screen.getByLabelText('slug')
    expect(slugInput).toBeRequired()
  })

  it('submits form and navigates back', async () => {
    global.fetch = vi.fn(() =>
      Promise.resolve(new Response('Success', { status: 200 }))
    )

    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('slug'), 'test-job')
    await user.type(screen.getByLabelText('company'), 'Test Company')
    await user.type(screen.getByLabelText('title'), 'Test Title')
    await user.type(screen.getByLabelText('start date'), '2020-01-01')

    const saveButton = screen.getByRole('button', { name: 'Save' })
    await user.click(saveButton)

    await waitFor(() => {
      // Navigation unmounts the new-job form
      expect(screen.queryByText('New job')).not.toBeInTheDocument()
    })
  })

  it('shows error message on save failure', async () => {
    global.fetch = vi.fn(() =>
      Promise.resolve(new Response('Save failed', { status: 400 }))
    )

    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('slug'), 'test-job')
    await user.type(screen.getByLabelText('company'), 'Test Company')
    await user.type(screen.getByLabelText('title'), 'Test Title')
    await user.type(screen.getByLabelText('start date'), '2020-01-01')

    const saveButton = screen.getByRole('button', { name: 'Save' })
    await user.click(saveButton)

    await waitFor(() => {
      expect(screen.getByText('Save failed')).toBeInTheDocument()
    })
  })

  it('deletes job and navigates back', async () => {
    global.fetch = vi.fn(() =>
      Promise.resolve(new Response(null, { status: 200 }))
    )

    // Mock confirm
    global.confirm = vi.fn(() => true)

    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/1', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    const deleteButton = screen.getByRole('button', { name: 'Delete' })
    await user.click(deleteButton)

    await waitFor(() => {
      expect(global.confirm).toHaveBeenCalledWith('Delete this job?')
    })
  })

  it('does not delete when confirmation is cancelled', async () => {
    // Mock confirm to return false
    global.confirm = vi.fn(() => false)

    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/1', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    const deleteButton = screen.getByRole('button', { name: 'Delete' })
    await user.click(deleteButton)

    expect(global.confirm).toHaveBeenCalledWith('Delete this job?')
  })

  it('loads existing job data', async () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/1', routes: [{ path: '/dashboard/jobs/:id' }] }
    )

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByLabelText('slug')).toHaveValue('senior-engineer')
    expect(screen.getByLabelText('company')).toHaveValue('Tech Corp')
  })

  it('shows loading state initially', () => {
    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/1', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('disables save button while saving', async () => {
    let resolvePromise: (value: Response) => void
    global.fetch = vi.fn(() => new Promise<Response>(resolve => {
      resolvePromise = resolve
    }))

    render(
      <DashboardLayout>
        <JobDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/jobs/new', routes: [{ path: '/dashboard/jobs/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('slug'), 'test-job')
    await user.type(screen.getByLabelText('company'), 'Test Company')
    await user.type(screen.getByLabelText('title'), 'Test Title')
    await user.type(screen.getByLabelText('start date'), '2020-01-01')

    const saveButton = screen.getByRole('button', { name: 'Save' })
    await user.click(saveButton)

    expect(saveButton).toBeDisabled()
    expect(saveButton).toHaveTextContent('Saving…')

    // Cleanup: resolve and let React finish state updates inside act()
    await act(async () => {
      resolvePromise!(new Response('Success', { status: 200 }))
    })
  })
})