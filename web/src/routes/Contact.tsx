import { useRef, useState, FormEvent } from 'react'
import { Helmet } from 'react-helmet-async'

interface Challenge {
  nonce: string
  difficulty: number
  timestamp: number
  signature: string
}

function hasLeadingZeroBits(hash: Uint8Array, bits: number): boolean {
  const fullBytes = Math.floor(bits / 8)
  const remainingBits = bits % 8
  for (let i = 0; i < fullBytes; i++) {
    if (hash[i] !== 0) return false
  }
  if (remainingBits > 0) {
    const mask = (0xff << (8 - remainingBits)) & 0xff
    if ((hash[fullBytes] & mask) !== 0) return false
  }
  return true
}

async function solvePoW(nonce: string, difficulty: number): Promise<number> {
  const encoder = new TextEncoder()
  let solution = 0
  while (true) {
    const data = encoder.encode(`${nonce}:${solution}`)
    const hashBuffer = await crypto.subtle.digest('SHA-256', data)
    const hash = new Uint8Array(hashBuffer)
    if (hasLeadingZeroBits(hash, difficulty)) return solution
    solution++
    if (solution % 1000 === 0) await new Promise(r => setTimeout(r, 0))
  }
}

export default function Contact() {
  const honeypotRef = useRef<HTMLInputElement>(null)
  const [form, setForm] = useState({ name: '', email: '', subject: '', message: '' })
  const [status, setStatus] = useState<{ ok: boolean; text: string } | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [submitLabel, setSubmitLabel] = useState('Send Message')

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    setSubmitting(true)
    setStatus(null)

    try {
      setSubmitLabel('Verifying…')
      const challengeRes = await fetch('/api/contact/challenge')
      if (!challengeRes.ok) throw new Error('challenge')
      const challenge: Challenge = await challengeRes.json()

      setSubmitLabel('Solving challenge…')
      const solution = await solvePoW(challenge.nonce, challenge.difficulty)

      setSubmitLabel('Sending…')
      const res = await fetch('/api/contact', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ...form,
          website: honeypotRef.current?.value ?? '',
          pow_nonce: challenge.nonce,
          pow_timestamp: challenge.timestamp,
          pow_solution: solution,
          pow_signature: challenge.signature,
        }),
      })
      const data = await res.json()
      if (data.success) {
        setStatus({ ok: true, text: "Message sent! I'll be in touch soon." })
        setForm({ name: '', email: '', subject: '', message: '' })
      } else {
        setStatus({ ok: false, text: data.message ?? 'Something went wrong. Please try again.' })
      }
    } catch {
      setStatus({ ok: false, text: 'Network error. Please check your connection and try again.' })
    } finally {
      setSubmitting(false)
      setSubmitLabel('Send Message')
    }
  }

  return (
    <>
      <Helmet>
        <title>Contact — Portfolio</title>
      </Helmet>

      <div className="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="mb-10">
          <h1 className="text-4xl font-bold text-white mb-2">Contact</h1>
          <p className="text-gray-400">Send me a message. I typically respond within a day or two.</p>
        </div>

        {status && (
          <div
            className={`mb-6 px-4 py-3 rounded-lg text-sm font-medium ${
              status.ok
                ? 'bg-green-900/60 border border-green-700 text-green-300'
                : 'bg-red-900/60 border border-red-700 text-red-300'
            }`}
          >
            {status.text}
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-5">
          {/* Honeypot */}
          <div style={{ display: 'none' }} aria-hidden="true">
            <input
              ref={honeypotRef}
              type="text"
              name="website"
              tabIndex={-1}
              autoComplete="off"
            />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-5">
            <div>
              <label htmlFor="name" className="block text-sm font-medium text-gray-300 mb-1">
                Name
              </label>
              <input
                id="name"
                type="text"
                required
                maxLength={100}
                value={form.name}
                onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
                placeholder="Sharful Islam"
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white
                           placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500 transition"
              />
            </div>
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-300 mb-1">
                Email
              </label>
              <input
                id="email"
                type="email"
                required
                maxLength={254}
                value={form.email}
                onChange={e => setForm(f => ({ ...f, email: e.target.value }))}
                placeholder="you@example.com"
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white
                           placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500 transition"
              />
            </div>
          </div>

          <div>
            <label htmlFor="subject" className="block text-sm font-medium text-gray-300 mb-1">
              Subject
            </label>
            <input
              id="subject"
              type="text"
              required
              maxLength={200}
              value={form.subject}
              onChange={e => setForm(f => ({ ...f, subject: e.target.value }))}
              placeholder="e.g. Collaboration opportunity"
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white
                         placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500 transition"
            />
          </div>

          <div>
            <label htmlFor="message" className="block text-sm font-medium text-gray-300 mb-1">
              Message
            </label>
            <textarea
              id="message"
              required
              maxLength={5000}
              rows={6}
              value={form.message}
              onChange={e => setForm(f => ({ ...f, message: e.target.value }))}
              placeholder="Your message…"
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white
                         placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500 transition resize-none"
            />
            <p className="text-gray-600 text-xs mt-1 text-right">{form.message.length}/5000</p>
          </div>

          <button
            type="submit"
            disabled={submitting}
            className="w-full bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 disabled:cursor-not-allowed
                       text-white font-semibold py-3 px-6 rounded-lg transition focus:outline-none
                       focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-gray-900"
          >
            {submitLabel}
          </button>
        </form>
      </div>
    </>
  )
}
