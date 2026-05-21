import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import userEvent from '@testing-library/user-event'
import SocialLinkDetail from '../../../routes/dashboard/SocialLinkDetail'
import DashboardLayout from '../../../routes/dashboard/Layout'

vi.mock('../../../hooks/useAuth', () => ({
  useAuth: () => ({ loading: false, authenticated: true, email: 'test@example.com' }),
}))

describe('SocialLinkDetail', () => {
  it('renders new social link form', () => {
    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/new', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    expect(screen.getByText('New social link')).toBeInTheDocument()
  })

  it('renders form fields', () => {
    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/new', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    expect(screen.getByLabelText('url')).toBeInTheDocument()
    expect(screen.getByLabelText('label')).toBeInTheDocument()
  })

  it('renders save button', () => {
    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/new', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
  })

  it('submits form and navigates back', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response(null, { status: 200 })))
    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/new', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('platform'), 'example')
    await user.type(screen.getByLabelText('url'), 'https://example.com')
    await user.type(screen.getByLabelText('label'), 'Example')

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => {
      expect(screen.queryByText('New social link')).not.toBeInTheDocument()
    })
  })

  it('shows error on save failure', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response('Save failed', { status: 400 })))
    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/new', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('platform'), 'example')
    await user.type(screen.getByLabelText('url'), 'https://example.com')
    await user.type(screen.getByLabelText('label'), 'Example')

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => {
      expect(screen.getByText('Save failed')).toBeInTheDocument()
    })
  })

  it('shows loading state for existing link', async () => {
    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/1', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    expect(screen.getByText('Loading…')).toBeInTheDocument()
  })

  it('deletes existing link and navigates back', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response(null, { status: 200 })))
    global.confirm = vi.fn(() => true)

    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/1', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    const deleteButton = screen.getByRole('button', { name: 'Delete' })
    await user.click(deleteButton)

    await waitFor(() => {
      expect(global.confirm).toHaveBeenCalledWith('Delete this social link?')
    })
  })

  it('shows delete error when fetch fails', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response('Delete failed', { status: 500 })))
    global.confirm = vi.fn(() => true)

    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/1', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.queryByText('Loading…')).not.toBeInTheDocument()
    })

    const deleteButton = screen.getByRole('button', { name: 'Delete' })
    await user.click(deleteButton)

    await waitFor(() => {
      expect(screen.getByText('Delete failed')).toBeInTheDocument()
    })
  })

  it('fills all fields and submits new social link', async () => {
    global.fetch = vi.fn(() => Promise.resolve(new Response(null, { status: 200 })))

    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/new', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('platform'), 'example')
    await user.type(screen.getByLabelText('url'), 'https://example.com')
    await user.type(screen.getByLabelText('label'), 'Example')
    await user.type(screen.getByLabelText('icon'), 'globe')
    await user.type(screen.getByLabelText('Sort order'), '3')

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => {
      expect(screen.queryByText('New social link')).not.toBeInTheDocument()
    })
  })

  it('toggles visible checkbox', async () => {
    render(
      <DashboardLayout>
        <SocialLinkDetail />
      </DashboardLayout>,
      { router: 'memory', route: '/dashboard/social-links/new', routes: [{ path: '/dashboard/social-links/:id' }] }
    )
    const user = userEvent.setup()

    const checkbox = screen.getByLabelText('Visible in nav')
    expect(checkbox).toBeChecked()

    await user.click(checkbox)
    expect(checkbox).not.toBeChecked()
  })
})