import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import SocialLinks from '../../../routes/dashboard/SocialLinks'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('SocialLinks', () => {
  it('renders social links heading', () => {
    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })
    expect(screen.getByText('Social Links')).toBeInTheDocument()
  })

  it('renders new link button', () => {
    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })
    const newButton = screen.getByText('+ New link')
    expect(newButton).toBeInTheDocument()
    expect(newButton).toHaveAttribute('href', '/dashboard/social-links/new')
  })

  it('renders loading state initially', () => {
    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders social links list', async () => {
    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText('LinkedIn')).toBeInTheDocument()
    expect(screen.getByText('GitHub')).toBeInTheDocument()
  })

  it('renders link label', async () => {
    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })

    await waitFor(() => {
      expect(screen.getByText('LinkedIn')).toBeInTheDocument()
    })
  })

  it('renders link URL', async () => {
    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })

    await waitFor(() => {
      expect(screen.getByText('https://linkedin.com/in/shantopagla')).toBeInTheDocument()
    })
  })

  it('renders link cards as links', async () => {
    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })

    await waitFor(() => {
      const card = screen.getByText('LinkedIn').closest('a')
      expect(card).toHaveAttribute('href', '/dashboard/social-links/1')
    })
  })

  it('shows error state on fetch failure', async () => {
    global.fetch = vi.fn(() =>
      Promise.reject(new Error('API Error'))
    )

    render(<SocialLinks />, { router: 'memory', route: '/dashboard/social-links' })

    await waitFor(() => {
      expect(screen.getByText('Failed to load social links')).toBeInTheDocument()
    })
  })
})