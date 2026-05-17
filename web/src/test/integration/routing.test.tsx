import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../utils/test-render'
import userEvent from '@testing-library/user-event'
import App from '../../App'

// Mock useAuth
vi.mock('../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('Routing Integration', () => {
  it('navigates between marketing routes', async () => {
    render(<App />, { router: 'memory', route: '/' })
    const user = userEvent.setup()

    // Start at home
    expect(await screen.findByText('AI Systems Engineer')).toBeInTheDocument()

    // Navigate to about me
    const aboutLink = await screen.findByLabelText('About')
    await user.click(aboutLink)

    await waitFor(() => {
      expect(screen.getByText('Learn more about me and this project.')).toBeInTheDocument()
    })
  })

  it('navigates to contact from nav', async () => {
    render(<App />, { router: 'memory', route: '/' })
    const user = userEvent.setup()

    const contactLink = await screen.findByLabelText('Contact')
    await user.click(contactLink)

    await waitFor(() => {
      expect(screen.getByText('Send me a message. I typically respond within a day or two.')).toBeInTheDocument()
    })
  })

  it('navigates between dashboard routes', async () => {
    render(<App />, { router: 'memory', route: '/dashboard' })
    const user = userEvent.setup()

    // Start at dashboard overview
    expect(await screen.findByRole('heading', { name: 'Overview' })).toBeInTheDocument()

    // Navigate to jobs via sidebar link
    const jobsLink = screen.getByRole('link', { name: 'Jobs' })
    await user.click(jobsLink)

    await waitFor(() => {
      expect(screen.getByText('Senior Software Engineer')).toBeInTheDocument()
    })
  })

  it('navigates to dashboard detail pages', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/jobs' })
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.getByText('Senior Software Engineer')).toBeInTheDocument()
    })

    // Click on a job card
    const jobCard = screen.getByText('Senior Software Engineer').closest('a')
    if (jobCard) {
      await user.click(jobCard)

      await waitFor(() => {
        expect(screen.getByText('Edit job')).toBeInTheDocument()
      })
    }
  })

  it('navigates back from detail pages', async () => {
    render(<App />, { router: 'memory', route: '/dashboard/jobs/1' })
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.getByText('Edit job')).toBeInTheDocument()
    })

    const backLink = screen.getByText('← Jobs')
    await user.click(backLink)

    await waitFor(() => {
      expect(screen.getByText('Senior Software Engineer')).toBeInTheDocument()
    })
  })

  it('handles 404 for unknown routes', async () => {
    render(<App />, { router: 'memory', route: '/unknown-route' })
    expect(await screen.findByText('404')).toBeInTheDocument()
    expect(await screen.findByText('Page not found.')).toBeInTheDocument()
  })

  it('navigates back to home from 404', async () => {
    render(<App />, { router: 'memory', route: '/unknown-route' })
    const user = userEvent.setup()

    const backLink = await screen.findByText('← Back home')
    await user.click(backLink)

    await waitFor(() => {
      expect(screen.getByText('AI Systems Engineer')).toBeInTheDocument()
    })
  })

  it('handles external links correctly', async () => {
    render(<App />, { router: 'memory', route: '/' })

    const docsLink = await screen.findByLabelText('API Docs')
    expect(docsLink).toHaveAttribute('target', '_blank')
    expect(docsLink).toHaveAttribute('rel', 'noopener noreferrer')
  })

  it('preserves query parameters in navigation', async () => {
    render(<App />, { router: 'memory', route: '/about/me?test=value' })

    // Should still render the page with query params
    expect(await screen.findByText('Learn more about me and this project.')).toBeInTheDocument()
  })
})
