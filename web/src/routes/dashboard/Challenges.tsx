import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'

interface Challenge {
  id: number
  slug: string
  title: string
  job_id: number | null
  short_description: string | null
  category: string | null
  featured: boolean
  sort_order: number
}

export default function Challenges() {
  const [challenges, setChallenges] = useState<Challenge[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch('/api/challenges')
      .then(r => r.json())
      .then((data: Challenge[]) => setChallenges(data))
      .catch(() => setError('Failed to load challenges'))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-white">Challenges</h1>
        <Link
          to="/dashboard/challenges/new"
          className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          + New challenge
        </Link>
      </div>

      {loading && <p className="text-gray-500 text-sm">Loading…</p>}
      {error && <p className="text-red-400 text-sm">{error}</p>}

      <div className="space-y-2">
        {challenges.map(ch => (
          <Link
            key={ch.id}
            to={`/dashboard/challenges/${ch.id}`}
            className="flex items-center justify-between bg-gray-800 border border-gray-700
                       hover:border-gray-500 rounded-xl px-5 py-4 transition"
          >
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <p className="text-white font-medium truncate">{ch.title}</p>
                {ch.featured && (
                  <span className="shrink-0 text-[10px] font-semibold px-1.5 py-0.5 rounded bg-cyan-900 text-cyan-300">
                    Featured
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-400 truncate">
                {ch.short_description ?? ch.slug}
              </p>
            </div>
            {ch.category && (
              <span className="text-xs text-gray-500 shrink-0 ml-4">{ch.category}</span>
            )}
          </Link>
        ))}
      </div>
    </div>
  )
}
