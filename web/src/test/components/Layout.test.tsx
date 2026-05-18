import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor, within } from '../utils/test-render'
import Layout from '../../components/Layout'

// Mock the useAuth hook since Layout doesn't use it directly but might be used in auth contexts
vi.mock('../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: false, email: null }),
}))

describe('Layout', () => {
  it('renders navigation with brand link', () => {
    render(<Layout />, { router: 'memory', route: '/' })
    const nav = screen.getByRole('navigation')
    const brandLink = within(nav).getByText('Sharful Islam')
    expect(brandLink).toBeInTheDocument()
    expect(brandLink).toHaveAttribute('href', '/')
  })

  it('renders navigation icons', () => {
    render(<Layout />, { router: 'memory', route: '/' })
    expect(screen.getByLabelText('About')).toBeInTheDocument()
    expect(screen.getByLabelText('Contact')).toBeInTheDocument()
    expect(screen.getByLabelText('API Docs')).toBeInTheDocument()
    expect(screen.getByLabelText('Login')).toBeInTheDocument()
  })

  it('renders footer with copyright', () => {
    render(<Layout />, { router: 'memory', route: '/' })
    expect(screen.getByText(/© 2026 Sharful Islam/)).toBeInTheDocument()
  })

  it('renders footer with license information', () => {
    render(<Layout />, { router: 'memory', route: '/' })
    expect(screen.getByText(/Dual-licensed under MIT or Apache-2.0/)).toBeInTheDocument()
  })

  it('renders footer with GitHub link', () => {
    render(<Layout />, { router: 'memory', route: '/' })
    const githubLink = screen.getByText('GitHub Repository')
    expect(githubLink).toBeInTheDocument()
    expect(githubLink).toHaveAttribute('href', 'https://github.com/shantopagla/deploy-baba')
  })

  it('renders mobile menu button', () => {
    render(<Layout />, { router: 'memory', route: '/' })
    const menuButton = screen.getByLabelText('Open menu')
    expect(menuButton).toBeInTheDocument()
  })

  it('toggles mobile menu when button is clicked', async () => {
    render(<Layout />, { router: 'memory', route: '/' })
    const menuButton = screen.getByLabelText('Open menu')

    fireEvent.click(menuButton)

    await waitFor(() => {
      expect(screen.queryByLabelText('Close menu')).toBeInTheDocument()
    })

    // Mobile panel items should now be visible
    const aboutLinks = screen.getAllByText('About')
    const visibleAbout = aboutLinks.find(el => el.closest('.sm\\:hidden'))
    expect(visibleAbout).toBeTruthy()
  })

  it('closes mobile menu when clicking menu item', async () => {
    render(<Layout />, { router: 'memory', route: '/' })
    const menuButton = screen.getByLabelText('Open menu')

    fireEvent.click(menuButton)

    await waitFor(() => {
      expect(screen.getByLabelText('Close menu')).toBeInTheDocument()
    })

    // Click the About link inside the mobile panel (hidden on sm screens)
    const aboutLinks = screen.getAllByText('About')
    const mobileAbout = aboutLinks.find(el => el.closest('.sm\\:hidden'))
    expect(mobileAbout).toBeTruthy()
    if (mobileAbout) {
      fireEvent.click(mobileAbout)
    }

    await waitFor(() => {
      expect(screen.queryByLabelText('Close menu')).not.toBeInTheDocument()
    })
  })

  it('fetches and displays social links', async () => {
    render(<Layout />, { router: 'memory', route: '/' })

    await waitFor(() => {
      // Social links should be fetched from API via MSW
      const socialIcons = screen.getAllByLabelText(/LinkedIn|GitHub/)
      expect(socialIcons.length).toBeGreaterThan(0)
    })
  })

  it('applies correct styling classes', () => {
    render(<Layout />, { router: 'memory', route: '/' })
    const nav = screen.getByRole('navigation')
    expect(nav).toHaveClass('border-b', 'border-gray-800', 'bg-gray-900/80', 'backdrop-blur-sm', 'sticky', 'top-0', 'z-50')
  })
})