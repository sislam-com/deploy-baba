import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'

export interface AuthState {
  loading: boolean
  authenticated: boolean
  email: string | null
}

export function useAuth(redirect = true): AuthState {
  const navigate = useNavigate()
  const [state, setState] = useState<AuthState>({ loading: true, authenticated: false, email: null })

  useEffect(() => {
    fetch('/api/auth/me')
      .then(r => r.json())
      .then((data: { authenticated: boolean; email?: string }) => {
        if (!data.authenticated && redirect) {
          navigate('/auth/login', { replace: true })
        } else {
          setState({ loading: false, authenticated: data.authenticated, email: data.email ?? null })
        }
      })
      .catch(() => {
        if (redirect) navigate('/auth/login', { replace: true })
        else setState({ loading: false, authenticated: false, email: null })
      })
  }, [navigate, redirect])

  return state
}
