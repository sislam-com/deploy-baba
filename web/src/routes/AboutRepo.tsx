import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'

interface Section {
  id: number
  heading: string | null
  body: string
}

export default function AboutRepo() {
  const [sections, setSections] = useState<Section[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetch('/api/about/sections?page=repo')
      .then(r => r.json())
      .then((data: Section[]) => setSections(Array.isArray(data) ? data : []))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  return (
    <>
      <Helmet>
        <title>About This Repo — Sharful Islam</title>
      </Helmet>

      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="mb-10">
          <h1 className="text-4xl font-bold text-white mb-2">About</h1>
          <p className="text-gray-400">Learn more about me and this project.</p>
        </div>

        <div className="flex space-x-2 mb-10">
          <Link
            to="/about/me"
            className="px-5 py-2 rounded-lg text-sm font-medium text-gray-400 hover:text-white hover:bg-gray-700 transition"
          >
            About Me
          </Link>
          <span className="px-5 py-2 rounded-lg text-sm font-medium bg-cyan-600 text-white">
            About This Repo
          </span>
        </div>

        {loading && <p className="text-gray-500 text-sm">Loading…</p>}

        <div className="space-y-6">
          {sections.map(section => (
            <div key={section.id} className="bg-gray-800/50 rounded-lg p-6 border border-gray-700">
              {section.heading && (
                <h2 className="text-xl font-semibold text-cyan-400 mb-3">{section.heading}</h2>
              )}
              <p className="text-gray-300 leading-relaxed">{section.body}</p>
            </div>
          ))}
        </div>
      </div>
    </>
  )
}
