import { describe, it, expect, vi } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../mocks/server'
import { useAuth } from '../../hooks/useAuth'

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <MemoryRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>{children}</MemoryRouter>
)

describe('useAuth', () => {
  it('returns loading state initially', () => {
    server.use(
      http.get('/api/auth/me', () => {
        return HttpResponse.json({ authenticated: true, email: 'test@example.com' })
      })
    )

    const { result } = renderHook(() => useAuth(false), { wrapper })
    expect(result.current.loading).toBe(true)
  })

  it('returns authenticated state on successful auth check', async () => {
    server.use(
      http.get('/api/auth/me', () => {
        return HttpResponse.json({ authenticated: true, email: 'test@example.com' })
      })
    )

    const { result } = renderHook(() => useAuth(false), { wrapper })

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.authenticated).toBe(true)
    expect(result.current.email).toBe('test@example.com')
  })

  it('returns unauthenticated state when API returns unauthenticated', async () => {
    server.use(
      http.get('/api/auth/me', () => {
        return HttpResponse.json({ authenticated: false })
      })
    )

    const { result } = renderHook(() => useAuth(false), { wrapper })

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.authenticated).toBe(false)
    expect(result.current.email).toBeNull()
  })

  it('handles fetch errors gracefully', async () => {
    server.use(http.get('/api/auth/me', () => HttpResponse.error()))

    const { result } = renderHook(() => useAuth(false), { wrapper })

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.authenticated).toBe(false)
    expect(result.current.email).toBeNull()
  })

  it('calls /api/auth/me endpoint', async () => {
    const requestSpy = vi.fn()
    server.use(
      http.get('/api/auth/me', ({ request }) => {
        requestSpy(request.url)
        return HttpResponse.json({ authenticated: true, email: 'test@example.com' })
      })
    )

    renderHook(() => useAuth(false), { wrapper })

    await waitFor(() => {
      expect(requestSpy).toHaveBeenCalledTimes(1)
    })
  })
})
