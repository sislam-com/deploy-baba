import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import userEvent from '@testing-library/user-event'
import CompetencyDetail from '../../../routes/dashboard/CompetencyDetail'
import DashboardLayout from '../../../routes/dashboard/Layout'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('CompetencyDetail', () => {
  it('renders new competency form', () => {
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/new', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    expect(screen.getByText('New competency')).toBeInTheDocument()
  })

  it('renders edit competency form', async () => {
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/1', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })
    // Should render edit form with existing data
    expect(screen.getByLabelText('slug')).toBeInTheDocument()
  })

  it('renders form fields', () => {
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/new', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    expect(screen.getByLabelText('slug')).toBeInTheDocument()
    expect(screen.getByLabelText('name')).toBeInTheDocument()
    expect(screen.getByLabelText('icon')).toBeInTheDocument()
    expect(screen.getByLabelText('Description')).toBeInTheDocument()
    expect(screen.getByLabelText('Sort order')).toBeInTheDocument()
  })

  it('renders save button', () => {
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/new', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
  })

  it('renders delete button for existing competency', async () => {
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/1', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument()
  })

  it('submits form and navigates back', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response(null, { status: 200 })))
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/new', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('slug'), 'test-competency')
    await user.type(screen.getByLabelText('name'), 'Test Competency')
    await user.type(screen.getByLabelText('Sort order'), '1')

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => {
      expect(screen.queryByText('New competency')).not.toBeInTheDocument()
    })
  })

  it('shows error on save failure', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response('Save failed', { status: 400 })))
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/new', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('slug'), 'test-competency')
    await user.type(screen.getByLabelText('name'), 'Test Competency')

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => {
      expect(screen.getByText('Save failed')).toBeInTheDocument()
    })
  })

  it('shows loading state for existing competency', async () => {
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/1', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('cancels delete when confirmation is rejected', async () => {
    global.confirm = vi.fn(() => false)
    render(
      <DashboardLayout>
        <CompetencyDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/competencies/1', routes: [{ path: '/dashboard/competencies/:id' }] }
    )
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    const deleteButton = screen.getByRole('button', { name: 'Delete' })
    await user.click(deleteButton)

    expect(global.confirm).toHaveBeenCalledWith('Delete this competency?')
  })
})