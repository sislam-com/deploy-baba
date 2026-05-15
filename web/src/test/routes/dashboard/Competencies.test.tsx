import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import Competencies from '../../../routes/dashboard/Competencies'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('Competencies', () => {
  it('renders competencies heading', () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })
    expect(screen.getByText('Competencies')).toBeInTheDocument()
  })

  it('renders new competency button', () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })
    const newButton = screen.getByText('+ New competency')
    expect(newButton).toBeInTheDocument()
    expect(newButton).toHaveAttribute('href', '/dashboard/competencies/new')
  })

  it('renders loading state initially', () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders competencies list', async () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText('Rust Systems Programming')).toBeInTheDocument()
  })

  it('renders competency name', async () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })

    await waitFor(() => {
      expect(screen.getByText('Rust Systems Programming')).toBeInTheDocument()
    })
  })

  it('renders competency description', async () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })

    await waitFor(() => {
      expect(screen.getByText('Expert in Rust systems programming')).toBeInTheDocument()
    })
  })

  it('renders competency icon', async () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })

    await waitFor(() => {
      expect(screen.getByText('🦀')).toBeInTheDocument()
    })
  })

  it('renders competency cards as links', async () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })

    await waitFor(() => {
      const card = screen.getByText('Rust Systems Programming').closest('a')
      expect(card).toHaveAttribute('href', '/dashboard/competencies/1')
    })
  })

  it('applies correct styling to competency cards', async () => {
    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })

    await waitFor(() => {
      const card = screen.getByText('Rust Systems Programming').closest('a')
      expect(card).toHaveClass('bg-gray-800', 'border', 'border-gray-700', 'hover:border-gray-500', 'rounded-xl')
    })
  })

  it('shows error state on fetch failure', async () => {
    global.fetch = vi.fn(() =>
      Promise.reject(new Error('API Error'))
    )

    render(<Competencies />, { router: 'memory', route: '/dashboard/competencies' })

    await waitFor(() => {
      expect(screen.getByText('Failed to load competencies')).toBeInTheDocument()
    })
  })
})