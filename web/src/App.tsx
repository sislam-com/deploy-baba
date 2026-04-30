import { Routes, Route } from 'react-router-dom'
import { Suspense, lazy } from 'react'

const Home = lazy(() => import('./routes/Home'))
const AboutMe = lazy(() => import('./routes/AboutMe'))
const AboutRepo = lazy(() => import('./routes/AboutRepo'))
const Contact = lazy(() => import('./routes/Contact'))
const Ask = lazy(() => import('./routes/Ask'))
const NotFound = lazy(() => import('./routes/NotFound'))

function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center min-h-screen">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyan-400" />
    </div>
  )
}

export default function App() {
  return (
    <Suspense fallback={<LoadingSpinner />}>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/about/me" element={<AboutMe />} />
        <Route path="/about/repo" element={<AboutRepo />} />
        <Route path="/contact" element={<Contact />} />
        <Route path="/ask" element={<Ask />} />
        <Route path="*" element={<NotFound />} />
      </Routes>
    </Suspense>
  )
}
