import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import DashboardLayout from '../../../routes/dashboard/Layout'

let mockAuth = { loading: false, authenticated: true, email: 'test@example.com' }

vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => mockAuth,
}))

describe('DashboardLayout', () => {
  it('renders sidebar with navigation items', () => {
    render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    expect(screen.getByText('Overview')).toBeInTheDocument()
    expect(screen.getByText('Jobs')).toBeInTheDocument()
    expect(screen.getByText('Competencies')).toBeInTheDocument()
    expect(screen.getByText('About Sections')).toBeInTheDocument()
    expect(screen.getByText('Social Links')).toBeInTheDocument()
    expect(screen.getByText('Challenges')).toBeInTheDocument()
  })

  it('renders user email in sidebar', () => {
    render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    expect(screen.getByText('test@example.com')).toBeInTheDocument()
  })

  it('renders dashboard heading', () => {
    render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    expect(screen.getByText('Dashboard')).toBeInTheDocument()
  })

  it('renders sign out link', () => {
    render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    const signOutLink = screen.getByText('Sign out')
    expect(signOutLink).toBeInTheDocument()
    expect(signOutLink).toHaveAttribute('href', '/auth/logout')
  })

  it('renders main content area', () => {
    render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    expect(screen.getByText('Test Content')).toBeInTheDocument()
  })

  it('shows loading state when auth is loading', () => {
    mockAuth = { loading: true, authenticated: false, email: null }

    render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    expect(screen.queryByText('Dashboard')).not.toBeInTheDocument()
    // Should show loading spinner
    expect(document.querySelector('.animate-spin')).toBeInTheDocument()

    mockAuth = { loading: false, authenticated: true, email: 'test@example.com' }
  })

  it('highlights active navigation item', () => {
    render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard' }
    )

    const overviewLink = screen.getByText('Overview')
    expect(overviewLink).toHaveClass('bg-gray-700', 'text-white', 'font-medium')
  })

  it('applies correct styling to sidebar', () => {
    const { container } = render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    const sidebar = container.querySelector('aside')
    expect(sidebar).toHaveClass('w-56', 'bg-gray-800', 'border-r', 'border-gray-700')
  })

  it('applies correct styling to main content area', () => {
    const { container } = render(
      <DashboardLayout>
        <div>Test Content</div>
      </DashboardLayout>
    )

    const main = container.querySelector('main')
    expect(main).toHaveClass('flex-1', 'overflow-auto')
  })

})