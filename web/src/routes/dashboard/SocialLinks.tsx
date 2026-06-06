import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'

interface SocialLink {
  id: number
  platform: string
  url: string
  label: string
  icon: string | null
  visible: boolean
  sort_order: number
}

export default function SocialLinks() {
  const [items, setItems] = useState<SocialLink[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch('/api/v1/social-links')
      .then(r => r.json())
      .then((data: SocialLink[]) => setItems(data))
      .catch(() => setError('Failed to load social links'))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-white">Social links</h1>
        <Link
          to="/dashboard/social-links/new"
          className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          + New link
        </Link>
      </div>

      {loading && <p className="text-gray-500 text-sm">Loading…</p>}
      {error && <p className="text-red-400 text-sm">{error}</p>}

      <div className="space-y-2">
        {items.map(item => (
          <Link
            key={item.id}
            to={`/dashboard/social-links/${item.id}`}
            className="flex items-center justify-between bg-gray-800 border border-gray-700
                       hover:border-gray-500 rounded-xl px-5 py-4 transition"
          >
            <div className="flex items-center gap-3">
              {item.icon && <span>{item.icon}</span>}
              <div>
                <p className="text-white font-medium">{item.label}</p>
                <p className="text-xs text-gray-500 font-mono">{item.platform}</p>
              </div>
            </div>
            <span
              className={`text-xs px-2 py-0.5 rounded-full ${
                item.visible ? 'bg-green-900/50 text-green-400' : 'bg-gray-700 text-gray-500'
              }`}
            >
              {item.visible ? 'visible' : 'hidden'}
            </span>
          </Link>
        ))}
      </div>
    </div>
  )
}
