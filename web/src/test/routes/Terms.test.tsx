import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '../utils/test-render'
import Terms from '../../routes/Terms'

describe('Terms', () => {
  it('renders page heading', () => {
    render(<Terms />)
    expect(screen.getByText('Terms of Service')).toBeInTheDocument()
  })

  it('renders loading state initially', () => {
    render(<Terms />)
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders document content', async () => {
    render(<Terms />)

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText(/Welcome to deploy-baba/)).toBeInTheDocument()
  })

  it('renders last updated date', async () => {
    render(<Terms />)

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText(/Last updated:/)).toBeInTheDocument()
  })
})
