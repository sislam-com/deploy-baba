import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import About from '../../../routes/dashboard/About'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('About', () => {
  it('renders about sections heading', () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })
    expect(screen.getByText('About sections')).toBeInTheDocument()
  })

  it('renders new section button', () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })
    const newButton = screen.getByText('+ New section')
    expect(newButton).toBeInTheDocument()
    expect(newButton).toHaveAttribute('href', '/dashboard/about/new')
  })

  it('renders loading state initially', () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders about sections list', async () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText('Background')).toBeInTheDocument()
  })

  it('renders section heading', async () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })

    await waitFor(() => {
      expect(screen.getByText('Background')).toBeInTheDocument()
    })
  })

  it('renders section slug', async () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })

    await waitFor(() => {
      expect(screen.getByText('background')).toBeInTheDocument()
    })
  })

  it('renders section page', async () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })

    await waitFor(() => {
      expect(screen.getByText('me')).toBeInTheDocument()
    })
  })

  it('renders section sort order', async () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })

    await waitFor(() => {
      expect(screen.getByText('#1')).toBeInTheDocument()
    })
  })

  it('renders section cards as links', async () => {
    render(<About />, { router: 'memory', route: '/dashboard/about' })

    await waitFor(() => {
      const card = screen.getByText('Background').closest('a')
      expect(card).toHaveAttribute('href', '/dashboard/about/1')
    })
  })

  it('shows error state on fetch failure', async () => {
    global.fetch = vi.fn(() =>
      Promise.reject(new Error('API Error'))
    )

    render(<About />, { router: 'memory', route: '/dashboard/about' })

    await waitFor(() => {
      expect(screen.getByText('Failed to load about sections')).toBeInTheDocument()
    })
  })
})