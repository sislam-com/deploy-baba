import { useState, useCallback, useRef } from 'react'

export type AgentName = 'preground' | 'cover_letter_writer' | 'pdf_uploader' | 'link_generator'
export type AgentStatus = 'pending' | 'running' | 'completed' | 'failed'

export interface AgentState {
  name: AgentName
  label: string
  description: string
  status: AgentStatus
  detail?: string
}

export interface WorkflowResult {
  download_url: string
  preview_html: string
  summary: string
}

const INITIAL_AGENTS: AgentState[] = [
  {
    name: 'preground',
    label: 'Context Loader',
    description: 'Fetch resume data and match keywords locally',
    status: 'pending',
  },
  {
    name: 'cover_letter_writer',
    label: 'Cover Letter Writer',
    description: 'Generate tailored cover letter with grounded context',
    status: 'pending',
  },
  {
    name: 'pdf_uploader',
    label: 'PDF Converter & Uploader',
    description: 'Convert to PDF and upload to S3',
    status: 'pending',
  },
  {
    name: 'link_generator',
    label: 'Link Generator',
    description: 'Generate secure download link',
    status: 'pending',
  },
]

interface SSEEvent {
  event: string
  data: string
}

function parseSSELines(text: string): SSEEvent[] {
  const events: SSEEvent[] = []
  const blocks = text.split('\n\n')
  for (const block of blocks) {
    if (!block.trim()) continue
    let event = ''
    let data = ''
    for (const line of block.split('\n')) {
      if (line.startsWith('event: ')) event = line.slice(7)
      else if (line.startsWith('data: ')) data = line.slice(6)
    }
    if (event && data) events.push({ event, data })
  }
  return events
}

export function useAgentStream() {
  const [agents, setAgents] = useState<AgentState[]>(INITIAL_AGENTS)
  const [result, setResult] = useState<WorkflowResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isStreaming, setIsStreaming] = useState(false)
  const abortRef = useRef<AbortController | null>(null)

  const reset = useCallback(() => {
    setAgents(INITIAL_AGENTS)
    setResult(null)
    setError(null)
  }, [])

  const generate = useCallback(async (jobDescription: string) => {
    if (abortRef.current) abortRef.current.abort()
    const controller = new AbortController()
    abortRef.current = controller

    reset()
    setIsStreaming(true)

    try {
      const res = await fetch('/api/v1/agent/cover-letter/stream', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ job_description: jobDescription }),
        signal: controller.signal,
      })

      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.detail ?? `HTTP ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('No response body')

      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lastDoubleNewline = buffer.lastIndexOf('\n\n')
        if (lastDoubleNewline === -1) continue
        const complete = buffer.slice(0, lastDoubleNewline + 2)
        buffer = buffer.slice(lastDoubleNewline + 2)
        const events = parseSSELines(complete)

        for (const evt of events) {
          if (evt.event === 'agent') {
            const payload = JSON.parse(evt.data)
            setAgents(prev =>
              prev.map(a =>
                a.name === payload.agent
                  ? { ...a, status: payload.status, detail: payload.detail }
                  : a
              )
            )
          } else if (evt.event === 'result') {
            setResult(JSON.parse(evt.data))
          } else if (evt.event === 'error') {
            const payload = JSON.parse(evt.data)
            setError(payload.message)
          }
        }
      }
    } catch (err) {
      if (err instanceof Error && err.name !== 'AbortError') {
        setError(err.message)
      }
    } finally {
      setIsStreaming(false)
      abortRef.current = null
    }
  }, [reset])

  const cancel = useCallback(() => {
    if (abortRef.current) abortRef.current.abort()
  }, [])

  return { agents, result, error, isStreaming, generate, cancel, reset }
}
