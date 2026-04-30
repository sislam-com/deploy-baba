import { Helmet } from 'react-helmet-async'
import { Link } from 'react-router-dom'

export default function NotFound() {
  return (
    <>
      <Helmet>
        <title>404 — Sharful Islam</title>
      </Helmet>
      <div className="max-w-4xl mx-auto px-4 py-12 text-center">
        <h1 className="text-6xl font-bold text-white">404</h1>
        <p className="text-gray-400 mt-4">Page not found.</p>
        <Link to="/" className="mt-6 inline-block text-cyan-400 hover:underline">
          ← Back home
        </Link>
      </div>
    </>
  )
}
