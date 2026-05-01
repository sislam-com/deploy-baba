import { useEffect, useState, FormEvent } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'

interface JobForm {
  slug: string
  company: string
  title: string
  location: string
  start_date: string
  end_date: string
  summary: string
  tech_stack: string
  sort_order: number
}

const EMPTY: JobForm = {
  slug: '', company: '', title: '', location: '', start_date: '',
  end_date: '', summary: '', tech_stack: '', sort_order: 0,
}

export default function JobDetail() {
  const { id } = useParams<{ id: string }>()
  const isNew = id === 'new'
  const navigate = useNavigate()

  const [form, setForm] = useState<JobForm>(EMPTY)
  const [loading, setLoading] = useState(!isNew)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (isNew) return
    fetch('/api/jobs')
      .then(r => r.json())
      .then((jobs: { id: number; slug: string; company: string; title: string; location: string | null; start_date: string; end_date: string | null; summary: string | null; tech_stack: string[] | null; sort_order: number }[]) => {
        const job = jobs.find(j => j.id === Number(id))
        if (!job) { setError('Job not found'); return }
        setForm({
          slug: job.slug ?? '',
          company: job.company ?? '',
          title: job.title ?? '',
          location: job.location ?? '',
          start_date: job.start_date ?? '',
          end_date: job.end_date ?? '',
          summary: job.summary ?? '',
          tech_stack: job.tech_stack?.join(', ') ?? '',
          sort_order: job.sort_order ?? 0,
        })
      })
      .catch(() => setError('Failed to load job'))
      .finally(() => setLoading(false))
  }, [id, isNew])

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    const body = {
      ...form,
      tech_stack: form.tech_stack || null,
      end_date: form.end_date || null,
      location: form.location || null,
      summary: form.summary || null,
      sort_order: Number(form.sort_order),
    }
    const res = await fetch(isNew ? '/api/admin/jobs' : `/api/admin/jobs/${id}`, {
      method: isNew ? 'POST' : 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }).catch(() => null)

    setSaving(false)
    if (!res || !res.ok) {
      setError(res ? await res.text() : 'Network error')
      return
    }
    navigate('/dashboard/jobs')
  }

  async function handleDelete() {
    if (!confirm('Delete this job?')) return
    const res = await fetch(`/api/admin/jobs/${id}`, { method: 'DELETE' }).catch(() => null)
    if (!res || !res.ok) { setError('Delete failed'); return }
    navigate('/dashboard/jobs')
  }

  if (loading) return <div className="p-8 text-gray-500 text-sm">Loading…</div>

  return (
    <div className="p-8 max-w-2xl">
      <div className="flex items-center gap-3 mb-6">
        <Link to="/dashboard/jobs" className="text-gray-500 hover:text-gray-300 text-sm transition">← Jobs</Link>
        <h1 className="text-2xl font-bold text-white">{isNew ? 'New job' : 'Edit job'}</h1>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      <form onSubmit={handleSubmit} className="space-y-4">
        {(['slug', 'company', 'title', 'location', 'start_date', 'end_date'] as const).map(field => (
          <div key={field}>
            <label className="block text-xs font-medium text-gray-400 mb-1 capitalize">{field.replace('_', ' ')}</label>
            <input
              type="text"
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
              required={field === 'slug' || field === 'company' || field === 'title' || field === 'start_date'}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                         placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>
        ))}

        <div>
          <label className="block text-xs font-medium text-gray-400 mb-1">Summary</label>
          <textarea
            rows={3}
            value={form.summary}
            onChange={e => setForm(f => ({ ...f, summary: e.target.value }))}
            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                       focus:outline-none focus:ring-2 focus:ring-cyan-500 resize-none"
          />
        </div>

        <div>
          <label className="block text-xs font-medium text-gray-400 mb-1">Tech stack (comma-separated)</label>
          <input
            type="text"
            value={form.tech_stack}
            onChange={e => setForm(f => ({ ...f, tech_stack: e.target.value }))}
            placeholder="Rust, React, AWS"
            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                       placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-cyan-500"
          />
        </div>

        <div>
          <label className="block text-xs font-medium text-gray-400 mb-1">Sort order</label>
          <input
            type="number"
            value={form.sort_order}
            onChange={e => setForm(f => ({ ...f, sort_order: Number(e.target.value) }))}
            className="w-24 bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                       focus:outline-none focus:ring-2 focus:ring-cyan-500"
          />
        </div>

        <div className="flex items-center gap-3 pt-2">
          <button
            type="submit"
            disabled={saving}
            className="bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 text-white text-sm
                       font-semibold px-5 py-2 rounded-lg transition"
          >
            {saving ? 'Saving…' : 'Save'}
          </button>
          {!isNew && (
            <button
              type="button"
              onClick={handleDelete}
              className="text-red-400 hover:text-red-300 text-sm transition"
            >
              Delete
            </button>
          )}
        </div>
      </form>
    </div>
  )
}
