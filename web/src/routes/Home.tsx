import { useEffect, useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'

interface Job {
  id: number
  slug: string
  company: string
  title: string
  location: string | null
  start_date: string
  end_date: string | null
  summary: string | null
  tech_stack: string[] | null
  sort_order: number
}

interface Competency {
  id: number
  slug: string
  name: string
  description: string | null
  icon: string | null
  sort_order: number
}

interface ResumeData {
  bio: string
  summary: string
  jobs: Job[]
  competencies: Competency[]
}

interface JobDetail {
  details: { id: number; detail_text: string; category: string | null; sort_order: number }[]
}

interface EvidenceItem {
  id: number
  job_id: number
  job_slug: string
  company: string
  highlight_text: string | null
  detail_text: string | null
}

interface CompetencyDetail {
  competency: Competency
  evidence: EvidenceItem[]
}

const CATEGORY_LABELS: Record<string, string> = {
  achievement: 'Achievements',
  responsibility: 'Responsibilities',
  'sub-engagement': 'Client Engagements',
}

function JobCard({ job }: { job: Job }) {
  const [open, setOpen] = useState(false)
  const [detail, setDetail] = useState<JobDetail | null>(null)
  const [detailLoading, setDetailLoading] = useState(false)

  async function toggle() {
    if (!open && !detail) {
      setDetailLoading(true)
      try {
        const res = await fetch(`/api/jobs/${job.slug}`)
        if (res.ok) setDetail(await res.json())
      } finally {
        setDetailLoading(false)
      }
    }
    setOpen(o => !o)
  }

  const groups: Record<string, typeof detail extends null ? never : JobDetail['details']> = {}
  if (detail) {
    for (const d of detail.details) {
      const cat = d.category ?? 'responsibility'
      if (!groups[cat]) groups[cat] = []
      groups[cat].push(d)
    }
  }

  return (
    <div className="relative sm:pl-12">
      <div className="absolute left-2.5 top-6 w-3 h-3 rounded-full bg-cyan-400 ring-2 ring-gray-900 hidden sm:block" />

      <div
        className="bg-gray-800 rounded-lg border border-gray-700 hover:border-cyan-500/50 transition cursor-pointer"
        onClick={toggle}
      >
        <div className="p-6">
          <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-2 mb-3">
            <div>
              <h3 className="text-lg font-semibold text-white">{job.company}</h3>
              <p className="text-cyan-400 text-sm">{job.title}</p>
            </div>
            <div className="text-right shrink-0">
              <span className="text-gray-400 text-sm font-mono">
                {job.start_date} – {job.end_date ?? 'Present'}
              </span>
            </div>
          </div>
          {job.summary && (
            <p className="text-gray-300 text-sm leading-relaxed mb-4">{job.summary}</p>
          )}
          {job.tech_stack && job.tech_stack.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {job.tech_stack.map(t => (
                <span key={t} className="text-xs bg-gray-700 text-gray-300 px-2 py-0.5 rounded">
                  {t}
                </span>
              ))}
            </div>
          )}
          <div className="mt-3 text-xs text-gray-500">
            {open ? 'click to collapse ▲' : 'click to expand details ▼'}
          </div>
        </div>
      </div>

      {open && (
        <div className="mt-2 bg-gray-800/60 rounded-lg border border-gray-700 p-6">
          {detailLoading && <p className="text-gray-500 text-sm">Loading…</p>}
          {detail && detail.details.length === 0 && (
            <p className="text-gray-500 italic text-sm">No details available.</p>
          )}
          {detail && detail.details.length > 0 && (
            <div className="space-y-4">
              {Object.entries(groups).map(([cat, items]) => (
                <div key={cat}>
                  <h4 className="text-xs font-semibold uppercase tracking-wider text-cyan-500 mb-2">
                    {CATEGORY_LABELS[cat] ?? cat}
                  </h4>
                  <ul className="space-y-2">
                    {items.map(item => (
                      <li key={item.id} className="flex gap-2 text-gray-300 text-sm">
                        <span className="text-cyan-400 mt-0.5 shrink-0">▸</span>
                        <span>{item.detail_text}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function CompetencyCard({ comp, onJobClick }: { comp: Competency; onJobClick: (slug: string) => void }) {
  const [open, setOpen] = useState(false)
  const [detail, setDetail] = useState<CompetencyDetail | null>(null)
  const [loading, setLoading] = useState(false)

  async function toggle() {
    if (!open && !detail) {
      setLoading(true)
      try {
        const res = await fetch(`/api/competencies/${comp.slug}`)
        if (res.ok) setDetail(await res.json())
      } finally {
        setLoading(false)
      }
    }
    setOpen(o => !o)
  }

  const byCompany: Record<string, { slug: string; items: EvidenceItem[] }> = {}
  if (detail) {
    for (const ev of detail.evidence) {
      if (!byCompany[ev.company]) byCompany[ev.company] = { slug: ev.job_slug, items: [] }
      byCompany[ev.company].items.push(ev)
    }
  }

  return (
    <div
      className="bg-gray-800 rounded-lg border border-gray-700 hover:border-cyan-500/50 transition cursor-pointer"
      onClick={toggle}
    >
      <div className="p-6">
        <div className="flex items-start gap-3 mb-3">
          {comp.icon && <span className="text-2xl">{comp.icon}</span>}
          <div>
            <h3 className="text-lg font-semibold text-white">{comp.name}</h3>
            {comp.description && (
              <p className="text-gray-400 text-sm mt-1">{comp.description}</p>
            )}
          </div>
        </div>
        <div className="mt-3 text-xs text-gray-500">
          {open ? 'click to collapse ▲' : 'click to see evidence ▼'}
        </div>
      </div>

      {open && (
        <div className="border-t border-gray-700 p-6" onClick={e => e.stopPropagation()}>
          {loading && <p className="text-gray-500 text-sm">Loading…</p>}
          {detail && detail.evidence.length === 0 && (
            <p className="text-gray-500 italic text-sm">No evidence linked yet.</p>
          )}
          {detail && detail.evidence.length > 0 && (
            <div className="space-y-4">
              {Object.entries(byCompany).map(([company, group]) => (
                <div key={company}>
                  <button
                    className="text-sm font-semibold text-cyan-400 hover:text-cyan-300 transition mb-2 block"
                    onClick={() => onJobClick(group.slug)}
                  >
                    {company} →
                  </button>
                  <ul className="space-y-1.5">
                    {group.items.map(ev => {
                      const text = ev.highlight_text ?? ev.detail_text ?? ''
                      return text ? (
                        <li key={ev.id} className="flex gap-2 text-gray-300 text-sm">
                          <span className="text-cyan-400 mt-0.5 shrink-0">▸</span>
                          <span>{text}</span>
                        </li>
                      ) : null
                    })}
                  </ul>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default function Home() {
  const [resume, setResume] = useState<ResumeData | null>(null)
  const [loading, setLoading] = useState(true)
  const [downloadOpen, setDownloadOpen] = useState(false)
  const downloadRef = useRef<HTMLDivElement>(null)
  const [searchParams, setSearchParams] = useSearchParams()
  const view = searchParams.get('view') === 'capabilities' ? 'capabilities' : 'timeline'

  useEffect(() => {
    fetch('/api/resume')
      .then(r => r.json())
      .then((data: ResumeData) => setResume(data))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (downloadRef.current && !downloadRef.current.contains(e.target as Node)) {
        setDownloadOpen(false)
      }
    }
    document.addEventListener('click', handleClick)
    return () => document.removeEventListener('click', handleClick)
  }, [])

  function setView(v: 'timeline' | 'capabilities') {
    setSearchParams(v === 'timeline' ? {} : { view: 'capabilities' }, { replace: true })
  }

  function handleJobClick(slug: string) {
    setView('timeline')
    setTimeout(() => {
      const el = document.querySelector(`[data-job-slug="${slug}"]`)
      if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }, 50)
  }

  return (
    <>
      <Helmet>
        <title>Sharful Islam — Portfolio</title>
      </Helmet>

      <section className="bg-gradient-to-b from-gray-800 to-gray-900 py-12">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col sm:flex-row items-center justify-between gap-6">
            <div>
              <h1 className="text-4xl font-bold text-white mb-1">Sharful Islam</h1>
              <p className="text-cyan-400 text-lg">Full-Stack SaaS Engineer · 20+ years</p>
            </div>

            <div className="flex flex-col sm:flex-row items-center gap-3">
              <div className="flex items-center bg-gray-800 rounded-lg p-1 border border-gray-700">
                <button
                  onClick={() => setView('timeline')}
                  className={`px-4 py-2 rounded-md text-sm font-medium transition ${
                    view === 'timeline'
                      ? 'bg-cyan-600/20 text-cyan-400'
                      : 'text-gray-400 hover:text-white'
                  }`}
                >
                  📅 Timeline
                </button>
                <button
                  onClick={() => setView('capabilities')}
                  className={`px-4 py-2 rounded-md text-sm font-medium transition ${
                    view === 'capabilities'
                      ? 'bg-cyan-600/20 text-cyan-400'
                      : 'text-gray-400 hover:text-white'
                  }`}
                >
                  ⚡ Capabilities
                </button>
              </div>

              <div className="relative" ref={downloadRef}>
                <button
                  onClick={() => setDownloadOpen(o => !o)}
                  className="flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-medium
                             bg-cyan-600 hover:bg-cyan-500 text-white transition"
                >
                  Download Resume
                  <span className="text-xs">{downloadOpen ? '▴' : '▾'}</span>
                </button>
                {downloadOpen && (
                  <div className="absolute right-0 mt-2 w-64 bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-10 overflow-hidden">
                    <div className="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-gray-500 border-b border-gray-700">
                      PDF
                    </div>
                    <a
                      href="/resume/sharful-islam-resume-chronological.pdf"
                      download
                      className="flex items-center gap-2 px-4 py-2.5 text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition"
                    >
                      Chronological
                    </a>
                    <a
                      href="/resume/sharful-islam-resume-functional.pdf"
                      download
                      className="flex items-center gap-2 px-4 py-2.5 text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition"
                    >
                      Functional (by skill)
                    </a>
                    <div className="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-gray-500 border-t border-b border-gray-700">
                      DOCX
                    </div>
                    <a
                      href="/resume/sharful-islam-resume-chronological.docx"
                      download
                      className="flex items-center gap-2 px-4 py-2.5 text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition"
                    >
                      Chronological
                    </a>
                    <a
                      href="/resume/sharful-islam-resume-functional.docx"
                      download
                      className="flex items-center gap-2 px-4 py-2.5 text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition"
                    >
                      Functional (by skill)
                    </a>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </section>

      {loading && (
        <div className="flex justify-center py-20">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyan-400" />
        </div>
      )}

      {resume && view === 'timeline' && (
        <section className="py-12">
          <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="relative">
              <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-gray-700 hidden sm:block" />
              <div className="space-y-6">
                {resume.jobs.map(job => (
                  <div key={job.id} data-job-slug={job.slug}>
                    <JobCard job={job} />
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>
      )}

      {resume && view === 'capabilities' && (
        <section className="py-12">
          <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {resume.competencies.map(comp => (
                <CompetencyCard key={comp.id} comp={comp} onJobClick={handleJobClick} />
              ))}
            </div>
          </div>
        </section>
      )}
    </>
  )
}
