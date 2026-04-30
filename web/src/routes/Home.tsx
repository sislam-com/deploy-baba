import { Helmet } from 'react-helmet-async'

export default function Home() {
  return (
    <>
      <Helmet>
        <title>Sharful Islam — Portfolio</title>
      </Helmet>
      <div className="max-w-4xl mx-auto px-4 py-12">
        <h1 className="text-4xl font-bold text-white">Sharful Islam</h1>
        <p className="text-gray-400 mt-2">Full resume loading — coming in Phase D.3</p>
      </div>
    </>
  )
}
