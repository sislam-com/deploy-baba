import { useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'

const AUTH_BASE = ''

interface LocationState {
  challengeName: string
  session: string
  challengeParameters: Record<string, string>
  username: string
}

export default function ChangePassword() {
  const navigate = useNavigate()
  const location = useLocation()
  const state = location.state as LocationState | null

  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  // If navigated here without challenge state, redirect to login
  if (!state?.challengeName) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900">
        <div className="text-center">
          <p className="text-gray-400">No active challenge. Redirecting…</p>
          {setTimeout(() => navigate('/auth/login', { replace: true }), 100) && null}
        </div>
      </div>
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (newPassword !== confirmPassword) {
      setError('Passwords do not match')
      return
    }
    if (newPassword.length < 12) {
      setError('Password must be at least 12 characters')
      return
    }

    setLoading(true)

    try {
      const challengeResponses: Record<string, string> = {
        USERNAME: state.username,
        NEW_PASSWORD: newPassword,
      }

      const resp = await fetch(`${AUTH_BASE}/api/auth/respond-to-challenge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          challenge_name: state.challengeName,
          session: state.session,
          ...challengeResponses,
        }),
      })

      const data = await resp.json()

      if (!resp.ok || !data.success) {
        setError(data.error ?? 'Failed to change password')
        setLoading(false)
        return
      }

      if (data.tokens?.id_token) {
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
    } catch {
      setError('Network error. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      <Helmet>
        <title>Change Password — Sharful Islam</title>
      </Helmet>

      <div className="flex items-center justify-center min-h-screen bg-gray-900 px-4">
        <div className="w-full max-w-sm">
          <div className="text-center mb-8">
            <h1 className="text-2xl font-bold text-white">Change Password</h1>
            <p className="text-gray-400 text-sm mt-1">
              Your password must be changed before continuing
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="rounded-lg bg-red-900/30 border border-red-700/50 px-4 py-3 text-sm text-red-300">
                {error}
              </div>
            )}

            <div>
              <label htmlFor="newPassword" className="block text-sm font-medium text-gray-300 mb-1">
                New Password
              </label>
              <input
                id="newPassword"
                type="password"
                value={newPassword}
                onChange={e => setNewPassword(e.target.value)}
                className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-white
                           placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500
                           focus:outline-none transition"
                placeholder="Min 12 characters"
                required
                autoComplete="new-password"
              />
            </div>

            <div>
              <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-300 mb-1">
                Confirm Password
              </label>
              <input
                id="confirmPassword"
                type="password"
                value={confirmPassword}
                onChange={e => setConfirmPassword(e.target.value)}
                className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-white
                           placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500
                           focus:outline-none transition"
                placeholder="Repeat password"
                required
                autoComplete="new-password"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full rounded-lg bg-cyan-600 px-4 py-2 text-sm font-medium text-white
                         hover:bg-cyan-500 disabled:opacity-50 disabled:cursor-not-allowed
                         focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-500 transition"
            >
              {loading ? 'Updating…' : 'Update Password'}
            </button>
          </form>
        </div>
      </div>
    </>
  )
}
