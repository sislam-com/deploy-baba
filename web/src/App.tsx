import { Routes, Route } from 'react-router-dom'
import { Suspense, lazy } from 'react'

const Layout = lazy(() => import('./components/Layout'))
const Home = lazy(() => import('./routes/Home'))
const AboutMe = lazy(() => import('./routes/AboutMe'))
const AboutRepo = lazy(() => import('./routes/AboutRepo'))
const Contact = lazy(() => import('./routes/Contact'))
const Terms = lazy(() => import('./routes/Terms'))
const Privacy = lazy(() => import('./routes/Privacy'))
const CoverLetter = lazy(() => import('./routes/CoverLetter'))
const NotFound = lazy(() => import('./routes/NotFound'))

const Login = lazy(() => import('./routes/auth/Login'))
const ForgotPassword = lazy(() => import('./routes/auth/ForgotPassword'))
const ResetPassword = lazy(() => import('./routes/auth/ResetPassword'))
const ChangePassword = lazy(() => import('./routes/auth/ChangePassword'))

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
const Challenges = lazy(() => import('./routes/dashboard/Challenges'))
const ChallengeDetail = lazy(() => import('./routes/dashboard/ChallengeDetail'))
const LinkedInSync = lazy(() => import('./routes/dashboard/LinkedInSync'))
const LinkedInPositionDiff = lazy(() => import('./routes/dashboard/LinkedInPositionDiff'))
const LinkedInProjectDiff = lazy(() => import('./routes/dashboard/LinkedInProjectDiff'))

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
          <Route path="/cover-letter" element={<CoverLetter />} />
          <Route path="/terms" element={<Terms />} />
          <Route path="/privacy" element={<Privacy />} />
          <Route path="*" element={<NotFound />} />
        </Route>

        {/* Auth routes — no nav/footer */}
        <Route path="/auth/login" element={<Login />} />
        <Route path="/auth/forgot-password" element={<ForgotPassword />} />
        <Route path="/auth/reset-password" element={<ResetPassword />} />
        <Route path="/auth/change-password" element={<ChangePassword />} />

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
          <Route path="challenges" element={<Challenges />} />
          <Route path="challenges/:id" element={<ChallengeDetail />} />
          <Route path="linkedin" element={<LinkedInSync />} />
          <Route path="linkedin/positions/:id" element={<LinkedInPositionDiff />} />
          <Route path="linkedin/projects/:id" element={<LinkedInProjectDiff />} />
        </Route>
      </Routes>
    </Suspense>
  )
}
