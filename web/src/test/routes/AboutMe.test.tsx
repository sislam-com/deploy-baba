import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '../utils/test-render'
import AboutMe from '../../routes/AboutMe'

describe('AboutMe', () => {
  it('renders page heading', () => {
    render(<AboutMe />)
    expect(screen.getByText('About')).toBeInTheDocument()
  })

  it('renders page description', () => {
    render(<AboutMe />)
    expect(screen.getByText('Learn more about me and this project.')).toBeInTheDocument()
  })

  it('renders active tab for About Me', () => {
    render(<AboutMe />)
    const activeTab = screen.getByText('About Me')
    expect(activeTab).toBeInTheDocument()
    expect(activeTab).toHaveClass('bg-cyan-600', 'text-white')
  })

  it('renders inactive tab for About This Repo', () => {
    render(<AboutMe />)
    const inactiveTab = screen.getByText('About This Repo')
    expect(inactiveTab).toBeInTheDocument()
    expect(inactiveTab).toHaveClass('text-gray-400')
  })

  it('renders link to About This Repo', () => {
    render(<AboutMe />)
    const link = screen.getByText('About This Repo')
    expect(link).toHaveAttribute('href', '/about/repo')
  })

  it('renders loading state initially', () => {
    render(<AboutMe />)
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders about sections', async () => {
    render(<AboutMe />)

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    // Should render sections from MSW mock data
    expect(screen.getByText('Background')).toBeInTheDocument()
  })

  it('renders section heading when present', async () => {
    render(<AboutMe />)

    await waitFor(() => {
      expect(screen.getByText('Background')).toBeInTheDocument()
    })
  })

  it('renders section body content', async () => {
    render(<AboutMe />)

    await waitFor(() => {
      expect(screen.getByText(/I am a software engineer/)).toBeInTheDocument()
    })
  })

  it('applies correct styling to sections', async () => {
    render(<AboutMe />)

    await waitFor(() => {
      expect(screen.getByText('Background')).toBeInTheDocument()
    })

    const section = screen.getByText('Background').closest('div')
    expect(section).toHaveClass('bg-gray-800/50', 'rounded-lg', 'p-6', 'border', 'border-gray-700')
  })
})