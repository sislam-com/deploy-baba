import { Helmet } from 'react-helmet-async'

export default function AboutRepo() {
  return (
    <>
      <Helmet>
        <title>About This Repo — Sharful Islam</title>
      </Helmet>
      <div className="max-w-4xl mx-auto px-4 py-12">
        <h1 className="text-4xl font-bold text-white">About This Repo</h1>
        <p className="text-gray-400 mt-2">Coming in Phase D.3</p>
      </div>
    </>
  )
}
