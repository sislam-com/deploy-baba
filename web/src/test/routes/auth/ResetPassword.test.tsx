import { describe, it, expect } from 'vitest'
import { render, screen, waitFor, userEvent } from '../../utils/test-render'
import ResetPassword from '../../../routes/auth/ResetPassword'

describe('ResetPassword', () => {
  it('renders heading', () => {
    render(<ResetPassword />, { router: 'memory', route: '/auth/reset-password' })
    expect(screen.getByRole('heading', { name: /reset password/i })).toBeInTheDocument()
  })

  it('renders all form fields', () => {
    render(<ResetPassword />, { router: 'memory', route: '/auth/reset-password' })
    expect(screen.getByLabelText(/username/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/reset code/i)).toBeInTheDocument()
    expect(screen.getByLabelText('New Password')).toBeInTheDocument()
    expect(screen.getByLabelText(/confirm password/i)).toBeInTheDocument()
  })

  it('shows password mismatch error', async () => {
    render(<ResetPassword />, { router: 'memory', route: '/auth/reset-password' })
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/username/i), 'admin')
    await user.type(screen.getByLabelText(/reset code/i), '123456')
    await user.type(screen.getByLabelText('New Password'), 'Password123!')
    await user.type(screen.getByLabelText(/confirm password/i), 'Different456!')
    await user.click(screen.getByRole('button', { name: /reset password/i }))

    await waitFor(() => {
      expect(screen.getByText(/passwords do not match/i)).toBeInTheDocument()
    })
  })

  it('shows short password error', async () => {
    render(<ResetPassword />, { router: 'memory', route: '/auth/reset-password' })
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/username/i), 'admin')
    await user.type(screen.getByLabelText(/reset code/i), '123456')
    await user.type(screen.getByLabelText('New Password'), 'short')
    await user.type(screen.getByLabelText(/confirm password/i), 'short')
    await user.click(screen.getByRole('button', { name: /reset password/i }))

    await waitFor(() => {
      expect(screen.getByText(/at least 12 characters/i)).toBeInTheDocument()
    })
  })

  it('submits successfully', async () => {
    render(<ResetPassword />, {
      router: 'memory',
      route: '/auth/reset-password',
      routes: [
        { path: '/auth/reset-password' },
        { path: '/auth/login' },
      ],
    })
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/username/i), 'admin')
    await user.type(screen.getByLabelText(/reset code/i), '123456')
    await user.type(screen.getByLabelText('New Password'), 'SecurePassword123!')
    await user.type(screen.getByLabelText(/confirm password/i), 'SecurePassword123!')
    await user.click(screen.getByRole('button', { name: /reset password/i }))

    await waitFor(() => {
      expect(screen.getByText(/password reset successfully/i)).toBeInTheDocument()
    })
  })
})
