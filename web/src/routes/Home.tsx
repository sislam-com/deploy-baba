import { useEffect, useMemo, useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'
import Ask from './Ask'
import SvgIcon from '../components/SvgIcon'

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
  name: string
  title: string
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

const COMPETENCY_ICON_MAP: Record<string, string> = {
  'platform-architecture': 'server',
  'cloud-infrastructure': 'cloud',
  'frontend-engineering': 'monitor',
  'ai-augmented-dev': 'cpu',
  'ai-llm-systems': 'brain',
  'technical-leadership': 'users',
  'saas-product': 'rocket',
}

function StatPill({ value, label }: { value: string; label: string }) {
  return (
    <div className="flex flex-col items-center">
      <span className="text-lg sm:text-xl font-bold text-white">{value}</span>
      <span className="text-xs text-gray-500 uppercase tracking-wider">{label}</span>
    </div>
  )
}

function TechStrip({ jobs }: { jobs: Job[] }) {
  const tags = useMemo(() => {
    const freq: Record<string, number> = {}
    for (const job of jobs) {
      for (const tech of job.tech_stack ?? []) {
        freq[tech] = (freq[tech] ?? 0) + 1
      }
    }
    return Object.entries(freq)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 12)
      .map(([name]) => name)
  }, [jobs])

  return (
    <div className="flex justify-center gap-2 flex-wrap scrollbar-hide">
      {tags.map((tag, i) => (
        <span
          key={tag}
          className="text-xs px-3 py-1 rounded-full border border-gray-700 text-gray-300 bg-gray-800/50
                     whitespace-nowrap animate-fadeIn"
          style={{ animationDelay: `${300 + i * 40}ms` }}
        >
          {tag}
        </span>
      ))}
    </div>
  )
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
    <div className="relative sm:pl-12" id={job.slug}>
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
                        <span className="text-cyan-400 mt-0.5 shrink-0">{'▸'}</span>
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
    <div id={comp.slug}>
      <div
        className="bg-gray-800 rounded-lg border border-gray-700 hover:border-cyan-500/50 transition cursor-pointer"
        onClick={toggle}
      >
      <div className="p-6">
        <div className="flex items-start gap-3 mb-3">
          <SvgIcon
            name={COMPETENCY_ICON_MAP[comp.slug] ?? 'diamond'}
            className="w-6 h-6 text-cyan-400 shrink-0 mt-0.5"
          />
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
                    {company} {'→'}
                  </button>
                  <ul className="space-y-1.5">
                    {group.items.map(ev => {
                      const text = ev.highlight_text ?? ev.detail_text ?? ''
                      return text ? (
                        <li key={ev.id} className="flex gap-2 text-gray-300 text-sm">
                          <span className="text-cyan-400 mt-0.5 shrink-0">{'▸'}</span>
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
    </div>
  )
}

export default function Home() {
  const [resume, setResume] = useState<ResumeData | null>(null)
  const [loading, setLoading] = useState(true)
  const [downloadOpen, setDownloadOpen] = useState(false)
  const downloadRef = useRef<HTMLDivElement>(null)
  const [searchParams, setSearchParams] = useSearchParams()
  const view = searchParams.get('view') === 'timeline' ? 'timeline' :
                searchParams.get('view') === 'capabilities' ? 'capabilities' : 'ask'

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

  const heroStats = useMemo(() => {
    if (!resume) return null

    const startDates = resume.jobs.map(j => {
      const [y, m] = j.start_date.split('-').map(Number)
      return new Date(y, (m ?? 1) - 1).getTime()
    })
    const endDates = resume.jobs.map(j => {
      if (!j.end_date) return Date.now()
      const [y, m] = j.end_date.split('-').map(Number)
      return new Date(y, (m ?? 1) - 1).getTime()
    })
    const years = Math.floor(
      (Math.max(...endDates) - Math.min(...startDates)) / (365.25 * 24 * 60 * 60 * 1000)
    )

    const freq: Record<string, number> = {}
    for (const job of resume.jobs) {
      for (const tech of job.tech_stack ?? []) {
        freq[tech] = (freq[tech] ?? 0) + 1
      }
    }
    const topTech = Object.entries(freq)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 3)
      .map(([name]) => name)

    return { years, skillCount: resume.competencies.length, topTech }
  }, [resume])

  function setView(v: 'timeline' | 'capabilities' | 'ask') {
    setSearchParams(v === 'ask' ? {} : { view: v }, { replace: true })
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
        <title>Portfolio</title>
      </Helmet>

      {/* Hero */}
      <section className="bg-gradient-to-b from-gray-800 to-gray-900 pt-12 pb-8">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          {/* Name + Title */}
          <div className="text-center animate-fadeInUp">
            <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold text-white tracking-tight">
              {resume?.name ?? ' '}
            </h1>
            <p className="text-xl sm:text-2xl text-cyan-400 font-medium mt-2">
              {resume?.title ?? ' '}
            </p>
            {resume?.bio && (
              <p className="text-base sm:text-lg text-gray-400 mt-4 max-w-2xl mx-auto leading-relaxed">
                {resume.bio.split('.').slice(0, 2).join('.').trim()}.
              </p>
            )}
          </div>

          {/* Stats strip */}
          {heroStats && (
            <div
              className="flex flex-col sm:flex-row items-center justify-center gap-4 sm:gap-8 mt-8 animate-fadeInUp"
              style={{ animationDelay: '150ms' }}
            >
              <StatPill value={`${heroStats.years}+`} label="years" />
              <span className="text-gray-700 text-2xl font-thin hidden sm:inline">|</span>
              <StatPill value={`${heroStats.skillCount}`} label="core skills" />
              <span className="text-gray-700 text-2xl font-thin hidden sm:inline">|</span>
              <StatPill value={heroStats.topTech.join(' · ')} label="top tech" />
            </div>
          )}

          {/* Tech strip */}
          {resume && (
            <div className="mt-6 animate-fadeIn" style={{ animationDelay: '300ms' }}>
              <TechStrip jobs={resume.jobs} />
            </div>
          )}

          {/* Tabs + Download */}
          <div
            className="flex flex-col sm:flex-row items-center justify-center gap-3 mt-8 animate-fadeIn"
            style={{ animationDelay: '400ms' }}
          >
            <div className="flex items-center bg-gray-800/80 rounded-full p-1 border border-gray-700">
              <button
                onClick={() => setView('timeline')}
                className={`flex items-center gap-1.5 px-4 py-2 rounded-full text-sm font-medium transition ${
                  view === 'timeline'
                    ? 'bg-cyan-600/20 text-cyan-400 shadow-sm shadow-cyan-500/10'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                <SvgIcon name="calendar" className="w-4 h-4" />
                Timeline
              </button>
              <button
                onClick={() => setView('capabilities')}
                className={`flex items-center gap-1.5 px-4 py-2 rounded-full text-sm font-medium transition ${
                  view === 'capabilities'
                    ? 'bg-cyan-600/20 text-cyan-400 shadow-sm shadow-cyan-500/10'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                <SvgIcon name="bolt" className="w-4 h-4" />
                Capabilities
              </button>
              <button
                onClick={() => setView('ask')}
                className={`flex items-center gap-1.5 px-4 py-2 rounded-full text-sm font-medium transition ${
                  view === 'ask'
                    ? 'bg-cyan-600/20 text-cyan-400 shadow-sm shadow-cyan-500/10'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                <SvgIcon name="chat" className="w-4 h-4" />
                Ask AI
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
                {resume.jobs.map((job, i) => (
                  <div
                    key={job.id}
                    data-job-slug={job.slug}
                    className="animate-fadeInUp"
                    style={{ animationDelay: `${i * 80}ms` }}
                  >
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
              {resume.competencies.map((comp, i) => (
                <div
                  key={comp.id}
                  className="animate-scaleIn"
                  style={{ animationDelay: `${i * 60}ms` }}
                >
                  <CompetencyCard comp={comp} onJobClick={handleJobClick} />
                </div>
              ))}
            </div>
          </div>
        </section>
      )}

      {view === 'ask' && (
        <section className="py-8 animate-fadeIn">
          <Ask embedded />
        </section>
      )}
    </>
  )
}
