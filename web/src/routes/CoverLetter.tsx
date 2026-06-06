import { useState, useEffect, FormEvent } from 'react'
import { useLocation } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'
import { useAgentStream, type AgentState, type AgentStatus } from '../hooks/useAgentStream'

function StatusIcon({ status }: { status: AgentStatus }) {
  if (status === 'pending') {
    return (
      <div className="w-8 h-8 rounded-full border-2 border-gray-600 flex items-center justify-center shrink-0">
        <div className="w-2 h-2 rounded-full bg-gray-600" />
      </div>
    )
  }
  if (status === 'running') {
    return (
      <div className="w-8 h-8 rounded-full border-2 border-cyan-400 flex items-center justify-center shrink-0 animate-pulse">
        <div className="w-3 h-3 rounded-full border-2 border-cyan-400 border-t-transparent animate-spin" />
      </div>
    )
  }
  if (status === 'completed') {
    return (
      <div className="w-8 h-8 rounded-full bg-emerald-500/20 border-2 border-emerald-500 flex items-center justify-center shrink-0">
        <svg className="w-4 h-4 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
        </svg>
      </div>
    )
  }
  return (
    <div className="w-8 h-8 rounded-full bg-red-500/20 border-2 border-red-500 flex items-center justify-center shrink-0">
      <svg className="w-4 h-4 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </div>
  )
}

function ConnectorLine({ fromStatus, toStatus }: { fromStatus: AgentStatus; toStatus: AgentStatus }) {
  const isActive = fromStatus === 'completed'
  const isFlowing = fromStatus === 'completed' && toStatus === 'running'
  return (
    <div className="flex justify-center py-1">
      <div
        className={`w-0.5 h-6 transition-colors duration-500 ${
          isFlowing
            ? 'bg-gradient-to-b from-emerald-500 to-cyan-400 animate-pulse'
            : isActive
              ? 'bg-emerald-500/50'
              : 'bg-gray-700'
        }`}
      />
    </div>
  )
}

function AgentCard({ agent }: { agent: AgentState }) {
  const borderColor = {
    pending: 'border-gray-700',
    running: 'border-cyan-500',
    completed: 'border-emerald-500/50',
    failed: 'border-red-500/50',
  }[agent.status]

  const bgColor = {
    pending: 'bg-gray-800/30',
    running: 'bg-gray-800/60',
    completed: 'bg-gray-800/40',
    failed: 'bg-red-900/20',
  }[agent.status]

  return (
    <div className={`rounded-lg border ${borderColor} ${bgColor} p-4 transition-all duration-500`}>
      <div className="flex items-start gap-3">
        <StatusIcon status={agent.status} />
        <div className="flex-1 min-w-0">
          <h3 className={`font-semibold text-sm ${agent.status === 'pending' ? 'text-gray-500' : 'text-white'}`}>
            {agent.label}
          </h3>
          <p className="text-xs text-gray-500 mt-0.5">{agent.description}</p>
          {agent.detail && agent.status !== 'pending' && (
            <p className={`text-xs mt-1.5 ${
              agent.status === 'failed' ? 'text-red-400' : 'text-cyan-400'
            }`}>
              {agent.detail}
            </p>
          )}
        </div>
      </div>
    </div>
  )
}

