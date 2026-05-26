import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'

interface Job {
  id: number
  slug: string
  company: string
  title: string
  location: string | null
  start_date: string
  end_date: string | null
  sort_order: number
}

interface LinkedInSyncInfo {
  mapped_job_id: number | null
  sync_status: string
}

const SYNC_COLORS: Record<string, string> = {
  synced: 'bg-green-900 text-green-300',
  diverged: 'bg-yellow-900 text-yellow-300',
  linkedin_only: 'bg-blue-900 text-blue-300',
  unreviewed: 'bg-purple-900 text-purple-300',
}

export default function Jobs() {
  const [jobs, setJobs] = useState<Job[]>([])
  const [syncMap, setSyncMap] = useState<Map<number, string>>(new Map())
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([
      fetch('/api/jobs').then(r => r.json()),
      fetch('/api/v1/admin/linkedin/positions').then(r => r.json()).catch(() => []),
    ])
      .then(([jobsData, positions]) => {
        setJobs(Array.isArray(jobsData) ? jobsData : [])
        const map = new Map<number, string>()
        if (Array.isArray(positions)) {
          for (const p of positions as LinkedInSyncInfo[]) {
            if (p.mapped_job_id != null) {
              map.set(p.mapped_job_id, p.sync_status)
            }
          }
        }
        setSyncMap(map)
      })
      .catch(() => setError('Failed to load jobs'))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-white">Jobs</h1>
        <Link
          to="/dashboard/jobs/new"
          className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          + New job
        </Link>
      </div>

      {loading && <p className="text-gray-500 text-sm">Loading…</p>}
      {error && <p className="text-red-400 text-sm">{error}</p>}

      <div className="space-y-2">
        {jobs.map(job => (
          <Link
            key={job.id}
            to={`/dashboard/jobs/${job.id}`}
            className="flex items-center justify-between bg-gray-800 border border-gray-700
                       hover:border-gray-500 rounded-xl px-5 py-4 transition"
          >
            <div>
              <div className="flex items-center gap-2">
                <p className="text-white font-medium">{job.title}</p>
                {syncMap.has(job.id) ? (
                  <span className={`text-[10px] font-semibold px-1.5 py-0.5 rounded ${SYNC_COLORS[syncMap.get(job.id)!] ?? 'bg-gray-700 text-gray-400'}`}>
                    LI: {syncMap.get(job.id)!.replace('_', ' ')}
                  </span>
                ) : (
                  <span className="text-[10px] font-semibold px-1.5 py-0.5 rounded bg-gray-700 text-gray-500">
                    No LI
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-400">{job.company} · {job.location ?? 'Remote'}</p>
            </div>
            <p className="text-xs text-gray-500 shrink-0 ml-4">
              {job.start_date} – {job.end_date ?? 'Present'}
            </p>
          </Link>
        ))}
      </div>
    </div>
  )
}
