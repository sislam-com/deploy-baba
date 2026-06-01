import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '../utils/test-render'
import Privacy from '../../routes/Privacy'

describe('Privacy', () => {
  it('renders page heading', () => {
    render(<Privacy />)
    expect(screen.getByText('Privacy Policy')).toBeInTheDocument()
  })

  it('renders loading state initially', () => {
    render(<Privacy />)
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('fetches and renders document content', async () => {
    render(<Privacy />)

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText(/minimal data/)).toBeInTheDocument()
  })

  it('renders last updated date', async () => {
    render(<Privacy />)

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    expect(screen.getByText(/Last updated:/)).toBeInTheDocument()
  })
})
