import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'

const AUTH_BASE = ''

export default function ForgotPassword() {
  const [username, setUsername] = useState('')
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setSuccess('')
    setLoading(true)

    try {
      const resp = await fetch(`${AUTH_BASE}/api/auth/forgot-password`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username }),
      })

      const data = await resp.json()

      if (!resp.ok || !data.success) {
        setError(data.error ?? 'Failed to send reset code')
      } else {
        setSuccess('Reset code sent. Check your email.')
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
        <title>Forgot Password — Sharful Islam</title>
      </Helmet>

      <div className="flex items-center justify-center min-h-screen bg-gray-900 px-4">
        <div className="w-full max-w-sm">
          <div className="text-center mb-8">
            <h1 className="text-2xl font-bold text-white">Reset Password</h1>
            <p className="text-gray-400 text-sm mt-1">Enter your username to receive a reset code</p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="rounded-lg bg-red-900/30 border border-red-700/50 px-4 py-3 text-sm text-red-300">
                {error}
              </div>
            )}
            {success && (
              <div className="rounded-lg bg-green-900/30 border border-green-700/50 px-4 py-3 text-sm text-green-300">
                {success}
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

            <button
              type="submit"
              disabled={loading}
              className="w-full rounded-lg bg-cyan-600 px-4 py-2 text-sm font-medium text-white
                         hover:bg-cyan-500 disabled:opacity-50 disabled:cursor-not-allowed
                         focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-500 transition"
            >
              {loading ? 'Sending…' : 'Send Reset Code'}
            </button>

            <div className="text-center">
              <Link
                to="/auth/login"
                className="text-sm text-gray-400 hover:text-cyan-400 transition"
              >
                Back to sign in
              </Link>
            </div>
          </form>
        </div>
      </div>
    </>
  )
}
