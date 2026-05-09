import { useState, useRef, FormEvent } from 'react'
import { Helmet } from 'react-helmet-async'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import SvgIcon from '../components/SvgIcon'

interface Citation {
  path: string
  kind: string
  sha: string
  url: string
  ord: number
}

interface AskResult {
  answer: string
  citations: Citation[]
  model: string
  input_tokens: number
  output_tokens: number
}

const KIND_ICON_MAP: Record<string, string> = {
  rust: 'cpu',
  hcl: 'server',
  plan: 'clipboard',
  cache: 'database',
  portfolio: 'briefcase',
}

const RECRUITER_QUESTIONS = [
  { value: 'Paste a job description below, and I\'ll explain how my experience aligns with the role.', label: 'Match to a role', featured: true },
  { value: 'What are your primary skills and technical expertise?', label: 'Primary skills' },
  { value: 'Tell me about your experience with AI/LLM systems and RAG pipelines', label: 'AI/LLM experience' },
  { value: 'What is your experience with cloud infrastructure and AWS?', label: 'Cloud & AWS' },
  { value: 'Describe your technical leadership and team management experience', label: 'Technical leadership' },
  { value: 'What platforms and products have you built end-to-end?', label: 'Products built' },
  { value: 'How does the RAG pipeline in this portfolio project work?', label: 'RAG pipeline' },
  { value: 'What are the key architecture decisions in this portfolio?', label: 'Architecture decisions' },
  { value: 'Why was SQLite chosen over PostgreSQL for this project?', label: 'SQLite vs PostgreSQL' },
  { value: 'How is authentication implemented in this portfolio?', label: 'Auth implementation' },
]

function CitationBadge({ index, path, kind, url }: { index: number; path: string; kind: string; url: string }) {
  const isPortfolio = kind === 'portfolio'
  const displayPath = isPortfolio ? path.replace('portfolio://', '').replace('/', ' → ') : path

  return (
    <li className="flex items-start gap-2 text-xs text-gray-500">
      <span className="text-gray-600 font-mono mt-0.5">[{index + 1}]</span>
      <span className="flex-1 flex items-center gap-1.5">
        <SvgIcon name={KIND_ICON_MAP[kind] ?? 'document'} className="w-3.5 h-3.5 text-gray-500 shrink-0" />
        {isPortfolio ? (
          <a href={url} className="font-mono text-cyan-400 hover:text-cyan-300 hover:underline">
            {displayPath}
          </a>
        ) : (
          <a href={url} target="_blank" rel="noopener noreferrer" className="font-mono text-cyan-400 hover:text-cyan-300 hover:underline">
            {path}
          </a>
        )}
      </span>
    </li>
  )
}

