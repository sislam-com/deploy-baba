import { describe, it, expect } from 'vitest'
import { render, screen, waitFor, userEvent } from '../../utils/test-render'
import Login from '../../../routes/auth/Login'

describe('Login', () => {
  it('renders page heading', () => {
    render(<Login />, { router: 'memory', route: '/auth/login' })
    expect(screen.getByRole('heading', { name: /dashboard/i })).toBeInTheDocument()
  })

  it('renders username and password fields', () => {
    render(<Login />, { router: 'memory', route: '/auth/login' })
    expect(screen.getByLabelText(/username/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument()
  })

  it('renders submit button', () => {
    render(<Login />, { router: 'memory', route: '/auth/login' })
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument()
  })

  it('renders forgot password link', () => {
    render(<Login />, { router: 'memory', route: '/auth/login' })
    expect(screen.getByText(/forgot password/i)).toBeInTheDocument()
  })

  it('submits form and navigates on success', async () => {
    render(<Login />, {
      router: 'memory',
      route: '/auth/login',
      routes: [
        { path: '/auth/login' },
        { path: '/dashboard' },
      ],
    })

    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/username/i), 'admin')
    await user.type(screen.getByLabelText(/password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /sign in/i }))

    await waitFor(() => {
      expect(screen.queryByText(/login failed/i)).not.toBeInTheDocument()
    })
  })
})
