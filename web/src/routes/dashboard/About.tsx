import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'

interface AboutSection {
  id: number
  page: string
  slug: string
  heading: string | null
  body: string
  sort_order: number
}

export default function About() {
  const [items, setItems] = useState<AboutSection[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch('/api/about/sections')
      .then(r => r.json())
      .then((data: AboutSection[]) => setItems(data))
      .catch(() => setError('Failed to load about sections'))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-white">About sections</h1>
        <Link
          to="/dashboard/about/new"
          className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          + New section
        </Link>
      </div>

      {loading && <p className="text-gray-500 text-sm">Loading…</p>}
      {error && <p className="text-red-400 text-sm">{error}</p>}

      <div className="space-y-2">
        {items.map(item => (
          <Link
            key={item.id}
            to={`/dashboard/about/${item.id}`}
            className="flex items-center justify-between bg-gray-800 border border-gray-700
                       hover:border-gray-500 rounded-xl px-5 py-4 transition"
          >
            <div className="min-w-0">
              <p className="text-white font-medium truncate">{item.heading ?? item.slug}</p>
              <p className="text-xs text-gray-500 mt-0.5">
                <span className="font-mono bg-gray-700 px-1.5 py-0.5 rounded text-gray-300">{item.page}</span>
                <span className="ml-2 font-mono text-gray-500">{item.slug}</span>
              </p>
            </div>
            <p className="text-xs text-gray-600 shrink-0 ml-4">#{item.sort_order}</p>
          </Link>
        ))}
      </div>
    </div>
  )
}
