import { useEffect, useState, useRef } from 'react'
import { Link, useSearchParams } from 'react-router-dom'

interface LinkedInConnectionStatus {
  connected: boolean
  name: string | null
  email: string | null
  picture_url: string | null
  token_expires_at: string | null
}

interface LinkedInPosition {
  id: number
  company: string
  title: string
  location: string | null
  start_date: string
  end_date: string | null
  sync_status: string
  mapped_job_id: number | null
  imported_at: string
}

interface LinkedInProject {
  id: number
  title: string
  description: string | null
  url: string | null
  start_date: string | null
  end_date: string | null
  associated_position: string | null
  sync_status: string
  mapped_challenge_id: number | null
  imported_at: string
}

interface SyncLogEntry {
  id: number
  source: string
  positions_count: number
  projects_count: number
  imported_at: string
}

interface ImportResult {
  positions_imported: number
  projects_imported: number
  positions_matched: number
  projects_matched: number
}

interface ReconciliationItem {
  id: number
  entity_type: string
  title: string
  sync_status: string
  has_mapping: boolean
  differing_fields: string[]
}

interface ReconciliationSummary {
  needs_linkedin_update: ReconciliationItem[]
  needs_db_import: ReconciliationItem[]
  in_sync: ReconciliationItem[]
}

const STATUS_COLORS: Record<string, string> = {
  synced: 'bg-green-900 text-green-300',
  diverged: 'bg-yellow-900 text-yellow-300',
  linkedin_only: 'bg-blue-900 text-blue-300',
  local_only: 'bg-gray-700 text-gray-300',
  unreviewed: 'bg-purple-900 text-purple-300',
}

function StatusBadge({ status }: { status: string }) {
  const colors = STATUS_COLORS[status] ?? 'bg-gray-700 text-gray-400'
  return (
    <span className={`text-[10px] font-semibold px-1.5 py-0.5 rounded ${colors}`}>
      {status.replace('_', ' ')}
    </span>
  )
}

