import { useEffect, useState } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'

interface FieldComparison {
  field: string
  linkedin_value: string | null
  db_value: string | null
  differs: boolean
}

interface PositionDiff {
  position: {
    id: number
    company: string
    title: string
    location: string | null
    start_date: string
    end_date: string | null
    description: string | null
    mapped_job_id: number | null
    sync_status: string
  }
  job_title: string | null
  job_company: string | null
  fields: FieldComparison[]
}

interface Job {
  id: number
  company: string
  title: string
}

export default function LinkedInPositionDiff() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [diff, setDiff] = useState<PositionDiff | null>(null)
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedJobId, setSelectedJobId] = useState<string>('')
  const [applying, setApplying] = useState(false)
  const [applyResult, setApplyResult] = useState<string | null>(null)
  const [copiedField, setCopiedField] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([
      fetch(`/api/v1/admin/linkedin/positions/${id}/diff`).then(r => {
        if (!r.ok) throw new Error('Position not found')
        return r.json()
      }),
      fetch('/api/jobs').then(r => r.json()),
    ])
      .then(([diffData, jobsData]) => {
        setDiff(diffData)
        setJobs(Array.isArray(jobsData) ? jobsData : [])
        if (diffData.position.mapped_job_id) {
          setSelectedJobId(String(diffData.position.mapped_job_id))
        }
      })
      .catch(e => setError(e.message))
      .finally(() => setLoading(false))
  }, [id])

  const handleMap = async () => {
    const targetId = selectedJobId ? parseInt(selectedJobId, 10) : null
    const res = await fetch(`/api/v1/admin/linkedin/positions/${id}/map`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ target_id: targetId }),
    })
    if (res.ok) {
      window.location.reload()
    }
  }

  const handleStatusUpdate = async (status: string) => {
    const res = await fetch(`/api/v1/admin/linkedin/positions/${id}/status`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    })
    if (res.ok) {
      navigate('/dashboard/linkedin')
    }
  }

  const handleApplyAll = async () => {
    if (!diff) return
    const differingFields = diff.fields.filter(f => f.differs).map(f => f.field)
    if (differingFields.length === 0) return

    setApplying(true)
    setApplyResult(null)
    try {
      const res = await fetch(`/api/v1/admin/linkedin/positions/${id}/apply`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ fields: differingFields }),
      })
      if (!res.ok) throw new Error(await res.text())
      const result = await res.json()
      setApplyResult(`Applied: ${result.fields_applied.join(', ')}`)
      window.location.reload()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Apply failed')
    } finally {
      setApplying(false)
    }
  }

  const handleApplyField = async (field: string) => {
    setApplying(true)
    setApplyResult(null)
    try {
      const res = await fetch(`/api/v1/admin/linkedin/positions/${id}/apply`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ fields: [field] }),
      })
      if (!res.ok) throw new Error(await res.text())
      setApplyResult(`Applied: ${field}`)
      window.location.reload()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Apply failed')
    } finally {
      setApplying(false)
    }
  }

  const copyToClipboard = (field: string, value: string | null) => {
    if (!value) return
    navigator.clipboard.writeText(value).then(() => {
      setCopiedField(field)
      setTimeout(() => setCopiedField(null), 1500)
    })
  }

  if (loading) return <div className="p-8 text-gray-500">Loading...</div>
  if (error) return <div className="p-8 text-red-400">{error}</div>
  if (!diff) return <div className="p-8 text-gray-500">Not found</div>

  const { position, fields } = diff
  const hasDiffs = fields.some(f => f.differs)

  return (
    <div className="p-8 max-w-4xl">
      <Link to="/dashboard/linkedin" className="text-cyan-400 text-sm hover:underline mb-4 block">
        &larr; Back to LinkedIn Sync
      </Link>

      <h1 className="text-2xl font-bold text-white mb-1">{position.title}</h1>
      <p className="text-gray-400 mb-6">{position.company}</p>

      {/* Mapping */}
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-5 mb-6">
        <h2 className="text-sm font-semibold text-gray-300 mb-3">Map to Internal Job</h2>
        <div className="flex gap-3 items-center">
          <select
            value={selectedJobId}
            onChange={e => setSelectedJobId(e.target.value)}
            className="bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 text-sm text-gray-200 flex-1"
          >
            <option value="">— No mapping —</option>
            {jobs.map(j => (
              <option key={j.id} value={j.id}>
                {j.title} @ {j.company}
              </option>
            ))}
          </select>
          <button
            onClick={handleMap}
            className="bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
          >
            Save Mapping
          </button>
        </div>
      </div>

      {/* Field-by-field diff */}
      {fields.length > 0 && (
        <div className="bg-gray-800 border border-gray-700 rounded-xl overflow-hidden mb-6">
          {hasDiffs && position.mapped_job_id && (
            <div className="flex items-center justify-between px-4 py-3 border-b border-gray-700 bg-gray-800/50">
              <p className="text-sm text-gray-400">
                {fields.filter(f => f.differs).length} field(s) differ
              </p>
              <button
                onClick={handleApplyAll}
                disabled={applying}
                className="bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-600 text-white text-xs font-semibold px-3 py-1.5 rounded-lg transition"
              >
                {applying ? 'Applying...' : 'Apply All LinkedIn → DB'}
              </button>
            </div>
          )}
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left px-4 py-3 text-gray-500 font-medium w-32">Field</th>
                <th className="text-left px-4 py-3 text-gray-500 font-medium">LinkedIn</th>
                <th className="text-left px-4 py-3 text-gray-500 font-medium">Database</th>
                <th className="text-right px-4 py-3 text-gray-500 font-medium w-24">Actions</th>
              </tr>
            </thead>
            <tbody>
              {fields.map(f => (
                <tr
                  key={f.field}
                  className={`border-b border-gray-700/50 ${f.differs ? 'bg-yellow-900/10' : ''}`}
                >
                  <td className="px-4 py-3 text-gray-400 font-medium">{f.field}</td>
                  <td className={`px-4 py-3 ${f.differs ? 'text-red-300' : 'text-gray-300'}`}>
                    {f.linkedin_value ?? <span className="text-gray-600 italic">empty</span>}
                  </td>
                  <td className={`px-4 py-3 ${f.differs ? 'text-green-300' : 'text-gray-300'}`}>
                    {f.db_value ?? <span className="text-gray-600 italic">empty</span>}
                  </td>
                  <td className="px-4 py-3 text-right">
                    <div className="flex gap-1 justify-end">
                      {f.differs && position.mapped_job_id && (
                        <button
                          onClick={() => handleApplyField(f.field)}
                          disabled={applying}
                          className="text-[10px] bg-cyan-700 hover:bg-cyan-600 text-white px-2 py-0.5 rounded transition"
                          title="Apply LinkedIn value to DB"
                        >
                          Apply
                        </button>
                      )}
                      {f.db_value && (
                        <button
                          onClick={() => copyToClipboard(f.field, f.db_value)}
                          className="text-[10px] bg-gray-700 hover:bg-gray-600 text-gray-300 px-2 py-0.5 rounded transition"
                          title="Copy DB value (for manual LinkedIn update)"
                        >
                          {copiedField === f.field ? 'Copied' : 'Copy'}
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {fields.length === 0 && position.mapped_job_id === null && (
        <p className="text-gray-500 text-sm mb-6">
          Map this position to an internal job to see a field-by-field comparison.
        </p>
      )}

      {applyResult && (
        <p className="text-green-400 text-sm mb-4">{applyResult}</p>
      )}

      {/* Actions */}
      <div className="flex gap-3">
        <button
          onClick={() => handleStatusUpdate('synced')}
          className="bg-green-700 hover:bg-green-600 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          Mark as Synced
        </button>
        <button
          onClick={() => handleStatusUpdate('diverged')}
          className="bg-yellow-700 hover:bg-yellow-600 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          Mark as Diverged
        </button>
        <button
          onClick={() => handleStatusUpdate('linkedin_only')}
          className="bg-blue-700 hover:bg-blue-600 text-white text-sm font-semibold px-4 py-2 rounded-lg transition"
        >
          LinkedIn Only
        </button>
      </div>
    </div>
  )
}
