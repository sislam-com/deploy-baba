import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'

interface Competency {
  id: number
  slug: string
  name: string
  description: string | null
  icon: string | null
  sort_order: number
}

export default function Competencies() {
  const [items, setItems] = useState<Competency[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch('/api/competencies')
      .then(r => r.json())
      .then((data: Competency[]) => setItems(data))
      .catch(() => setError('Failed to load competencies'))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-white">Competencies</h1>
        <Link
          to="/dashboard/competencies/new"
          className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          + New competency
        </Link>
      </div>

      {loading && <p className="text-gray-500 text-sm">Loading…</p>}
      {error && <p className="text-red-400 text-sm">{error}</p>}

      <div className="space-y-2">
        {items.map(item => (
          <Link
            key={item.id}
            to={`/dashboard/competencies/${item.id}`}
            className="flex items-center gap-4 bg-gray-800 border border-gray-700
                       hover:border-gray-500 rounded-xl px-5 py-4 transition"
          >
            {item.icon && <span className="text-xl">{item.icon}</span>}
            <div className="min-w-0">
              <p className="text-white font-medium">{item.name}</p>
              {item.description && (
                <p className="text-sm text-gray-400 truncate">{item.description}</p>
              )}
            </div>
          </Link>
        ))}
      </div>
    </div>
  )
}
