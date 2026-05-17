import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '../utils/test-render'
import App from '../../App'

// Mock useAuth for all routes
vi.mock('../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('App Integration', () => {
  it('renders home route', async () => {
    render(<App />, { router: 'memory', route: '/' })
    expect(await screen.findByText('AI Systems Engineer')).toBeInTheDocument()
  })

  it('renders about me route', async () => {
    render(<App />, { router: 'memory', route: '/about/me' })
    expect(await screen.findByText('Learn more about me and this project.')).toBeInTheDocument()
  })

  it('renders about repo route', async () => {
    render(<App />, { router: 'memory', route: '/about/repo' })
    expect(await screen.findByText('About This Repo')).toBeInTheDocument()
  })

  it('renders contact route', async () => {
    render(<App />, { router: 'memory', route: '/contact' })
    expect(await screen.findByText('Send me a message. I typically respond within a day or two.')).toBeInTheDocument()
  })

  it('renders not found route', async () => {
    render(<App />, { router: 'memory', route: '/non-existent' })
    expect(await screen.findByText('404')).toBeInTheDocument()
  })

  it('renders dashboard overview route', async () => {
    render(<App />, { router: 'memory', route: '/dashboard' })
    expect(await screen.findByRole('heading', { name: 'Overview' })).toBeInTheDocument()
  })

  it('renders dashboard jobs route', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/jobs' })
    expect(await screen.findByText('Senior Software Engineer')).toBeInTheDocument()
  })

  it('renders dashboard competencies route', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/competencies' })
    expect(await screen.findByText('Rust Systems Programming')).toBeInTheDocument()
  })

  it('renders dashboard about route', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/about' })
    expect(await screen.findByRole('heading', { name: 'About sections' })).toBeInTheDocument()
  })

  it('renders dashboard social links route', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/social-links' })
    expect(await screen.findByRole('heading', { name: 'Social links' })).toBeInTheDocument()
  })

  it('renders dashboard challenges route', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/challenges' })
    expect(await screen.findByText('Portfolio RAG System')).toBeInTheDocument()
  })

  it('applies Layout to marketing routes', async () => {
    render(<App />, { router: 'memory', route: '/' })
    expect(await screen.findByLabelText('About')).toBeInTheDocument()
    expect(await screen.findByLabelText('Contact')).toBeInTheDocument()
  })

  it('applies DashboardLayout to dashboard routes', async () => {
    render(<App />, { router: 'memory', route: '/dashboard' })
    expect(await screen.findByText('Dashboard')).toBeInTheDocument()
    expect(await screen.findByRole('heading', { name: 'Overview' })).toBeInTheDocument()
  })

  it('renders nested dashboard routes', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/jobs/1' })
    expect(await screen.findByText('Dashboard')).toBeInTheDocument()
  })
})
