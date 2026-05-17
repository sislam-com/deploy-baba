import { useEffect, useState, FormEvent } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'

interface CompetencyForm {
  slug: string
  name: string
  description: string
  icon: string
  sort_order: number
}

const EMPTY: CompetencyForm = { slug: '', name: '', description: '', icon: '', sort_order: 0 }

export default function CompetencyDetail() {
  const { id } = useParams<{ id: string }>()
  const isNew = id === 'new'
  const navigate = useNavigate()

  const [form, setForm] = useState<CompetencyForm>(EMPTY)
  const [loading, setLoading] = useState(!isNew)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (isNew) return
    fetch('/api/competencies')
      .then(r => r.json())
      .then((items: { id: number; slug: string; name: string; description: string | null; icon: string | null; sort_order: number }[]) => {
        const item = items.find(c => c.id === Number(id))
        if (!item) { setError('Competency not found'); return }
        setForm({
          slug: item.slug ?? '',
          name: item.name ?? '',
          description: item.description ?? '',
          icon: item.icon ?? '',
          sort_order: item.sort_order ?? 0,
        })
      })
      .catch(() => setError('Failed to load competency'))
      .finally(() => setLoading(false))
  }, [id, isNew])

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    const body = {
      ...form,
      description: form.description || null,
      icon: form.icon || null,
      sort_order: Number(form.sort_order),
    }
    const res = await fetch(
      isNew ? '/api/admin/competencies' : `/api/admin/competencies/${id}`,
      { method: isNew ? 'POST' : 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) },
    ).catch(() => null)

    setSaving(false)
    if (!res || !res.ok) { setError(res ? await res.text() : 'Network error'); return }
    navigate('/dashboard/competencies')
  }

  async function handleDelete() {
    if (!confirm('Delete this competency?')) return
    const res = await fetch(`/api/admin/competencies/${id}`, { method: 'DELETE' }).catch(() => null)
    if (!res || !res.ok) { setError('Delete failed'); return }
    navigate('/dashboard/competencies')
  }

  if (loading) return <div className="p-8 text-gray-500 text-sm">Loading…</div>

  return (
    <div className="p-8 max-w-2xl">
      <div className="flex items-center gap-3 mb-6">
        <Link to="/dashboard/competencies" className="text-gray-500 hover:text-gray-300 text-sm transition">← Competencies</Link>
        <h1 className="text-2xl font-bold text-white">{isNew ? 'New competency' : 'Edit competency'}</h1>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      <form onSubmit={handleSubmit} className="space-y-4">
        {(['slug', 'name', 'icon'] as const).map(field => (
          <div key={field}>
            <label htmlFor={`comp-${field}`} className="block text-xs font-medium text-gray-400 mb-1 capitalize">{field}</label>
            <input
              id={`comp-${field}`}
              type="text"
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
              required={field === 'slug' || field === 'name'}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                         focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>
        ))}

        <div>
          <label htmlFor="comp-description" className="block text-xs font-medium text-gray-400 mb-1">Description</label>
          <textarea
            id="comp-description"
            rows={3}
            value={form.description}
            onChange={e => setForm(f => ({ ...f, description: e.target.value }))}
            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                       focus:outline-none focus:ring-2 focus:ring-cyan-500 resize-none"
          />
        </div>

        <div>
          <label htmlFor="comp-sort-order" className="block text-xs font-medium text-gray-400 mb-1">Sort order</label>
          <input
            id="comp-sort-order"
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
