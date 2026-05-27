import { useEffect, useState } from 'react'
import { Helmet } from 'react-helmet-async'

interface LegalDocument {
  id: number
  slug: string
  title: string
  content: string
  updated_at: string
}

export default function Terms() {
  const [doc, setDoc] = useState<LegalDocument | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetch('/api/legal/terms')
      .then(r => {
        if (!r.ok) throw new Error('Failed to load')
        return r.json()
      })
      .then((data: LegalDocument) => setDoc(data))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  return (
    <>
      <Helmet>
        <title>Terms of Service — Sharful Islam</title>
      </Helmet>

      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <h1 className="text-4xl font-bold text-white mb-2">
          {doc?.title ?? 'Terms of Service'}
        </h1>
        {doc && (
          <p className="text-gray-400 text-sm mb-10">
            Last updated: {new Date(doc.updated_at).toLocaleDateString()}
          </p>
        )}

        {loading && <p className="text-gray-500 text-sm">Loading…</p>}

        {doc && (
          <div className="bg-gray-800/50 rounded-lg p-6 border border-gray-700">
            <div className="text-gray-300 leading-relaxed whitespace-pre-line">
              {doc.content}
            </div>
          </div>
        )}
      </div>
    </>
  )
}
