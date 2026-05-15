import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '../utils/test-render'
import Layout from '../../components/Layout'

// Mock the useAuth hook since Layout doesn't use it directly but might be used in auth contexts
vi.mock('../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: false, email: null }),
}))

describe('Layout', () => {
  it('renders navigation with brand link', () => {
    render(<Layout />)
    const brandLink = screen.getByText('Sharful Islam')
    expect(brandLink).toBeInTheDocument()
    expect(brandLink).toHaveAttribute('href', '/')
  })

  it('renders navigation icons', () => {
    render(<Layout />)
    expect(screen.getByLabelText('About')).toBeInTheDocument()
    expect(screen.getByLabelText('Contact')).toBeInTheDocument()
    expect(screen.getByLabelText('API Docs')).toBeInTheDocument()
    expect(screen.getByLabelText('Login')).toBeInTheDocument()
  })

  it('renders footer with copyright', () => {
    render(<Layout />)
    expect(screen.getByText(/© 2026 Sharful Islam/)).toBeInTheDocument()
  })

  it('renders footer with license information', () => {
    render(<Layout />)
    expect(screen.getByText(/Dual-licensed under MIT or Apache-2.0/)).toBeInTheDocument()
  })

  it('renders footer with GitHub link', () => {
    render(<Layout />)
    const githubLink = screen.getByText('GitHub Repository')
    expect(githubLink).toBeInTheDocument()
    expect(githubLink).toHaveAttribute('href', 'https://github.com/shantopagla/deploy-baba')
  })

  it('renders mobile menu button', () => {
    render(<Layout />)
    const menuButton = screen.getByLabelText('Open menu')
    expect(menuButton).toBeInTheDocument()
  })

  it('toggles mobile menu when button is clicked', async () => {
    render(<Layout />)
    const menuButton = screen.getByLabelText('Open menu')

    fireEvent.click(menuButton)

    await waitFor(() => {
      const closeButton = screen.queryByLabelText('Close menu')
      expect(closeButton).toBeInTheDocument()
    })

    // Check that mobile menu items are visible
    expect(screen.getByText('About')).toBeVisible()
    expect(screen.getByText('Contact')).toBeVisible()
  })

  it('closes mobile menu when clicking menu item', async () => {
    render(<Layout />)
    const menuButton = screen.getByLabelText('Open menu')

    fireEvent.click(menuButton)

    await waitFor(() => {
      expect(screen.getByLabelText('Close menu')).toBeInTheDocument()
    })

    const aboutLink = screen.getByText('About')
    fireEvent.click(aboutLink)

    await waitFor(() => {
      expect(screen.queryByLabelText('Close menu')).not.toBeInTheDocument()
    })
  })

  it('fetches and displays social links', async () => {
    render(<Layout />)

    await waitFor(() => {
      // Social links should be fetched from API via MSW
      const socialIcons = screen.getAllByLabelText(/LinkedIn|GitHub/)
      expect(socialIcons.length).toBeGreaterThan(0)
    })
  })

  it('renders main content outlet', () => {
    const { container } = render(
      <Layout>
        <div>Test Content</div>
      </Layout>
    )
    expect(screen.getByText('Test Content')).toBeInTheDocument()
  })

  it('applies correct styling classes', () => {
    render(<Layout />)
    const nav = screen.getByRole('navigation')
    expect(nav).toHaveClass('border-b', 'border-gray-800', 'bg-gray-900/80', 'backdrop-blur-sm', 'sticky', 'top-0', 'z-50')
  })
})