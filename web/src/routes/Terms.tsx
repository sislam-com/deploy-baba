import { useEffect, useState } from 'react'
import { Helmet } from 'react-helmet-async'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

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
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              components={{
                h1: ({ children }) => <h1 className="text-xl font-bold text-white mt-6 mb-2">{children}</h1>,
                h2: ({ children }) => <h2 className="text-lg font-semibold text-cyan-400 mt-6 mb-2">{children}</h2>,
                h3: ({ children }) => <h3 className="text-base font-medium text-gray-100 mt-4 mb-1">{children}</h3>,
                p: ({ children }) => <p className="text-gray-300 leading-relaxed my-2">{children}</p>,
                ul: ({ children }) => <ul className="list-disc list-inside my-2 space-y-1 text-gray-300">{children}</ul>,
                ol: ({ children }) => <ol className="list-decimal list-inside my-2 space-y-1 text-gray-300">{children}</ol>,
                li: ({ children }) => <li className="leading-relaxed">{children}</li>,
                a: ({ href, children }) => <a href={href} className="text-cyan-400 underline hover:text-cyan-300">{children}</a>,
                strong: ({ children }) => <strong className="text-gray-100 font-semibold">{children}</strong>,
                blockquote: ({ children }) => <blockquote className="border-l-2 border-cyan-500 pl-3 text-gray-400 my-2 italic">{children}</blockquote>,
              }}
            >
              {doc.content}
            </ReactMarkdown>
          </div>
        )}
      </div>
    </>
  )
}
