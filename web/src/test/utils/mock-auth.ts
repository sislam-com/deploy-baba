// Mock auth state objects for use in tests
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