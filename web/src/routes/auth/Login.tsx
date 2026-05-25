import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'

const AUTH_BASE = ''

interface SignInResponse {
  success: boolean
  tokens?: {
    id_token: string
    access_token: string
    refresh_token?: string
    expires_in: number
  }
  challenge?: {
    challenge_name: string
    session: string
    challenge_parameters: Record<string, string>
  }
}

export default function Login() {
  const navigate = useNavigate()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      const resp = await fetch(`${AUTH_BASE}/api/auth/signin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      })

      const data: SignInResponse = await resp.json()

      if (!resp.ok || !data.success) {
        setError(data.tokens ? 'Authentication failed' : (await resp.json()).error ?? 'Login failed')
        setLoading(false)
        return
      }

      if (data.challenge) {
        // Redirect to challenge page
        navigate('/auth/change-password', {
          state: {
            challengeName: data.challenge.challenge_name,
            session: data.challenge.session,
            challengeParameters: data.challenge.challenge_parameters,
            username,
          },
        })
        return
      }

      if (data.tokens?.id_token) {
        // Exchange id_token for HttpOnly cookie via UI Lambda
        const sessionResp = await fetch(`/auth/set-session?id_token=${encodeURIComponent(data.tokens.id_token)}`, {
          method: 'GET',
          redirect: 'manual',
        })

        if (sessionResp.ok || sessionResp.status === 302) {
          navigate('/dashboard', { replace: true })
        } else {
          setError('Failed to establish session')
        }
      }
    } catch (err) {
      setError('Network error. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      <Helmet>
        <title>Sign In — Sharful Islam</title>
      </Helmet>

      <div className="flex items-center justify-center min-h-screen bg-gray-900 px-4">
        <div className="w-full max-w-sm">
          <div className="text-center mb-8">
            <h1 className="text-2xl font-bold text-white">Dashboard</h1>
            <p className="text-gray-400 text-sm mt-1">Sign in to access the admin panel</p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="rounded-lg bg-red-900/30 border border-red-700/50 px-4 py-3 text-sm text-red-300">
                {error}
              </div>
            )}

            <div>
              <label htmlFor="username" className="block text-sm font-medium text-gray-300 mb-1">
                Username
              </label>
              <input
                id="username"
                type="text"
                value={username}
                onChange={e => setUsername(e.target.value)}
                className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-white
                           placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500
                           focus:outline-none transition"
                placeholder="Enter username"
                required
                autoComplete="username"
              />
            </div>

            <div>
              <label htmlFor="password" className="block text-sm font-medium text-gray-300 mb-1">
                Password
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-white
                           placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500
                           focus:outline-none transition"
                placeholder="Enter password"
                required
                autoComplete="current-password"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full rounded-lg bg-cyan-600 px-4 py-2 text-sm font-medium text-white
                         hover:bg-cyan-500 disabled:opacity-50 disabled:cursor-not-allowed
                         focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-500 transition"
            >
              {loading ? 'Signing in…' : 'Sign In'}
            </button>

            <div className="text-center">
              <Link
                to="/auth/forgot-password"
                className="text-sm text-gray-400 hover:text-cyan-400 transition"
              >
                Forgot password?
              </Link>
            </div>
          </form>
        </div>
      </div>
    </>
  )
}
