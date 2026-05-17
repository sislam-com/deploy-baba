import { useEffect, useState, FormEvent } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'

interface AboutForm {
  page: string
  slug: string
  heading: string
  body: string
  icon: string
  sort_order: number
}

const EMPTY: AboutForm = { page: '', slug: '', heading: '', body: '', icon: '', sort_order: 0 }

export default function AboutDetail() {
  const { id } = useParams<{ id: string }>()
  const isNew = id === 'new'
  const navigate = useNavigate()

  const [form, setForm] = useState<AboutForm>(EMPTY)
  const [loading, setLoading] = useState(!isNew)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (isNew) return
    fetch('/api/about/sections')
      .then(r => r.json())
      .then((items: { id: number; page: string; slug: string; heading: string | null; body: string; icon: string | null; sort_order: number }[]) => {
        const item = items.find(s => s.id === Number(id))
        if (!item) { setError('Section not found'); return }
        setForm({
          page: item.page ?? '',
          slug: item.slug ?? '',
          heading: item.heading ?? '',
          body: item.body ?? '',
          icon: item.icon ?? '',
          sort_order: item.sort_order ?? 0,
        })
      })
      .catch(() => setError('Failed to load section'))
      .finally(() => setLoading(false))
  }, [id, isNew])

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    const body = {
      ...form,
      heading: form.heading || null,
      icon: form.icon || null,
      sort_order: Number(form.sort_order),
    }
    const res = await fetch(
      isNew ? '/api/admin/about' : `/api/admin/about/${id}`,
      { method: isNew ? 'POST' : 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) },
    ).catch(() => null)

    setSaving(false)
    if (!res || !res.ok) { setError(res ? await res.text() : 'Network error'); return }
    navigate('/dashboard/about')
  }

  async function handleDelete() {
    if (!confirm('Delete this section?')) return
    const res = await fetch(`/api/admin/about/${id}`, { method: 'DELETE' }).catch(() => null)
    if (!res || !res.ok) { setError('Delete failed'); return }
    navigate('/dashboard/about')
  }

  if (loading) return <div className="p-8 text-gray-500 text-sm">Loading…</div>

  return (
    <div className="p-8 max-w-2xl">
      <div className="flex items-center gap-3 mb-6">
        <Link to="/dashboard/about" className="text-gray-500 hover:text-gray-300 text-sm transition">← About sections</Link>
        <h1 className="text-2xl font-bold text-white">{isNew ? 'New section' : 'Edit section'}</h1>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      <form onSubmit={handleSubmit} className="space-y-4">
        {(['page', 'slug', 'heading', 'icon'] as const).map(field => (
          <div key={field}>
            <label htmlFor={`about-${field}`} className="block text-xs font-medium text-gray-400 mb-1 capitalize">{field}</label>
            <input
              id={`about-${field}`}
              type="text"
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
              required={field === 'page' || field === 'slug'}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                         focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>
        ))}

        <div>
          <label htmlFor="about-body" className="block text-xs font-medium text-gray-400 mb-1">Body</label>
          <textarea
            id="about-body"
            rows={6}
            required
            value={form.body}
            onChange={e => setForm(f => ({ ...f, body: e.target.value }))}
            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                       focus:outline-none focus:ring-2 focus:ring-cyan-500 resize-y"
          />
        </div>

        <div>
          <label htmlFor="about-sort-order" className="block text-xs font-medium text-gray-400 mb-1">Sort order</label>
          <input
            id="about-sort-order"
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
            <button type="button" onClick={handleDelete} className="text-red-400 hover:text-red-300 text-sm transition">
              Delete
            </button>
          )}
        </div>
      </form>
    </div>
  )
}
