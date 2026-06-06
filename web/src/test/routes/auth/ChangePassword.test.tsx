import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '../../utils/test-render'
import { userEvent } from '../../utils/test-render'
import ChangePassword from '../../../routes/auth/ChangePassword'
import { server } from '../../mocks/server'
import { http, HttpResponse } from 'msw'

const mockNavigate = vi.fn()
const mockLocationState = { state: null as Record<string, unknown> | null }

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useLocation: () => ({ state: mockLocationState.state, pathname: '/auth/change-password', search: '', hash: '' }),
  }
})

const mockChallengeState = {
  challengeName: 'NEW_PASSWORD_REQUIRED',
  session: 'mock-session-token',
  challengeParameters: {},
  username: 'test@example.com',
}

describe('ChangePassword', () => {
  beforeEach(() => {
    mockNavigate.mockClear()
    vi.useFakeTimers({ shouldAdvanceTime: true })
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows redirect message without challenge state', () => {
    render(<ChangePassword />, { router: 'memory', route: '/auth/change-password' })
    expect(screen.getByText(/no active challenge/i)).toBeInTheDocument()
  })

  it('redirects to login after delay when no challenge state', async () => {
    render(<ChangePassword />, { router: 'memory', route: '/auth/change-password' })
    
    expect(screen.getByText(/no active challenge/i)).toBeInTheDocument()
    
    // Advance timers to trigger redirect
    await vi.advanceTimersByTimeAsync(150)
    
    expect(mockNavigate).toHaveBeenCalledWith('/auth/login', { replace: true })
  })

  it('renders form with challenge state', () => {
    mockLocationState.state = mockChallengeState
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
    })
    
    expect(screen.getByRole('heading', { name: /change password/i })).toBeInTheDocument()
    expect(screen.getByLabelText(/new password/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/confirm password/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /update password/i })).toBeInTheDocument()
  })

  it('shows error when passwords do not match', async () => {
    mockLocationState.state = mockChallengeState
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
    })
    
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/new password/i), 'password123456')
    await user.type(screen.getByLabelText(/confirm password/i), 'differentpassword')
    await user.click(screen.getByRole('button', { name: /update password/i }))
    
    expect(screen.getByText(/passwords do not match/i)).toBeInTheDocument()
  })

  it('shows error when password is too short', async () => {
    mockLocationState.state = mockChallengeState
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
    })
    
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/new password/i), 'short')
    await user.type(screen.getByLabelText(/confirm password/i), 'short')
    await user.click(screen.getByRole('button', { name: /update password/i }))
    
    expect(screen.getByText(/password must be at least 12 characters/i)).toBeInTheDocument()
  })

  it('successfully changes password and navigates to dashboard', async () => {
    mockLocationState.state = mockChallengeState
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
      routes: [
        { path: '/auth/change-password' },
        { path: '/dashboard' },
      ],
    })
    
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/new password/i), 'newsecurepassword123')
    await user.type(screen.getByLabelText(/confirm password/i), 'newsecurepassword123')
    await user.click(screen.getByRole('button', { name: /update password/i }))
    
    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    })
  })

  it('shows API error message on failed response', async () => {
    mockLocationState.state = mockChallengeState
    server.use(
      http.post('/api/auth/respond-to-challenge', () => {
        return HttpResponse.json({ success: false, error: 'Invalid password policy' })
      })
    )
    
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
    })
    
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/new password/i), 'newsecurepassword123')
    await user.type(screen.getByLabelText(/confirm password/i), 'newsecurepassword123')
    await user.click(screen.getByRole('button', { name: /update password/i }))
    
    await waitFor(() => {
      expect(screen.getByText(/invalid password policy/i)).toBeInTheDocument()
    })
  })

  it('shows error when session establishment fails', async () => {
    mockLocationState.state = mockChallengeState
    server.use(
      http.get('/auth/set-session', () => {
        return new HttpResponse(null, { status: 500 })
      })
    )
    
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
    })
    
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/new password/i), 'newsecurepassword123')
    await user.type(screen.getByLabelText(/confirm password/i), 'newsecurepassword123')
    await user.click(screen.getByRole('button', { name: /update password/i }))
    
    await waitFor(() => {
      expect(screen.getByText(/failed to establish session/i)).toBeInTheDocument()
    })
  })

  it('shows network error on fetch failure', async () => {
    mockLocationState.state = mockChallengeState
    server.use(
      http.post('/api/auth/respond-to-challenge', () => {
        return HttpResponse.error()
      })
    )
    
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
    })
    
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/new password/i), 'newsecurepassword123')
    await user.type(screen.getByLabelText(/confirm password/i), 'newsecurepassword123')
    await user.click(screen.getByRole('button', { name: /update password/i }))
    
    await waitFor(() => {
      expect(screen.getByText(/network error/i)).toBeInTheDocument()
    })
  })

  it('disables submit button while loading', async () => {
    mockLocationState.state = mockChallengeState
    server.use(
      http.post('/api/auth/respond-to-challenge', async () => {
        await new Promise(resolve => setTimeout(resolve, 100))
        return HttpResponse.json({ 
          success: true,
          tokens: { id_token: 'mock-id-token' }
        })
      })
    )
    
    render(<ChangePassword />, { 
      router: 'memory', 
      route: '/auth/change-password',
    })
    
    const user = userEvent.setup()
    await user.type(screen.getByLabelText(/new password/i), 'newsecurepassword123')
    await user.type(screen.getByLabelText(/confirm password/i), 'newsecurepassword123')
    
    const submitButton = screen.getByRole('button', { name: /update password/i })
    await user.click(submitButton)
    
    expect(submitButton).toBeDisabled()
    expect(screen.getByText(/updating/i)).toBeInTheDocument()
  })
})
