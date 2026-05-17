import { type ReactNode } from 'react'
import { NavLink, Outlet } from 'react-router-dom'
import { Helmet } from 'react-helmet-async'
import { useAuth } from '../../hooks/useAuth'

const NAV_ITEMS = [
  { to: '/dashboard', label: 'Overview', end: true },
  { to: '/dashboard/jobs', label: 'Jobs' },
  { to: '/dashboard/competencies', label: 'Competencies' },
  { to: '/dashboard/about', label: 'About Sections' },
  { to: '/dashboard/social-links', label: 'Social Links' },
  { to: '/dashboard/challenges', label: 'Challenges' },
]

export default function DashboardLayout({ children }: { children?: ReactNode }) {
  const { loading, email } = useAuth()

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyan-400" />
      </div>
    )
  }

  return (
    <>
      <Helmet>
        <title>Dashboard — Sharful Islam</title>
      </Helmet>

      <div className="flex min-h-screen bg-gray-900">
        <aside className="w-56 shrink-0 bg-gray-800 border-r border-gray-700 flex flex-col">
          <div className="px-4 py-5 border-b border-gray-700">
            <p className="text-xs text-gray-500 truncate">{email}</p>
            <p className="text-sm font-semibold text-white mt-0.5">Dashboard</p>
          </div>

          <nav className="flex-1 py-4 space-y-1 px-2">
            {NAV_ITEMS.map(item => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  `block px-3 py-2 rounded-lg text-sm transition ${
                    isActive
                      ? 'bg-gray-700 text-white font-medium'
                      : 'text-gray-400 hover:bg-gray-700/60 hover:text-gray-200'
                  }`
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>

          <div className="px-4 py-4 border-t border-gray-700">
            <a
              href="/auth/logout"
              className="block text-xs text-gray-500 hover:text-gray-300 transition"
            >
              Sign out
            </a>
          </div>
        </aside>

        <main className="flex-1 overflow-auto">
          <Outlet />
          {children}
        </main>
      </div>
    </>
  )
}