export default function LinkedInSync() {
  const [searchParams] = useSearchParams()
  const [tab, setTab] = useState<'positions' | 'projects'>('positions')
  const [positions, setPositions] = useState<LinkedInPosition[]>([])
  const [projects, setProjects] = useState<LinkedInProject[]>([])
  const [syncLog, setSyncLog] = useState<SyncLogEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [importing, setImporting] = useState(false)
  const [importResult, setImportResult] = useState<ImportResult | null>(null)
  const [jsonInput, setJsonInput] = useState('')
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [connectionStatus, setConnectionStatus] = useState<LinkedInConnectionStatus | null>(null)
  const [connecting, setConnecting] = useState(false)
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set())
  const [reconciliation, setReconciliation] = useState<ReconciliationSummary | null>(null)

  useEffect(() => {
    fetch('/api/v1/agent/linkedin/status')
      .then(r => r.json())
      .then(setConnectionStatus)
      .catch(() => setConnectionStatus({ connected: false, name: null, email: null, picture_url: null, token_expires_at: null }))
  }, [searchParams.get('connected')])

  const handleConnect = async () => {
    setConnecting(true)
    try {
      const res = await fetch('/api/v1/agent/linkedin/auth-url')
      if (!res.ok) throw new Error(await res.text())
      const data = await res.json()
      window.location.href = data.url
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to start LinkedIn auth')
      setConnecting(false)
    }
  }

  const handleDisconnect = async () => {
    await fetch('/api/v1/agent/linkedin/disconnect', { method: 'POST' })
    setConnectionStatus({ connected: false, name: null, email: null, picture_url: null, token_expires_at: null })
  }

  const fetchData = () => {
    setLoading(true)
    Promise.all([
      fetch('/api/v1/admin/linkedin/positions').then(r => r.json()),
      fetch('/api/v1/admin/linkedin/projects').then(r => r.json()),
      fetch('/api/v1/admin/linkedin/sync-log').then(r => r.json()),
      fetch('/api/v1/admin/linkedin/reconciliation').then(r => r.json()),
    ])
      .then(([pos, proj, log, recon]) => {
        setPositions(Array.isArray(pos) ? pos : [])
        setProjects(Array.isArray(proj) ? proj : [])
        setSyncLog(Array.isArray(log) ? log : [])
        setReconciliation(recon ?? null)
      })
      .catch(() => setError('Failed to load LinkedIn data'))
      .finally(() => setLoading(false))
  }

  useEffect(fetchData, [])

  const handleImportJson = async () => {
    if (!jsonInput.trim()) return
    setImporting(true)
    setImportResult(null)
    setError(null)
    try {
      const payload = JSON.parse(jsonInput)
      const res = await fetch('/api/v1/admin/linkedin/import', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!res.ok) throw new Error(await res.text())
      const result: ImportResult = await res.json()
      setImportResult(result)
      setJsonInput('')
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Import failed')
    } finally {
      setImporting(false)
    }
  }

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    setImporting(true)
    setImportResult(null)
    setError(null)
    try {
      const text = await file.text()
      const payload = parseCsv(text, file.name)
      const res = await fetch('/api/v1/admin/linkedin/import', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!res.ok) throw new Error(await res.text())
      const result: ImportResult = await res.json()
      setImportResult(result)
      fetchData()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'CSV import failed')
    } finally {
      setImporting(false)
      if (fileInputRef.current) fileInputRef.current.value = ''
    }
  }

  const handleBulkStatus = async (status: string) => {
    if (selectedIds.size === 0) return
    const endpoint = tab === 'positions'
      ? '/api/v1/admin/linkedin/positions/bulk-status'
      : '/api/v1/admin/linkedin/projects/bulk-status'
    const res = await fetch(endpoint, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ids: [...selectedIds], status }),
    })
    if (res.ok) {
      setSelectedIds(new Set())
      fetchData()
    }
  }

  const handleAutoMatch = async () => {
    const res = await fetch('/api/v1/admin/linkedin/auto-match', { method: 'POST' })
    if (res.ok) {
      const result = await res.json()
      setError(null)
      setImportResult({
        positions_imported: 0,
        projects_imported: 0,
        positions_matched: result.positions_matched,
        projects_matched: result.projects_matched,
      })
      fetchData()
    }
  }

  const toggleSelection = (id: number) => {
    setSelectedIds(prev => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const toggleAll = () => {
    const items = tab === 'positions' ? positions : projects
    if (selectedIds.size === items.length) {
      setSelectedIds(new Set())
    } else {
      setSelectedIds(new Set(items.map(i => i.id)))
    }
  }

  // Clear selection when switching tabs
  useEffect(() => { setSelectedIds(new Set()) }, [tab])

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold text-white mb-6">LinkedIn Sync</h1>

      {/* Connection Status */}
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-5 mb-6">
        <h2 className="text-sm font-semibold text-gray-300 mb-3">LinkedIn Connection</h2>
        {connectionStatus?.connected ? (
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              {connectionStatus.picture_url && (
                <img
                  src={connectionStatus.picture_url}
                  alt=""
                  className="w-10 h-10 rounded-full"
                />
              )}
              <div>
                <p className="text-white font-medium">
                  {connectionStatus.name ?? 'Connected'}
                </p>
                {connectionStatus.email && (
                  <p className="text-sm text-gray-400">{connectionStatus.email}</p>
                )}
              </div>
              <span className="bg-green-900 text-green-300 text-[10px] font-semibold px-2 py-0.5 rounded ml-2">
                Connected
              </span>
            </div>
            <button
              onClick={handleDisconnect}
              className="text-sm text-gray-400 hover:text-red-400 transition"
            >
              Disconnect
            </button>
          </div>
        ) : (
          <div className="flex items-center gap-4">
            <button
              onClick={handleConnect}
              disabled={connecting}
              className="bg-[#0A66C2] hover:bg-[#004182] disabled:bg-gray-600 text-white text-sm font-semibold px-5 py-2.5 rounded-lg transition flex items-center gap-2"
            >
              {connecting ? 'Redirecting...' : 'Connect LinkedIn'}
            </button>
            <p className="text-xs text-gray-500">
              Sign in with LinkedIn to enable API-driven data sync.
            </p>
          </div>
        )}
      </div>

      {/* Reconciliation Summary */}
      {reconciliation && (
        <div className="grid grid-cols-3 gap-3 mb-6">
          <div className="bg-gray-800 border border-yellow-700/50 rounded-xl p-4">
            <p className="text-yellow-400 text-2xl font-bold">{reconciliation.needs_linkedin_update.length}</p>
            <p className="text-xs text-gray-400 mt-1">Need LinkedIn update</p>
          </div>
          <div className="bg-gray-800 border border-blue-700/50 rounded-xl p-4">
            <p className="text-blue-400 text-2xl font-bold">{reconciliation.needs_db_import.length}</p>
            <p className="text-xs text-gray-400 mt-1">Need DB import</p>
          </div>
          <div className="bg-gray-800 border border-green-700/50 rounded-xl p-4">
            <p className="text-green-400 text-2xl font-bold">{reconciliation.in_sync.length}</p>
            <p className="text-xs text-gray-400 mt-1">In sync</p>
          </div>
        </div>
      )}

      {/* Import Section */}
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-5 mb-6">
        <h2 className="text-sm font-semibold text-gray-300 mb-3">Import LinkedIn Data</h2>
        <p className="text-xs text-gray-500 mb-3">
          Export your data from LinkedIn (Settings &rarr; Data Privacy &rarr; Get a copy of your data).
          Upload the Positions.csv or Projects.csv, or paste JSON directly.
        </p>

        <div className="flex gap-3 mb-3">
          <label className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition cursor-pointer">
            Upload CSV
            <input
              ref={fileInputRef}
              type="file"
              accept=".csv"
              className="hidden"
              onChange={handleFileUpload}
            />
          </label>
          <button
            onClick={handleAutoMatch}
            className="bg-purple-600 hover:bg-purple-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
          >
            Re-run Auto-Match
          </button>
          <span className="text-gray-500 text-sm self-center">or paste JSON below</span>
        </div>

        <textarea
          className="w-full h-32 bg-gray-900 border border-gray-600 rounded-lg p-3 text-sm text-gray-200 font-mono resize-y"
          placeholder='{"positions": [...], "projects": [...]}'
          value={jsonInput}
          onChange={e => setJsonInput(e.target.value)}
        />

        <div className="flex items-center gap-3 mt-3">
          <button
            onClick={handleImportJson}
            disabled={importing || !jsonInput.trim()}
            className="bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-600 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
          >
            {importing ? 'Importing...' : 'Import JSON'}
          </button>

          {importResult && (
            <p className="text-sm text-green-400">
              Imported {importResult.positions_imported} positions, {importResult.projects_imported} projects.
              Auto-matched: {importResult.positions_matched} positions, {importResult.projects_matched} projects.
            </p>
          )}
        </div>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      {/* Last sync info */}
      {syncLog.length > 0 && (
        <p className="text-xs text-gray-500 mb-4">
          Last import: {syncLog[0].imported_at} ({syncLog[0].source}) —{' '}
          {syncLog[0].positions_count} positions, {syncLog[0].projects_count} projects
        </p>
      )}

      {/* Tabs */}
      <div className="flex gap-1 mb-4">
        <button
          onClick={() => setTab('positions')}
          className={`px-4 py-2 text-sm rounded-t-lg transition ${
            tab === 'positions'
              ? 'bg-gray-800 text-white font-medium border border-gray-700 border-b-0'
              : 'text-gray-400 hover:text-gray-200'
          }`}
        >
          Positions ({positions.length})
        </button>
        <button
          onClick={() => setTab('projects')}
          className={`px-4 py-2 text-sm rounded-t-lg transition ${
            tab === 'projects'
              ? 'bg-gray-800 text-white font-medium border border-gray-700 border-b-0'
              : 'text-gray-400 hover:text-gray-200'
          }`}
        >
          Projects ({projects.length})
        </button>
      </div>

      {/* Bulk actions bar */}
      {selectedIds.size > 0 && (
        <div className="flex items-center gap-3 mb-3 bg-gray-800 border border-gray-600 rounded-lg px-4 py-2">
          <span className="text-sm text-gray-300">{selectedIds.size} selected</span>
          <button
            onClick={() => handleBulkStatus('synced')}
            className="text-xs bg-green-700 hover:bg-green-600 text-white px-3 py-1 rounded transition"
          >
            Mark Synced
          </button>
          <button
            onClick={() => handleBulkStatus('diverged')}
            className="text-xs bg-yellow-700 hover:bg-yellow-600 text-white px-3 py-1 rounded transition"
          >
            Mark Diverged
          </button>
          <button
            onClick={() => handleBulkStatus('linkedin_only')}
            className="text-xs bg-blue-700 hover:bg-blue-600 text-white px-3 py-1 rounded transition"
          >
            LinkedIn Only
          </button>
          <button
            onClick={() => setSelectedIds(new Set())}
            className="text-xs text-gray-400 hover:text-white ml-auto transition"
          >
            Clear
          </button>
        </div>
      )}

      {loading && <p className="text-gray-500 text-sm">Loading...</p>}

      {/* Positions Tab */}
      {tab === 'positions' && !loading && (
        <div className="space-y-2">
          {positions.length > 0 && (
            <label className="flex items-center gap-2 text-xs text-gray-400 mb-2 cursor-pointer">
              <input
                type="checkbox"
                checked={selectedIds.size === positions.length && positions.length > 0}
                onChange={toggleAll}
                className="rounded border-gray-600"
              />
              Select all
            </label>
          )}
          {positions.length === 0 && (
            <p className="text-gray-500 text-sm">No LinkedIn positions imported yet.</p>
          )}
          {positions.map(p => (
            <div key={p.id} className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={selectedIds.has(p.id)}
                onChange={() => toggleSelection(p.id)}
                className="rounded border-gray-600 shrink-0"
              />
              <Link
                to={`/dashboard/linkedin/positions/${p.id}`}
                className="flex-1 flex items-center justify-between bg-gray-800 border border-gray-700
                           hover:border-gray-500 rounded-xl px-5 py-4 transition"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <p className="text-white font-medium truncate">{p.title}</p>
                    <StatusBadge status={p.sync_status} />
                  </div>
                  <p className="text-sm text-gray-400 truncate">
                    {p.company} {p.location ? `· ${p.location}` : ''}
                  </p>
                </div>
                <p className="text-xs text-gray-500 shrink-0 ml-4">
                  {p.start_date} – {p.end_date ?? 'Present'}
                </p>
              </Link>
            </div>
          ))}
        </div>
      )}

      {/* Projects Tab */}
      {tab === 'projects' && !loading && (
        <div className="space-y-2">
          {projects.length > 0 && (
            <label className="flex items-center gap-2 text-xs text-gray-400 mb-2 cursor-pointer">
              <input
                type="checkbox"
                checked={selectedIds.size === projects.length && projects.length > 0}
                onChange={toggleAll}
                className="rounded border-gray-600"
              />
              Select all
            </label>
          )}
          {projects.length === 0 && (
            <p className="text-gray-500 text-sm">No LinkedIn projects imported yet.</p>
          )}
          {projects.map(p => (
            <div key={p.id} className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={selectedIds.has(p.id)}
                onChange={() => toggleSelection(p.id)}
                className="rounded border-gray-600 shrink-0"
              />
              <Link
                to={`/dashboard/linkedin/projects/${p.id}`}
                className="flex-1 flex items-center justify-between bg-gray-800 border border-gray-700
                           hover:border-gray-500 rounded-xl px-5 py-4 transition"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <p className="text-white font-medium truncate">{p.title}</p>
                    <StatusBadge status={p.sync_status} />
                  </div>
                  <p className="text-sm text-gray-400 truncate">
                    {p.associated_position ?? 'No associated position'}
                  </p>
                </div>
                <p className="text-xs text-gray-500 shrink-0 ml-4">
                  {p.start_date ?? '—'} – {p.end_date ?? 'Present'}
                </p>
              </Link>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

function parseCsv(
  text: string,
  filename: string,
): { positions: Array<Record<string, string | null>>; projects: Array<Record<string, string | null>> } {
  const lines = text.trim().split('\n')
  if (lines.length < 2) return { positions: [], projects: [] }

  const headers = lines[0].split(',').map(h => h.trim().replace(/^"|"$/g, ''))
  const rows = lines.slice(1).map(line => {
    const values = line.split(',').map(v => v.trim().replace(/^"|"$/g, ''))
    const obj: Record<string, string> = {}
    headers.forEach((h, i) => {
      obj[h] = values[i] ?? ''
    })
    return obj
  })

  const isPositions = filename.toLowerCase().includes('position')

  if (isPositions) {
    const positions = rows.map(r => ({
      company: r['Company Name'] ?? '',
      title: r['Title'] ?? '',
      location: r['Location'] || null,
      start_date: parseLinkedInDate(r['Started On'] ?? ''),
      end_date: r['Finished On'] ? parseLinkedInDate(r['Finished On']) : null,
      description: r['Description'] || null,
    }))
    return { positions, projects: [] }
  } else {
    const projects = rows.map(r => ({
      title: r['Title'] ?? '',
      description: r['Description'] || null,
      url: r['Url'] || null,
      start_date: r['Started On'] ? parseLinkedInDate(r['Started On']) : null,
      end_date: r['Finished On'] ? parseLinkedInDate(r['Finished On']) : null,
      associated_position: null,
    }))
    return { positions: [], projects }
  }
}

const MONTHS: Record<string, string> = {
  Jan: '01', Feb: '02', Mar: '03', Apr: '04', May: '05', Jun: '06',
  Jul: '07', Aug: '08', Sep: '09', Oct: '10', Nov: '11', Dec: '12',
}

function parseLinkedInDate(dateStr: string): string {
  const parts = dateStr.trim().split(' ')
  if (parts.length === 2) {
    const month = MONTHS[parts[0]] ?? '01'
    return `${parts[1]}-${month}`
  }
  return dateStr
}