export default function Ask({ embedded = false }: { embedded?: boolean }) {
  const [query, setQuery] = useState(RECRUITER_QUESTIONS[0].value)
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

  const content = (
    <div className={`${embedded ? 'max-w-5xl' : 'max-w-7xl'} mx-auto px-4 sm:px-6 lg:px-8 ${embedded ? '' : 'py-6'}`}>
      {!embedded && (
        <div className="mb-4">
          <h1 className="text-2xl font-bold text-white">Ask</h1>
          <p className="text-gray-400 text-xs sm:text-sm">
            Questions about this portfolio and the codebase are answered with source citations.
          </p>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6">
        {/* Left column - Form */}
        <div className="space-y-3">
          {/* Suggested question pills */}
          <div>
            <span className="block text-xs font-medium text-gray-400 mb-2">Common questions</span>
            <div className="flex flex-wrap gap-2">
              {RECRUITER_QUESTIONS.map((q, i) => {
                const featured = 'featured' in q && q.featured
                return (
                  <button
                    key={q.value}
                    type="button"
                    onClick={() => { setQuery(q.value); textareaRef.current?.focus() }}
                    className={`text-xs px-3 py-1.5 rounded-full border transition cursor-pointer animate-fadeIn ${
                      query === q.value
                        ? 'border-cyan-500 text-cyan-400 bg-cyan-600/10'
                        : featured
                          ? 'border-cyan-600 text-cyan-400 hover:bg-cyan-600/10 bg-gray-800/50'
                          : 'border-gray-700 text-gray-400 hover:text-cyan-400 hover:border-cyan-500/50 bg-gray-800/50'
                    }`}
                    style={{ animationDelay: `${i * 40}ms` }}
                  >
                    {q.label}
                  </button>
                )
              })}
            </div>
          </div>

          <form id="ask-form" onSubmit={handleSubmit} className="space-y-3">
            <div>
              <label htmlFor="query" className="block text-xs font-medium text-gray-300 mb-1">
                Your question
              </label>
              <textarea
                ref={textareaRef}
                id="query"
                rows={4}
                required
                maxLength={6000}
                value={query}
                onChange={e => setQuery(e.target.value)}
                placeholder="Or type your own question..."
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm
                           placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500
                           focus:border-transparent transition resize-none"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 disabled:cursor-not-allowed
                         text-white font-semibold py-2 px-4 rounded-lg transition focus:outline-none
                         focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-gray-900 text-sm"
            >
              {loading ? 'Asking...' : 'Ask'}
            </button>
          </form>

          {error && (
            <div className="px-3 py-2 rounded-lg text-xs font-medium bg-red-900/60 border border-red-700 text-red-300">
              {error}
            </div>
          )}
        </div>

        {/* Right column - Response */}
        <div className="lg:max-h-[70vh] lg:overflow-y-auto">
          {result && (
            <div className="space-y-3">
              <div className="bg-gray-800/60 border border-gray-700 rounded-lg p-4">
                <div className="flex items-center gap-2 mb-3">
                  <span className="text-xs font-semibold uppercase tracking-wider text-cyan-400">Answer</span>
                  <span className="text-xs text-gray-600">&middot; {result.model}</span>
                </div>
                <div className="ask-prose text-xs sm:text-sm max-h-[40vh] overflow-y-auto">
                  <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    components={{
                      p: ({ children }) => <p className="my-1.5 leading-relaxed text-gray-200">{children}</p>,
                      h1: ({ children }) => <h1 className="text-lg font-bold text-white mt-3 mb-1.5">{children}</h1>,
                      h2: ({ children }) => <h2 className="text-base font-semibold text-gray-100 mt-2.5 mb-1">{children}</h2>,
                      h3: ({ children }) => <h3 className="text-sm font-semibold text-gray-200 mt-2 mb-0.5">{children}</h3>,
                      ul: ({ children }) => <ul className="list-disc list-inside my-1.5 space-y-0.5 text-gray-300">{children}</ul>,
                      ol: ({ children }) => <ol className="list-decimal list-inside my-1.5 space-y-0.5 text-gray-300">{children}</ol>,
                      li: ({ children }) => <li className="leading-relaxed">{children}</li>,
                      code: ({ className, children, ...rest }) => {
                        const isBlock = className?.startsWith('language-')
                        return isBlock ? (
                          <code className="block bg-gray-900 text-green-300 text-xs p-3 rounded-lg overflow-x-auto font-mono my-2" {...rest}>
                            {children}
                          </code>
                        ) : (
                          <code className="bg-gray-700 text-cyan-300 text-xs px-1 py-0.5 rounded font-mono" {...rest}>
                            {children}
                          </code>
                        )
                      },
                      pre: ({ children }) => <pre className="bg-gray-900 border border-gray-700 rounded-lg overflow-x-auto my-2">{children}</pre>,
                      blockquote: ({ children }) => (
                        <blockquote className="border-l-2 border-cyan-500 pl-3 text-gray-400 my-2 italic">{children}</blockquote>
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
                <div className="bg-gray-800/40 border border-gray-700 rounded-lg p-3">
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-2">Sources</h3>
                  <ul className="space-y-1.5">
                    {result.citations.map((c, i) => (
                      <CitationBadge key={i} index={i} {...c} />
                    ))}
                  </ul>
                </div>
              )}

              <p className="text-xs text-gray-600">
                {result.input_tokens} in &middot; {result.output_tokens} out
              </p>
            </div>
          )}

          {!result && !loading && (
            <div className="bg-gray-800/30 border border-gray-700 rounded-lg p-8 text-center">
              <SvgIcon name="chat" className="w-8 h-8 text-gray-600 mx-auto mb-3" />
              <p className="text-gray-400 text-sm font-medium mb-1">AI-Powered Q&A</p>
              <p className="text-gray-500 text-xs max-w-xs mx-auto">
                Ask about skills, experience, architecture decisions, or anything in the codebase. Answers include source citations.
              </p>
            </div>
          )}

          {loading && (
            <div className="bg-gray-800/30 border border-gray-700 rounded-lg p-6 text-center">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-cyan-400 mx-auto mb-3" />
              <p className="text-gray-500 text-sm">Retrieving sources and generating answer…</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )

  if (embedded) return content

  return (
    <>
      <Helmet>
        <title>Ask — Portfolio</title>
      </Helmet>
      <div className="min-h-screen bg-gray-900">
        {content}
      </div>
    </>
  )
}
