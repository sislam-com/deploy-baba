import { Routes, Route } from 'react-router-dom'
import { Suspense, lazy } from 'react'

const Layout = lazy(() => import('./components/Layout'))
const Home = lazy(() => import('./routes/Home'))
const AboutMe = lazy(() => import('./routes/AboutMe'))
const AboutRepo = lazy(() => import('./routes/AboutRepo'))
const Contact = lazy(() => import('./routes/Contact'))
const Ask = lazy(() => import('./routes/Ask'))
const NotFound = lazy(() => import('./routes/NotFound'))

const DashboardLayout = lazy(() => import('./routes/dashboard/Layout'))
const DashboardHome = lazy(() => import('./routes/dashboard/index'))
const Jobs = lazy(() => import('./routes/dashboard/Jobs'))
const JobDetail = lazy(() => import('./routes/dashboard/JobDetail'))
const Competencies = lazy(() => import('./routes/dashboard/Competencies'))
const CompetencyDetail = lazy(() => import('./routes/dashboard/CompetencyDetail'))
const About = lazy(() => import('./routes/dashboard/About'))
const AboutDetail = lazy(() => import('./routes/dashboard/AboutDetail'))
const SocialLinks = lazy(() => import('./routes/dashboard/SocialLinks'))
const SocialLinkDetail = lazy(() => import('./routes/dashboard/SocialLinkDetail'))

function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-900">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyan-400" />
    </div>
  )
}

export default function App() {
  return (
    <Suspense fallback={<LoadingSpinner />}>
      <Routes>
        {/* Marketing routes — share the nav/footer Layout */}
        <Route element={<Layout />}>
          <Route path="/" element={<Home />} />
          <Route path="/about/me" element={<AboutMe />} />
          <Route path="/about/repo" element={<AboutRepo />} />
          <Route path="/contact" element={<Contact />} />
          <Route path="/ask" element={<Ask />} />
          <Route path="*" element={<NotFound />} />
        </Route>

        {/* Dashboard — own sidebar layout, auth-gated */}
        <Route path="/dashboard" element={<DashboardLayout />}>
          <Route index element={<DashboardHome />} />
          <Route path="jobs" element={<Jobs />} />
          <Route path="jobs/:id" element={<JobDetail />} />
          <Route path="competencies" element={<Competencies />} />
          <Route path="competencies/:id" element={<CompetencyDetail />} />
          <Route path="about" element={<About />} />
          <Route path="about/:id" element={<AboutDetail />} />
          <Route path="social-links" element={<SocialLinks />} />
          <Route path="social-links/:id" element={<SocialLinkDetail />} />
        </Route>
      </Routes>
    </Suspense>
  )
}
