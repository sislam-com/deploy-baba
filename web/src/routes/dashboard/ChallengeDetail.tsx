import { useEffect, useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'

interface Challenge {
  id: number
  slug: string
  title: string
  job_id: number | null
  description: string
  short_description: string | null
  tech_stack: string[] | null
  category: string | null
  url: string | null
  image_url: string | null
  problem: string | null
  constraints: string | null
  decisions: string | null
  implementation: string | null
  outcomes: string | null
  metrics: string | null
  related_job_slug: string | null
  related_plan_module: string | null
  related_adr: string | null
  featured: boolean
  sort_order: number
}

interface Job {
  id: number
  slug: string
  company: string
  title: string
}

interface ChallengeForm {
  slug: string
  title: string
  job_id: string
  description: string
  short_description: string
  tech_stack: string
  category: string
  url: string
  image_url: string
  problem: string
  constraints: string
  decisions: string
  implementation: string
  outcomes: string
  metrics: string
  related_job_slug: string
  related_plan_module: string
  related_adr: string
  featured: boolean
  sort_order: number
}

const EMPTY: ChallengeForm = {
  slug: '',
  title: '',
  job_id: '',
  description: '',
  short_description: '',
  tech_stack: '',
  category: '',
  url: '',
  image_url: '',
  problem: '',
  constraints: '',
  decisions: '',
  implementation: '',
  outcomes: '',
  metrics: '',
  related_job_slug: '',
  related_plan_module: '',
  related_adr: '',
  featured: false,
  sort_order: 0,
}

export default function ChallengeDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const isNew = id === 'new'

  const [form, setForm] = useState<ChallengeForm>(EMPTY)
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(!isNew)
  const [error, setError] = useState<string | null>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)

  useEffect(() => {
    fetch('/api/jobs')
      .then(r => r.json())
      .then((data: Job[]) => setJobs(data))
      .catch(() => {})

    if (isNew) return

    fetch('/api/challenges')
      .then(r => r.json())
      .then((data: Challenge[]) => {
        const ch = data.find(c => c.id === Number(id))
        if (!ch) {
          setError('Challenge not found')
          return
        }
        setForm({
          slug: ch.slug,
          title: ch.title,
          job_id: ch.job_id?.toString() ?? '',
          description: ch.description,
          short_description: ch.short_description ?? '',
          tech_stack: ch.tech_stack?.join(', ') ?? '',
          category: ch.category ?? '',
          url: ch.url ?? '',
          image_url: ch.image_url ?? '',
          problem: ch.problem ?? '',
          constraints: ch.constraints ?? '',
          decisions: ch.decisions ?? '',
          implementation: ch.implementation ?? '',
          outcomes: ch.outcomes ?? '',
          metrics: ch.metrics ?? '',
          related_job_slug: ch.related_job_slug ?? '',
          related_plan_module: ch.related_plan_module ?? '',
          related_adr: ch.related_adr ?? '',
          featured: ch.featured,
          sort_order: ch.sort_order,
        })
      })
      .catch(() => setError('Failed to load challenge'))
      .finally(() => setLoading(false))
  }, [id, isNew])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    const body = {
      slug: form.slug,
      title: form.title,
      job_id: form.job_id ? Number(form.job_id) : null,
      description: form.description,
      short_description: form.short_description || null,
      tech_stack: form.tech_stack || null,
      category: form.category || null,
      url: form.url || null,
      image_url: form.image_url || null,
      problem: form.problem || null,
      constraints: form.constraints || null,
      decisions: form.decisions || null,
      implementation: form.implementation || null,
      outcomes: form.outcomes || null,
      metrics: form.metrics || null,
      related_job_slug: form.related_job_slug || null,
      related_plan_module: form.related_plan_module || null,
      related_adr: form.related_adr || null,
      featured: form.featured,
      sort_order: form.sort_order,
    }

    const url = isNew ? '/api/v1/admin/challenges' : `/api/v1/admin/challenges/${id}`
    const method = isNew ? 'POST' : 'PUT'

    const res = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }).catch(() => null)

    if (!res || !res.ok) {
      setError(res ? await res.text() : 'Network error')
      return
    }
    navigate('/dashboard/challenges')
  }

  const handleDelete = async () => {
    const res = await fetch(`/api/v1/admin/challenges/${id}`, { method: 'DELETE' }).catch(() => null)
    if (!res || !res.ok) {
      setError(res ? await res.text() : 'Network error')
      return
    }
    navigate('/dashboard/challenges')
  }

  if (loading) return <p className="p-8 text-gray-500 text-sm">Loading…</p>

  return (
    <div className="p-8 max-w-2xl">
      <Link to="/dashboard/challenges" className="text-sm text-gray-400 hover:text-gray-200 transition">
        ← Challenges
      </Link>
      <h1 className="text-2xl font-bold text-white mt-4 mb-6">
        {isNew ? 'New Challenge' : 'Edit Challenge'}
      </h1>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      <form onSubmit={handleSubmit} className="space-y-4">
        {(['slug', 'title'] as const).map(field => (
          <label key={field} className="block">
            <span className="text-sm text-gray-400 capitalize">{field}</span>
            <input
              className="mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                         text-white focus:ring-2 focus:ring-cyan-500 focus:outline-none"
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
              required
            />
          </label>
        ))}

        <label className="block">
          <span className="text-sm text-gray-400">Job</span>
          <select
            className="mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                       text-white focus:ring-2 focus:ring-cyan-500 focus:outline-none"
            value={form.job_id}
            onChange={e => setForm(f => ({ ...f, job_id: e.target.value }))}
          >
            <option value="">None (independent project)</option>
            {jobs.map(j => (
              <option key={j.id} value={j.id}>
                {j.company} — {j.title}
              </option>
            ))}
          </select>
        </label>

        <label className="block">
          <span className="text-sm text-gray-400">Description</span>
          <textarea
            className="mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                       text-white focus:ring-2 focus:ring-cyan-500 focus:outline-none min-h-[120px]"
            value={form.description}
            onChange={e => setForm(f => ({ ...f, description: e.target.value }))}
            required
          />
        </label>

        {(['short_description', 'tech_stack', 'category', 'url', 'image_url'] as const).map(field => (
          <label key={field} className="block">
            <span className="text-sm text-gray-400">{field.replace(/_/g, ' ')}</span>
            <input
              className="mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                         text-white focus:ring-2 focus:ring-cyan-500 focus:outline-none"
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
              placeholder={field === 'tech_stack' ? 'Rust, React, AWS' : undefined}
            />
          </label>
        ))}

        {(['problem', 'constraints', 'decisions', 'implementation', 'outcomes', 'metrics'] as const).map(field => (
          <label key={field} className="block">
            <span className="text-sm text-gray-400">{field}</span>
            <textarea
              className="mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                         text-white focus:ring-2 focus:ring-cyan-500 focus:outline-none min-h-[96px]"
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
            />
          </label>
        ))}

        {(['related_job_slug', 'related_plan_module', 'related_adr'] as const).map(field => (
          <label key={field} className="block">
            <span className="text-sm text-gray-400">{field.replace(/_/g, ' ')}</span>
            <input
              className="mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                         text-white focus:ring-2 focus:ring-cyan-500 focus:outline-none"
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
            />
          </label>
        ))}

        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={form.featured}
            onChange={e => setForm(f => ({ ...f, featured: e.target.checked }))}
            className="w-4 h-4 rounded bg-gray-800 border-gray-700 text-cyan-600 focus:ring-cyan-500"
          />
          <span className="text-sm text-gray-400">Featured on homepage</span>
        </label>

        <label className="block">
          <span className="text-sm text-gray-400">Sort order</span>
          <input
            type="number"
            className="mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                       text-white focus:ring-2 focus:ring-cyan-500 focus:outline-none"
            value={form.sort_order}
            onChange={e => setForm(f => ({ ...f, sort_order: Number(e.target.value) }))}
          />
        </label>

        <div className="flex items-center gap-3 pt-4">
          <button
            type="submit"
            className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-5 py-2 rounded-lg transition"
          >
            {isNew ? 'Create' : 'Save'}
          </button>

          {!isNew && !confirmDelete && (
            <button
              type="button"
              onClick={() => setConfirmDelete(true)}
              className="text-sm text-red-400 hover:text-red-300 transition"
            >
              Delete
            </button>
          )}
          {!isNew && confirmDelete && (
            <button
              type="button"
              onClick={handleDelete}
              className="bg-red-700 hover:bg-red-600 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
            >
              Confirm delete
            </button>
          )}
        </div>
      </form>
    </div>
  )
}
