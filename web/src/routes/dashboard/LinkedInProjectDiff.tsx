import { useEffect, useState } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'

interface FieldComparison {
  field: string
  linkedin_value: string | null
  db_value: string | null
  differs: boolean
}

interface ProjectDiff {
  project: {
    id: number
    title: string
    description: string | null
    url: string | null
    start_date: string | null
    end_date: string | null
    associated_position: string | null
    mapped_challenge_id: number | null
    sync_status: string
  }
  challenge_title: string | null
  fields: FieldComparison[]
}

interface Challenge {
  id: number
  title: string
}

export default function LinkedInProjectDiff() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [diff, setDiff] = useState<ProjectDiff | null>(null)
  const [challenges, setChallenges] = useState<Challenge[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedChallengeId, setSelectedChallengeId] = useState<string>('')

  useEffect(() => {
    Promise.all([
      fetch(`/api/v1/admin/linkedin/projects/${id}/diff`).then(r => {
        if (!r.ok) throw new Error('Project not found')
        return r.json()
      }),
      fetch('/api/challenges').then(r => r.json()),
    ])
      .then(([diffData, chData]) => {
        setDiff(diffData)
        setChallenges(Array.isArray(chData) ? chData : [])
        if (diffData.project.mapped_challenge_id) {
          setSelectedChallengeId(String(diffData.project.mapped_challenge_id))
        }
      })
      .catch(e => setError(e.message))
      .finally(() => setLoading(false))
  }, [id])

  const handleMap = async () => {
    const targetId = selectedChallengeId ? parseInt(selectedChallengeId, 10) : null
    const res = await fetch(`/api/v1/admin/linkedin/projects/${id}/map`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ target_id: targetId }),
    })
    if (res.ok) {
      window.location.reload()
    }
  }

  const handleStatusUpdate = async (status: string) => {
    const res = await fetch(`/api/v1/admin/linkedin/projects/${id}/status`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    })
    if (res.ok) {
      navigate('/dashboard/linkedin')
    }
  }

  if (loading) return <div className="p-8 text-gray-500">Loading...</div>
  if (error) return <div className="p-8 text-red-400">{error}</div>
  if (!diff) return <div className="p-8 text-gray-500">Not found</div>

  const { project, fields } = diff

  return (
    <div className="p-8 max-w-4xl">
      <Link to="/dashboard/linkedin" className="text-cyan-400 text-sm hover:underline mb-4 block">
        &larr; Back to LinkedIn Sync
      </Link>

      <h1 className="text-2xl font-bold text-white mb-1">{project.title}</h1>
      {project.associated_position && (
        <p className="text-gray-400 mb-6">{project.associated_position}</p>
      )}

      {/* Mapping */}
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-5 mb-6">
        <h2 className="text-sm font-semibold text-gray-300 mb-3">Map to Internal Challenge</h2>
        <div className="flex gap-3 items-center">
          <select
            value={selectedChallengeId}
            onChange={e => setSelectedChallengeId(e.target.value)}
            className="bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 text-sm text-gray-200 flex-1"
          >
            <option value="">— No mapping —</option>
            {challenges.map(c => (
              <option key={c.id} value={c.id}>
                {c.title}
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
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left px-4 py-3 text-gray-500 font-medium w-32">Field</th>
                <th className="text-left px-4 py-3 text-gray-500 font-medium">LinkedIn</th>
                <th className="text-left px-4 py-3 text-gray-500 font-medium">Database</th>
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
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {fields.length === 0 && project.mapped_challenge_id === null && (
        <p className="text-gray-500 text-sm mb-6">
          Map this project to an internal challenge to see a field-by-field comparison.
        </p>
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
