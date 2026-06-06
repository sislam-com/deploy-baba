import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor, within } from '../../utils/test-render'
import DashboardHome from '../../../routes/dashboard/index'
import { http, HttpResponse } from 'msw'
import { server } from '../../mocks/server'

// Mock useAuth
vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('DashboardHome', () => {
  it('renders overview heading', () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })
    expect(screen.getByText('Overview')).toBeInTheDocument()
  })

  it('renders stat tiles', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      expect(screen.getByText('Jobs')).toBeInTheDocument()
      expect(screen.getByText('Competencies')).toBeInTheDocument()
      expect(screen.getByText('About sections')).toBeInTheDocument()
      expect(screen.getByText('Social links')).toBeInTheDocument()
      expect(screen.getByText('LI Unreviewed')).toBeInTheDocument()
    })
  })

  it('displays job count', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      const jobsTile = screen.getByText('Jobs').closest('div')!
      expect(within(jobsTile).getByText('1')).toBeInTheDocument()
    })
  })

  it('displays competency count', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      const compTile = screen.getByText('Competencies').closest('div')!
      expect(within(compTile).getByText('1')).toBeInTheDocument()
    })
  })

  it('displays about sections count', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      const aboutTile = screen.getByText('About sections').closest('div')!
      expect(within(aboutTile).getByText('1')).toBeInTheDocument()
    })
  })

  it('displays social links count', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      const socialTile = screen.getByText('Social links').closest('div')!
      expect(within(socialTile).getByText('2')).toBeInTheDocument()
    })
  })

  it('shows dashes while loading', () => {
    // Delay all responses so the initial null state is visible
    server.use(
      http.get('/api/jobs', async () => {
        await new Promise(r => setTimeout(r, 1000))
        return HttpResponse.json([])
      }),
      http.get('/api/competencies', async () => {
        await new Promise(r => setTimeout(r, 1000))
        return HttpResponse.json([])
      }),
      http.get('/api/about/sections', async () => {
        await new Promise(r => setTimeout(r, 1000))
        return HttpResponse.json([])
      }),
      http.get('/api/social-links', async () => {
        await new Promise(r => setTimeout(r, 1000))
        return HttpResponse.json([])
      }),
      http.get('/api/v1/admin/linkedin/positions', async () => {
        await new Promise(r => setTimeout(r, 1000))
        return HttpResponse.json([])
      }),
      http.get('/api/v1/admin/linkedin/projects', async () => {
        await new Promise(r => setTimeout(r, 1000))
        return HttpResponse.json([])
      })
    )

    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    expect(screen.getAllByText('—').length).toBe(5)
  })

  it('applies correct styling to stat tiles', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      const tile = screen.getByText('Jobs').closest('div')
      expect(tile).toHaveClass('bg-gray-800', 'border', 'border-gray-700', 'rounded-xl', 'p-5')
    })
  })

  it('applies correct styling to count values', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      const jobsTile = screen.getByText('Jobs').closest('div')!
      const count = within(jobsTile).getByText('1')
      expect(count).toHaveClass('text-3xl', 'font-bold', 'text-white')
    })
  })

  it('applies correct styling to labels', async () => {
    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    await waitFor(() => {
      const label = screen.getByText('Jobs')
      expect(label).toHaveClass('text-xs', 'font-semibold', 'uppercase', 'tracking-wider', 'text-gray-500')
    })
  })

  it('fetches data from multiple endpoints', async () => {
    server.use(
      http.get('/api/jobs', () => HttpResponse.json([{ id: 1 }])),
      http.get('/api/competencies', () => HttpResponse.json([{ id: 1 }])),
      http.get('/api/about/sections', () => HttpResponse.json([{ id: 1 }])),
      http.get('/api/v1/social-links', () => HttpResponse.json([{ id: 1 }]))
    )

    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    // All endpoints returned one item, so every count should be 1
    await waitFor(() => {
      const jobsTile = screen.getByText('Jobs').closest('div')!
      const compTile = screen.getByText('Competencies').closest('div')!
      const aboutTile = screen.getByText('About sections').closest('div')!
      const socialTile = screen.getByText('Social links').closest('div')!
      expect(within(jobsTile).getByText('1')).toBeInTheDocument()
      expect(within(compTile).getByText('1')).toBeInTheDocument()
      expect(within(aboutTile).getByText('1')).toBeInTheDocument()
      expect(within(socialTile).getByText('1')).toBeInTheDocument()
    })
  })

  it('handles API errors gracefully', async () => {
    server.use(
      http.get('/api/jobs', () => HttpResponse.error()),
      http.get('/api/competencies', () => HttpResponse.error()),
      http.get('/api/about/sections', () => HttpResponse.error()),
      http.get('/api/social-links', () => HttpResponse.error()),
      http.get('/api/v1/admin/linkedin/positions', () => HttpResponse.error()),
      http.get('/api/v1/admin/linkedin/projects', () => HttpResponse.error())
    )

    render(<DashboardHome />, { router: 'memory', route: '/dashboard' })

    // Should not crash; on error counts stays null so dashes persist
    await waitFor(() => {
      expect(screen.getAllByText('—').length).toBe(5)
    })
  })
})
