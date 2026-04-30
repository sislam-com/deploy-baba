import { useState, useRef, FormEvent } from 'react'
import { Helmet } from 'react-helmet-async'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

interface Citation {
  path: string
  kind: string
  sha: string
}

interface AskResult {
  answer: string
  citations: Citation[]
  model: string
  input_tokens: number
  output_tokens: number
}

const KIND_ICON: Record<string, string> = {
  rust: '🦀',
  hcl: '🏗',
  plan: '📋',
  cache: '💾',
}

const EXAMPLES = [
  'Why SQLite instead of PostgreSQL?',
  'How does Lambda load secrets at cold start?',
  'How does the PoW challenge protect the contact form?',
  'What is the RAG pipeline and how does it work?',
  'How is Cognito authentication implemented?',
  'What are the ADRs for infrastructure decisions?',
]

function CitationBadge({ index, path, kind, sha }: { index: number; path: string; kind: string; sha: string }) {
  return (
    <li className="flex items-start gap-2 text-xs text-gray-500">
      <span className="text-gray-600 font-mono mt-0.5">[{index + 1}]</span>
      <span>
        {KIND_ICON[kind] ?? '📄'}{' '}
        <span className="font-mono text-gray-400">{path}</span>
        <span className="text-gray-600 ml-1">sha:{sha.slice(0, 7)}</span>
      </span>
    </li>
  )
}

export default function Ask() {
  const [query, setQuery] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<AskResult | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    const q = query.trim()
    if (!q) return

    setLoading(true)
    setError(null)
    setResult(null)

    try {
      const res = await fetch('/api/ask', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query: q, top_k: 10 }),
      })

      const data = await res.json()

      if (!res.ok) {
        if (res.status === 429) {
          setError('Rate limit reached — please wait a minute and try again.')
        } else if (res.status === 503) {
          setError('The Q&A feature is not available right now.')
        } else {
          setError(data.error ?? `Error ${res.status}`)
        }
        return
      }

      setResult(data as AskResult)
    } catch {
      setError('Network error — please check your connection and try again.')
    } finally {
      setLoading(false)
    }
  }

  function fillQuery(text: string) {
    setQuery(text)
    textareaRef.current?.focus()
  }

  return (
    <>
      <Helmet>
        <title>Ask — Sharful Islam</title>
      </Helmet>

      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="mb-10">
          <h1 className="text-4xl font-bold text-white mb-2">Ask the Codebase</h1>
          <p className="text-gray-400">
            Ask anything about this portfolio project — architecture decisions, how features
            work, why something was built a certain way. Answers are grounded in the actual
            source code, infrastructure, and design documents.
          </p>
          <p className="text-gray-600 text-sm mt-2">
            Powered by Claude + SQLite FTS5 retrieval over Rust source, OpenTofu HCL, and ADRs.
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="query" className="block text-sm font-medium text-gray-300 mb-1">
              Your question
            </label>
            <textarea
              ref={textareaRef}
              id="query"
              rows={3}
              required
              maxLength={1000}
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder="e.g. How does Lambda load secrets at cold start? Why SQLite instead of PostgreSQL?"
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white
                         placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500
                         focus:border-transparent transition resize-none"
            />
          </div>

          <div className="flex items-center gap-4">
            <button
              type="submit"
              disabled={loading}
              className="bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 disabled:cursor-not-allowed
                         text-white font-semibold py-2.5 px-6 rounded-lg transition focus:outline-none
                         focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-gray-900"
            >
              Ask
            </button>
            {loading && (
              <span className="text-sm text-gray-500">
                Retrieving sources and generating answer…
              </span>
            )}
          </div>
        </form>

        {error && (
          <div className="mt-6 px-4 py-3 rounded-lg text-sm font-medium bg-red-900/60 border border-red-700 text-red-300">
            {error}
          </div>
        )}

        {result && (
          <div className="mt-8 space-y-6">
            <div className="bg-gray-800/60 border border-gray-700 rounded-xl p-6">
              <div className="flex items-center gap-2 mb-4">
                <span className="text-xs font-semibold uppercase tracking-wider text-cyan-400">Answer</span>
                <span className="text-xs text-gray-600">· {result.model}</span>
              </div>
              <div className="ask-prose text-sm">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  components={{
                    p: ({ children }) => <p className="my-2 leading-relaxed text-gray-200">{children}</p>,
                    h1: ({ children }) => <h1 className="text-xl font-bold text-white mt-5 mb-2">{children}</h1>,
                    h2: ({ children }) => <h2 className="text-lg font-semibold text-gray-100 mt-4 mb-2">{children}</h2>,
                    h3: ({ children }) => <h3 className="text-base font-semibold text-gray-200 mt-3 mb-1">{children}</h3>,
                    ul: ({ children }) => <ul className="list-disc list-inside my-2 space-y-1 text-gray-300">{children}</ul>,
                    ol: ({ children }) => <ol className="list-decimal list-inside my-2 space-y-1 text-gray-300">{children}</ol>,
                    li: ({ children }) => <li className="leading-relaxed">{children}</li>,
                    code: ({ className, children, ...rest }) => {
                      const isBlock = className?.startsWith('language-')
                      return isBlock ? (
                        <code className="block bg-gray-900 text-green-300 text-xs p-4 rounded-lg overflow-x-auto font-mono my-3" {...rest}>
                          {children}
                        </code>
                      ) : (
                        <code className="bg-gray-700 text-cyan-300 text-xs px-1.5 py-0.5 rounded font-mono" {...rest}>
                          {children}
                        </code>
                      )
                    },
                    pre: ({ children }) => <pre className="bg-gray-900 border border-gray-700 rounded-lg overflow-x-auto my-3">{children}</pre>,
                    blockquote: ({ children }) => (
                      <blockquote className="border-l-2 border-cyan-500 pl-4 text-gray-400 my-3 italic">{children}</blockquote>
                    ),
                    strong: ({ children }) => <strong className="text-gray-100 font-semibold">{children}</strong>,
                    a: ({ href, children }) => (
                      <a href={href} className="text-cyan-400 underline hover:text-cyan-300">{children}</a>
                    ),
                  }}
                >
                  {result.answer.replace(/\[source (\d+)\]/g, '**[source $1]**')}
                </ReactMarkdown>
              </div>
            </div>

            {result.citations.length > 0 && (
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-3">Sources</h3>
                <ul className="space-y-2">
                  {result.citations.map((c, i) => (
                    <CitationBadge key={i} index={i} {...c} />
                  ))}
                </ul>
              </div>
            )}

            <p className="text-xs text-gray-600">
              {result.input_tokens} in · {result.output_tokens} out
            </p>
          </div>
        )}

        <div className="mt-12">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-gray-600 mb-4">Try asking</h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {EXAMPLES.map(ex => (
              <button
                key={ex}
                onClick={() => fillQuery(ex)}
                className="text-left bg-gray-800/40 border border-gray-700 hover:border-gray-500
                           rounded-lg px-4 py-3 text-sm text-gray-400 hover:text-gray-200 transition"
              >
                {ex}
              </button>
            ))}
          </div>
        </div>
      </div>
    </>
  )
}