export default function CoverLetter() {
  const location = useLocation()
  const prefilledJd = (location.state as { jobDescription?: string } | null)?.jobDescription ?? ''
  const [jobDescription, setJobDescription] = useState(prefilledJd)
  const { agents, result, error, isStreaming, generate, cancel } = useAgentStream()

  useEffect(() => {
    if (prefilledJd && prefilledJd.trim().length >= 50) {
      generate(prefilledJd)
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  function handleSubmit(e: FormEvent) {
    e.preventDefault()
    const jd = jobDescription.trim()
    if (jd.length < 50) return
    generate(jd)
  }

  return (
    <>
      <Helmet>
        <title>AI Cover Letter Generator — Portfolio</title>
      </Helmet>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <div className="mb-6">
          <h1 className="text-2xl font-bold text-white">AI Cover Letter Generator</h1>
          <p className="text-gray-400 text-sm mt-1">
            Paste a job description and watch three AI agents collaborate to create a tailored cover letter.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Left — Input */}
          <div className="space-y-4">
            <form onSubmit={handleSubmit} className="space-y-3">
              <div>
                <label htmlFor="jd" className="block text-xs font-medium text-gray-300 mb-1">
                  Job Description
                </label>
                <textarea
                  id="jd"
                  rows={12}
                  required
                  minLength={50}
                  maxLength={10000}
                  value={jobDescription}
                  onChange={e => setJobDescription(e.target.value)}
                  disabled={isStreaming}
                  placeholder="Paste the full job description here (minimum 50 characters)..."
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm
                             placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500
                             focus:border-transparent transition resize-none disabled:opacity-50"
                />
                <p className="text-xs text-gray-600 mt-1">{jobDescription.length} / 10,000 characters</p>
              </div>

              <div className="flex gap-2">
                <button
                  type="submit"
                  disabled={isStreaming || jobDescription.trim().length < 50}
                  className="flex-1 bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 disabled:cursor-not-allowed
                             text-white font-semibold py-2.5 px-4 rounded-lg transition focus:outline-none
                             focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-gray-900 text-sm"
                >
                  {isStreaming ? 'Generating...' : 'Generate Cover Letter'}
                </button>
                {isStreaming && (
                  <button
                    type="button"
                    onClick={cancel}
                    className="px-4 py-2.5 rounded-lg border border-gray-600 text-gray-400 hover:text-white
                               hover:border-gray-500 transition text-sm"
                  >
                    Cancel
                  </button>
                )}
              </div>
            </form>

            {error && (
              <div className="px-3 py-2 rounded-lg text-xs font-medium bg-red-900/60 border border-red-700 text-red-300">
                {error}
              </div>
            )}

            {/* Result preview */}
            {result && (
              <div className="space-y-3">
                <div className="bg-gray-800/60 border border-gray-700 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-xs font-semibold uppercase tracking-wider text-cyan-400">Cover Letter Preview</span>
                    <a
                      href={result.download_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500
                                 text-white text-xs font-medium transition"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                      </svg>
                      Download PDF
                    </a>
                  </div>
                  <div
                    className="prose prose-invert prose-sm max-h-[40vh] overflow-y-auto text-gray-200"
                    dangerouslySetInnerHTML={{ __html: result.preview_html }}
                  />
                </div>
                <p className="text-xs text-gray-600">{result.summary}</p>
              </div>
            )}
          </div>

          {/* Right — Agent Workflow Visualizer */}
          <div>
            <span className="block text-xs font-medium text-gray-400 mb-3 uppercase tracking-wider">Agent Workflow</span>
            <div className="space-y-0">
              {agents.map((agent, i) => (
                <div key={agent.name}>
                  <AgentCard agent={agent} />
                  {i < agents.length - 1 && (
                    <ConnectorLine fromStatus={agents[i].status} toStatus={agents[i + 1].status} />
                  )}
                </div>
              ))}
            </div>

            {!isStreaming && !result && (
              <div className="mt-6 bg-gray-800/20 border border-gray-700/50 rounded-lg p-6 text-center">
                <div className="w-10 h-10 rounded-full bg-gray-800 border border-gray-700 flex items-center justify-center mx-auto mb-3">
                  <svg className="w-5 h-5 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9.75 3.104v5.714a2.25 2.25 0 01-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 014.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.393A9.065 9.065 0 0112 15a9.065 9.065 0 00-6.23.693L5 14.5m14.8.8l1.402 1.402c1.232 1.232.65 3.318-1.067 3.611A48.309 48.309 0 0112 21c-2.773 0-5.491-.235-8.135-.687-1.718-.293-2.3-2.379-1.067-3.61L5 14.5" />
                  </svg>
                </div>
                <p className="text-gray-400 text-sm font-medium mb-1">Multi-Agent Workflow</p>
                <p className="text-gray-500 text-xs max-w-xs mx-auto">
                  Three AI agents work in sequence: generate content, convert to PDF, and create a download link.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
