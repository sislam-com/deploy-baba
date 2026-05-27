import { describe, it, expect } from 'vitest'
import { render, screen, waitFor, userEvent } from '../../utils/test-render'
import ForgotPassword from '../../../routes/auth/ForgotPassword'

describe('ForgotPassword', () => {
  it('renders heading', () => {
    render(<ForgotPassword />, { router: 'memory', route: '/auth/forgot-password' })
    expect(screen.getByText('Reset Password')).toBeInTheDocument()
  })

  it('renders username field', () => {
    render(<ForgotPassword />, { router: 'memory', route: '/auth/forgot-password' })
    expect(screen.getByLabelText(/username/i)).toBeInTheDocument()
  })

  it('renders submit button', () => {
    render(<ForgotPassword />, { router: 'memory', route: '/auth/forgot-password' })
    expect(screen.getByRole('button', { name: /send reset code/i })).toBeInTheDocument()
  })

  it('renders back to sign in link', () => {
    render(<ForgotPassword />, { router: 'memory', route: '/auth/forgot-password' })
    expect(screen.getByText(/back to sign in/i)).toBeInTheDocument()
  })

  it('shows success message on submit', async () => {
    render(<ForgotPassword />, { router: 'memory', route: '/auth/forgot-password' })
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/username/i), 'admin')
    await user.click(screen.getByRole('button', { name: /send reset code/i }))

    await waitFor(() => {
      expect(screen.getByText(/reset code sent/i)).toBeInTheDocument()
    })
  })
})
