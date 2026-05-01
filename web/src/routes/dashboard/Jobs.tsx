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

export default function Jobs() {
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch('/api/jobs')
      .then(r => r.json())
      .then((data: Job[]) => setJobs(data))
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
              <p className="text-white font-medium">{job.title}</p>
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
