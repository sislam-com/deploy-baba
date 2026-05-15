import { renderHook, act } from '@testing-library/react'
import { vi } from 'vitest'

// Mock the useAuth hook
export const mockAuthState = {
  loading: false,
  authenticated: true,
  email: 'test@example.com',
}

export const mockUnauthState = {
  loading: false,
  authenticated: false,
  email: null,
}

export const mockLoadingState = {
  loading: true,
  authenticated: false,
  email: null,
}

// Helper to mock useAuth hook in tests
export function mockUseAuth(state = mockAuthState) {
  vi.mock('../../hooks/useAuth', () => ({
    useAuth: () => state,
  }))
}

// Helper to test auth redirects
export async function testAuthRedirect(
  component: React.ReactElement,
  expectedRedirect: string
) {
  const { navigate } = renderHook(() => {
    // This would be used with actual useAuth hook testing
    // For now, it's a placeholder for the pattern
  })

  await act(async () => {
    // Test redirect logic
  })

  // Assert navigation to expectedRedirect
}