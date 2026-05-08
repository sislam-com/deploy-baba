import { useState, useRef, FormEvent, useEffect } from 'react'
import { Helmet } from 'react-helmet-async'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

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

const KIND_ICON: Record<string, string> = {
  rust: '🦀',
  hcl: '🏗',
  plan: '📋',
  cache: '💾',
  portfolio: '💼',
}

const RECRUITER_QUESTIONS = [
  { value: '', label: 'Select a question...' },
  { value: 'What are your primary skills and technical expertise?', label: 'What are your primary skills?' },
  { value: 'Tell me about your experience with AI/LLM systems and RAG pipelines', label: 'AI/LLM systems experience' },
  { value: 'What is your experience with cloud infrastructure and AWS?', label: 'Cloud infrastructure experience' },
  { value: 'Describe your technical leadership and team management experience', label: 'Technical leadership experience' },
  { value: 'What platforms and products have you built end-to-end?', label: 'Products built end-to-end' },
  { value: 'How does the RAG pipeline in this portfolio project work?', label: 'How the RAG pipeline works' },
  { value: 'What are the key architecture decisions in this portfolio?', label: 'Key architecture decisions' },
  { value: 'Why was SQLite chosen over PostgreSQL for this project?', label: 'SQLite vs PostgreSQL decision' },
  { value: 'How is authentication implemented in this portfolio?', label: 'Authentication implementation' },
]

function CitationBadge({ index, path, kind, url }: { index: number; path: string; kind: string; url: string }) {
  const isPortfolio = kind === 'portfolio'
  const displayPath = isPortfolio ? path.replace('portfolio://', '').replace('/', ' → ') : path

  return (
    <li className="flex items-start gap-2 text-xs text-gray-500">
      <span className="text-gray-600 font-mono mt-0.5">[{index + 1}]</span>
      <span className="flex-1">
        {KIND_ICON[kind] ?? '📄'}{' '}
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

export default function Ask() {
  const [query, setQuery] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<AskResult | null>(null)
  const [selectedQuestion, setSelectedQuestion] = useState('')
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // Auto-trigger the most pertinent question on first load
  useEffect(() => {
    if (!result && !loading && !error) {
      const defaultQuestion = 'What are your primary skills and technical expertise?'
      setQuery(defaultQuestion)
      setSelectedQuestion(defaultQuestion)
      // Auto-submit after a short delay
      setTimeout(() => {
        const form = document.getElementById('ask-form') as HTMLFormElement
        if (form) form.requestSubmit()
      }, 500)
    }
  }, [])

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

  function handleQuestionChange(e: React.ChangeEvent<HTMLSelectElement>) {
    const selected = e.target.value
    setSelectedQuestion(selected)
    setQuery(selected)
    textareaRef.current?.focus()
  }

  return (
    <>
      <Helmet>
        <title>Ask — Sharful Islam</title>
      </Helmet>

      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white mb-2">Ask</h1>
          <p className="text-gray-400 text-sm">
            Questions about this portfolio and the codebase are answered with source citations.
          </p>
        </div>

        <form id="ask-form" onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="question-select" className="block text-sm font-medium text-gray-300 mb-2">
              Common questions
            </label>
            <select
              id="question-select"
              value={selectedQuestion}
              onChange={handleQuestionChange}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white
                         focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:border-transparent transition"
            >
              {RECRUITER_QUESTIONS.map(q => (
                <option key={q.value} value={q.value}>
                  {q.label}
                </option>
              ))}
            </select>
          </div>

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
              placeholder="Or type your own question..."
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
              {loading ? 'Asking...' : 'Ask'}
            </button>
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
      </div>
    </>
  )
}
