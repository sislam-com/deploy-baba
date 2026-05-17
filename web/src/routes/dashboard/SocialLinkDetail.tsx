import { useEffect, useState, FormEvent } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'

interface SocialLinkForm {
  platform: string
  url: string
  label: string
  icon: string
  visible: boolean
  sort_order: number
}

const EMPTY: SocialLinkForm = {
  platform: '', url: '', label: '', icon: '', visible: true, sort_order: 0,
}

export default function SocialLinkDetail() {
  const { id } = useParams<{ id: string }>()
  const isNew = id === 'new'
  const navigate = useNavigate()

  const [form, setForm] = useState<SocialLinkForm>(EMPTY)
  const [loading, setLoading] = useState(!isNew)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (isNew) return
    fetch('/api/social-links')
      .then(r => r.json())
      .then((items: { id: number; platform: string; url: string; label: string; icon: string | null; visible: boolean; sort_order: number }[]) => {
        const item = items.find(s => s.id === Number(id))
        if (!item) { setError('Social link not found'); return }
        setForm({
          platform: item.platform ?? '',
          url: item.url ?? '',
          label: item.label ?? '',
          icon: item.icon ?? '',
          visible: item.visible,
          sort_order: item.sort_order ?? 0,
        })
      })
      .catch(() => setError('Failed to load social link'))
      .finally(() => setLoading(false))
  }, [id, isNew])

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    const body = {
      ...form,
      icon: form.icon || null,
      sort_order: Number(form.sort_order),
    }
    const res = await fetch(
      isNew ? '/api/admin/social-links' : `/api/admin/social-links/${id}`,
      { method: isNew ? 'POST' : 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) },
    ).catch(() => null)

    setSaving(false)
    if (!res || !res.ok) { setError(res ? await res.text() : 'Network error'); return }
    navigate('/dashboard/social-links')
  }

  async function handleDelete() {
    if (!confirm('Delete this social link?')) return
    const res = await fetch(`/api/admin/social-links/${id}`, { method: 'DELETE' }).catch(() => null)
    if (!res || !res.ok) { setError('Delete failed'); return }
    navigate('/dashboard/social-links')
  }

  if (loading) return <div className="p-8 text-gray-500 text-sm">Loading…</div>

  return (
    <div className="p-8 max-w-2xl">
      <div className="flex items-center gap-3 mb-6">
        <Link to="/dashboard/social-links" className="text-gray-500 hover:text-gray-300 text-sm transition">← Social links</Link>
        <h1 className="text-2xl font-bold text-white">{isNew ? 'New social link' : 'Edit social link'}</h1>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      <form onSubmit={handleSubmit} className="space-y-4">
        {(['platform', 'url', 'label', 'icon'] as const).map(field => (
          <div key={field}>
            <label htmlFor={`social-${field}`} className="block text-xs font-medium text-gray-400 mb-1 capitalize">{field}</label>
            <input
              id={`social-${field}`}
              type={field === 'url' ? 'url' : 'text'}
              value={form[field]}
              onChange={e => setForm(f => ({ ...f, [field]: e.target.value }))}
              required={field !== 'icon'}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                         focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>
        ))}

        <div className="flex items-center gap-3">
          <input
            type="checkbox"
            id="social-visible"
            checked={form.visible}
            onChange={e => setForm(f => ({ ...f, visible: e.target.checked }))}
            className="h-4 w-4 rounded border-gray-600 bg-gray-800 text-cyan-500 focus:ring-cyan-500"
          />
          <label htmlFor="social-visible" className="text-sm text-gray-300">Visible in nav</label>
        </div>

        <div>
          <label htmlFor="social-sort-order" className="block text-xs font-medium text-gray-400 mb-1">Sort order</label>
          <input
            id="social-sort-order"
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
