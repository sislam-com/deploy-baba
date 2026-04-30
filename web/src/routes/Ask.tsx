import { Helmet } from 'react-helmet-async'

export default function Ask() {
  return (
    <>
      <Helmet>
        <title>Ask — Sharful Islam</title>
      </Helmet>
      <div className="max-w-4xl mx-auto px-4 py-12">
        <h1 className="text-4xl font-bold text-white">Ask</h1>
        <p className="text-gray-400 mt-2">Interactive Q&amp;A — coming in Phase D.2</p>
      </div>
    </>
  )
}
