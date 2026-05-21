import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import userEvent from '@testing-library/user-event'
import AboutDetail from '../../../routes/dashboard/AboutDetail'
import DashboardLayout from '../../../routes/dashboard/Layout'

vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('AboutDetail', () => {
  it('renders new about section form', () => {
    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/new', routes: [{ path: '/dashboard/about/:id' }] }
    )
    expect(screen.getByText('New section')).toBeInTheDocument()
  })

  it('renders form fields', () => {
    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/new', routes: [{ path: '/dashboard/about/:id' }] }
    )
    expect(screen.getByLabelText('page')).toBeInTheDocument()
    expect(screen.getByLabelText('slug')).toBeInTheDocument()
    expect(screen.getByLabelText('heading')).toBeInTheDocument()
    expect(screen.getByLabelText('Body')).toBeInTheDocument()
    expect(screen.getByLabelText('Sort order')).toBeInTheDocument()
  })

  it('renders save button', () => {
    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/new', routes: [{ path: '/dashboard/about/:id' }] }
    )
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
  })

  it('submits form and navigates back', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response(null, { status: 200 })))
    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/new', routes: [{ path: '/dashboard/about/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('page'), 'me')
    await user.type(screen.getByLabelText('slug'), 'test-section')
    await user.type(screen.getByLabelText('Body'), 'Test body content')

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => {
      expect(screen.queryByText('New section')).not.toBeInTheDocument()
    })
  })

  it('shows error on save failure', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response('Save failed', { status: 400 })))
    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/new', routes: [{ path: '/dashboard/about/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('page'), 'me')
    await user.type(screen.getByLabelText('slug'), 'test-section')
    await user.type(screen.getByLabelText('Body'), 'Test body content')

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => {
      expect(screen.getByText('Save failed')).toBeInTheDocument()
    })
  })

  it('shows loading state for existing section', async () => {
    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/1', routes: [{ path: '/dashboard/about/:id' }] }
    )
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('deletes existing section and navigates back', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response(null, { status: 200 })))
    global.confirm = vi.fn(() => true)

    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/1', routes: [{ path: '/dashboard/about/:id' }] }
    )
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    const deleteButton = screen.getByRole('button', { name: 'Delete' })
    await user.click(deleteButton)

    await waitFor(() => {
      expect(global.confirm).toHaveBeenCalledWith('Delete this section?')
    })
  })

  it('shows delete error when fetch fails', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response('Delete failed', { status: 500 })))
    global.confirm = vi.fn(() => true)

    render(
      <DashboardLayout>
        <AboutDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/about/1', routes: [{ path: '/dashboard/about/:id' }] }
    )
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    const deleteButton = screen.getByRole('button', { name: 'Delete' })
    await user.click(deleteButton)

    await waitFor(() => {
      expect(screen.getByText('Delete failed')).toBeInTheDocument()
    })
  })
})