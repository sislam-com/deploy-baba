import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import Challenges from '../../../routes/dashboard/Challenges'
import DashboardLayout from '../../../routes/dashboard/Layout'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('Challenges', () => {
  it('renders challenges heading', () => {
    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )
    expect(screen.getByRole('heading', { name: 'Challenges' })).toBeInTheDocument()
  })

  it('renders new challenge button', () => {
    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )
    const newButton = screen.getByText('+ New challenge')
    expect(newButton).toBeInTheDocument()
    expect(newButton).toHaveAttribute('href', '/dashboard/challenges/new')
  })

  it('renders loading state initially', () => {
    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders challenges list', async () => {
    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText('Portfolio RAG System')).toBeInTheDocument()
  })

  it('renders challenge title', async () => {
    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )

    await waitFor(() => {
      expect(screen.getByText('Portfolio RAG System')).toBeInTheDocument()
    })
  })

  it('renders challenge short description', async () => {
    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )

    await waitFor(() => {
      expect(screen.getByText('RAG portfolio')).toBeInTheDocument()
    })
  })

  it('renders challenge cards as links', async () => {
    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )

    await waitFor(() => {
      const card = screen.getByText('Portfolio RAG System').closest('a')
      expect(card).toHaveAttribute('href', '/dashboard/challenges/1')
    })
  })

  it('shows error state on fetch failure', async () => {
    global.fetch = vi.fn(() =>
      Promise.reject(new Error('API Error'))
    )

    render(
      <DashboardLayout>
        <Challenges />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/challenges' }
    )

    await waitFor(() => {
      expect(screen.getByText('Failed to load challenges')).toBeInTheDocument()
    })
  })
})