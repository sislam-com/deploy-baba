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
    global.fetch = vi.fn(() => Promise.resolve({ ok: true }))
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
})