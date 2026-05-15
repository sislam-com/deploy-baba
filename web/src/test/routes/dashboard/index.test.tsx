import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import DashboardHome from '../../../routes/dashboard/index'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('DashboardHome', () => {
  it('renders overview heading', () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    expect(screen.getByText('Overview')).toBeInTheDocument()
  })

  it('renders stat tiles', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      expect(screen.getByText('Jobs')).toBeInTheDocument()
      expect(screen.getByText('Competencies')).toBeInTheDocument()
      expect(screen.getByText('About sections')).toBeInTheDocument()
      expect(screen.getByText('Social links')).toBeInTheDocument()
    })
  })

  it('displays job count', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      expect(screen.getByText('1')).toBeInTheDocument()
    })
  })

  it('displays competency count', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      expect(screen.getByText('1')).toBeInTheDocument()
    })
  })

  it('displays about sections count', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      expect(screen.getByText('1')).toBeInTheDocument()
    })
  })

  it('displays social links count', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      expect(screen.getByText('2')).toBeInTheDocument()
    })
  })

  it('shows dash when count is null', () => {
    // Mock empty responses
    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve([]),
      })
    )

    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    expect(screen.getByText('—')).toBeInTheDocument()
  })

  it('applies correct styling to stat tiles', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      const tile = screen.getByText('Jobs').closest('div')
      expect(tile).toHaveClass('bg-gray-800', 'border', 'border-gray-700', 'rounded-xl', 'p-5')
    })
  })

  it('applies correct styling to count values', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      const count = screen.getByText('1')
      expect(count).toHaveClass('text-3xl', 'font-bold', 'text-white')
    })
  })

  it('applies correct styling to labels', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      const label = screen.getByText('Jobs')
      expect(label).toHaveClass('text-xs', 'font-semibold', 'uppercase', 'tracking-wider', 'text-gray-500')
    })
  })

  it('fetches data from multiple endpoints', async () => {
    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith('/api/jobs')
      expect(global.fetch).toHaveBeenCalledWith('/api/competencies')
      expect(global.fetch).toHaveBeenCalledWith('/api/about/sections')
      expect(global.fetch).toHaveBeenCalledWith('/api/social-links')
    })
  })

  it('handles API errors gracefully', async () => {
    global.fetch = vi.fn(() =>
      Promise.reject(new Error('API Error'))
    )

    render(
      <DashboardHome />,
      { router: 'memory', route: '/dashboard' }
    )

    // Should not crash, just show dashes
    expect(screen.getByText('—')).toBeInTheDocument()
  })
})