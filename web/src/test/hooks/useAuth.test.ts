import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { useAuth } from '../../hooks/useAuth'

// Mock fetch globally
global.fetch = vi.fn()

describe('useAuth', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Mock window.location for redirect tests
    delete (window as any).location
    window.location = { href: '' } as any
  })

  it('returns loading state initially', () => {
    ;(global.fetch as any).mockImplementationOnce(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ authenticated: true, email: 'test@example.com' }),
      })
    )

    const { result } = renderHook(() => useAuth(false))
    expect(result.current.loading).toBe(true)
  })

  it('returns authenticated state on successful auth check', async () => {
    ;(global.fetch as any).mockImplementationOnce(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ authenticated: true, email: 'test@example.com' }),
      })
    )

    const { result } = renderHook(() => useAuth(false))

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.authenticated).toBe(true)
    expect(result.current.email).toBe('test@example.com')
  })

  it('returns unauthenticated state when API returns unauthenticated', async () => {
    ;(global.fetch as any).mockImplementationOnce(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ authenticated: false }),
      })
    )

    const { result } = renderHook(() => useAuth(false))

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.authenticated).toBe(false)
    expect(result.current.email).toBeNull()
  })

  it('handles fetch errors gracefully', async () => {
    ;(global.fetch as any).mockImplementationOnce(() => Promise.reject(new Error('Network error')))

    const { result } = renderHook(() => useAuth(false))

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.authenticated).toBe(false)
    expect(result.current.email).toBeNull()
  })

  it('calls /api/auth/me endpoint', async () => {
    ;(global.fetch as any).mockImplementationOnce(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ authenticated: true, email: 'test@example.com' }),
      })
    )

    renderHook(() => useAuth(false))

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith('/api/auth/me')
    })
  })
})