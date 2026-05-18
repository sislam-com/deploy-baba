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
})