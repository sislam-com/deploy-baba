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

interface LinkedInSyncInfo {
  mapped_challenge_id: number | null
  sync_status: string
}

const SYNC_COLORS: Record<string, string> = {
  synced: 'bg-green-900 text-green-300',
  diverged: 'bg-yellow-900 text-yellow-300',
  linkedin_only: 'bg-blue-900 text-blue-300',
  unreviewed: 'bg-purple-900 text-purple-300',
}

export default function Challenges() {
  const [challenges, setChallenges] = useState<Challenge[]>([])
  const [syncMap, setSyncMap] = useState<Map<number, string>>(new Map())
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([
      fetch('/api/challenges').then(r => r.json()),
      fetch('/api/v1/admin/linkedin/projects').then(r => r.json()).catch(() => []),
    ])
      .then(([chData, projects]) => {
        setChallenges(Array.isArray(chData) ? chData : [])
        const map = new Map<number, string>()
        if (Array.isArray(projects)) {
          for (const p of projects as LinkedInSyncInfo[]) {
            if (p.mapped_challenge_id != null) {
              map.set(p.mapped_challenge_id, p.sync_status)
            }
          }
        }
        setSyncMap(map)
      })
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
                {syncMap.has(ch.id) ? (
                  <span className={`shrink-0 text-[10px] font-semibold px-1.5 py-0.5 rounded ${SYNC_COLORS[syncMap.get(ch.id)!] ?? 'bg-gray-700 text-gray-400'}`}>
                    LI: {syncMap.get(ch.id)!.replace('_', ' ')}
                  </span>
                ) : (
                  <span className="shrink-0 text-[10px] font-semibold px-1.5 py-0.5 rounded bg-gray-700 text-gray-500">
                    No LI
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
