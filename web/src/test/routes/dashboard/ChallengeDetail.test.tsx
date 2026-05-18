import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import userEvent from '@testing-library/user-event'
import ChallengeDetail from '../../../routes/dashboard/ChallengeDetail'
import DashboardLayout from '../../../routes/dashboard/Layout'

vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('ChallengeDetail', () => {
  it('renders new challenge form', () => {
    render(
      <DashboardLayout>
        <ChallengeDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges/new', routes: [{ path: '/dashboard/challenges/:id' }] }
    )
    expect(screen.getByText('New Challenge')).toBeInTheDocument()
  })

  it('renders form fields', () => {
    render(
      <DashboardLayout>
        <ChallengeDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges/new', routes: [{ path: '/dashboard/challenges/:id' }] }
    )
    expect(screen.getByLabelText('slug')).toBeInTheDocument()
    expect(screen.getByLabelText('title')).toBeInTheDocument()
    expect(screen.getByLabelText('short description')).toBeInTheDocument()
    expect(screen.getByLabelText('Description')).toBeInTheDocument()
    expect(screen.getByLabelText('tech stack')).toBeInTheDocument()
    expect(screen.getByLabelText('category')).toBeInTheDocument()
    expect(screen.getByLabelText('url')).toBeInTheDocument()
    expect(screen.getByLabelText('Featured on homepage')).toBeInTheDocument()
    expect(screen.getByLabelText('Sort order')).toBeInTheDocument()
  })

  it('renders save button', () => {
    render(
      <DashboardLayout>
        <ChallengeDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges/new', routes: [{ path: '/dashboard/challenges/:id' }] }
    )
    expect(screen.getByRole('button', { name: 'Create' })).toBeInTheDocument()
  })

  it('renders delete button for existing challenge', async () => {
    render(
      <DashboardLayout>
        <ChallengeDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges/1', routes: [{ path: '/dashboard/challenges/:id' }] }
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
        <ChallengeDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges/new', routes: [{ path: '/dashboard/challenges/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('slug'), 'test-challenge')
    await user.type(screen.getByLabelText('title'), 'Test Challenge')
    await user.type(screen.getByLabelText('Description'), 'Test description')
    await user.type(screen.getByLabelText('Sort order'), '1')

    await user.click(screen.getByRole('button', { name: 'Create' }))
    await waitFor(() => {
      expect(screen.queryByText('New Challenge')).not.toBeInTheDocument()
    })
  })
})