import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '../utils/test-render'
import AboutRepo from '../../routes/AboutRepo'

describe('AboutRepo', () => {
  it('renders page heading', () => {
    render(<AboutRepo />)
    expect(screen.getByText('About')).toBeInTheDocument()
  })

  it('renders page description', () => {
    render(<AboutRepo />)
    expect(screen.getByText('Learn more about me and this project.')).toBeInTheDocument()
  })

  it('renders inactive tab for About Me', () => {
    render(<AboutRepo />)
    const inactiveTab = screen.getByText('About Me')
    expect(inactiveTab).toBeInTheDocument()
    expect(inactiveTab).toHaveClass('text-gray-400')
  })

  it('renders active tab for About This Repo', () => {
    render(<AboutRepo />)
    const activeTab = screen.getByText('About This Repo')
    expect(activeTab).toBeInTheDocument()
    expect(activeTab).toHaveClass('bg-cyan-600', 'text-white')
  })

  it('renders link to About Me', () => {
    render(<AboutRepo />)
    const link = screen.getByText('About Me')
    expect(link).toHaveAttribute('href', '/about/me')
  })

  it('renders loading state initially', () => {
    render(<AboutRepo />)
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders about sections', async () => {
    render(<AboutRepo />)

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    // Should render sections from MSW mock data
    expect(screen.getByText('Background')).toBeInTheDocument()
  })

  it('renders section heading when present', async () => {
    render(<AboutRepo />)

    await waitFor(() => {
      expect(screen.getByText('Background')).toBeInTheDocument()
    })
  })

  it('renders section body content', async () => {
    render(<AboutRepo />)

    await waitFor(() => {
      expect(screen.getByText(/I am a software engineer/)).toBeInTheDocument()
    })
  })

  it('sets correct page title', () => {
    render(<AboutRepo />)
    expect(document.title).toBe('About This Repo — Sharful Islam')
  })

  it('applies correct styling to sections', async () => {
    render(<AboutRepo />)

    await waitFor(() => {
      expect(screen.getByText('Background')).toBeInTheDocument()
    })

    const section = screen.getByText('Background').closest('div')
    expect(section).toHaveClass('bg-gray-800/50', 'rounded-lg', 'p-6', 'border', 'border-gray-700')
  })
})