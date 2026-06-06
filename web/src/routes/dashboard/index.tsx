import { useEffect, useState } from 'react'

interface Counts {
  jobs: number
  competencies: number
  about: number
  socialLinks: number
  linkedinUnreviewed: number
}

function StatTile({ label, count }: { label: string; count: number | null }) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl p-5">
      <p className="text-xs font-semibold uppercase tracking-wider text-gray-500">{label}</p>
      <p className="text-3xl font-bold text-white mt-1">
        {count === null ? <span className="text-gray-600">—</span> : count}
      </p>
    </div>
  )
}

export default function DashboardHome() {
  const [counts, setCounts] = useState<Counts | null>(null)

  useEffect(() => {
    Promise.all([
      fetch('/api/jobs').then(r => r.json()),
      fetch('/api/competencies').then(r => r.json()),
      fetch('/api/about/sections').then(r => r.json()),
      fetch('/api/v1/social-links').then(r => r.json()),
      fetch('/api/v1/admin/linkedin/positions').then(r => r.json()).catch(() => []),
      fetch('/api/v1/admin/linkedin/projects').then(r => r.json()).catch(() => []),
    ])
      .then(([jobs, comps, about, links, liPositions, liProjects]) => {
        const positions = Array.isArray(liPositions) ? liPositions : []
        const projects = Array.isArray(liProjects) ? liProjects : []
        const unreviewed = [...positions, ...projects].filter(
          (item: { sync_status: string }) => item.sync_status === 'unreviewed'
        ).length
        setCounts({
          jobs: Array.isArray(jobs) ? jobs.length : 0,
          competencies: Array.isArray(comps) ? comps.length : 0,
          about: Array.isArray(about) ? about.length : 0,
          socialLinks: Array.isArray(links) ? links.length : 0,
          linkedinUnreviewed: unreviewed,
        })
      })
      .catch(() => {})
  }, [])

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold text-white mb-6">Overview</h1>
      <div className="grid grid-cols-2 sm:grid-cols-5 gap-4">
        <StatTile label="Jobs" count={counts?.jobs ?? null} />
        <StatTile label="Competencies" count={counts?.competencies ?? null} />
        <StatTile label="About sections" count={counts?.about ?? null} />
        <StatTile label="Social links" count={counts?.socialLinks ?? null} />
        <StatTile label="LI Unreviewed" count={counts?.linkedinUnreviewed ?? null} />
      </div>
    </div>
  )
}
