import { useEffect, useState } from 'react'
import { Link, NavLink, Outlet } from 'react-router-dom'

interface SocialLink {
  id: number
  platform: string
  url: string
  label: string
}

export default function Layout() {
  const [socialLinks, setSocialLinks] = useState<SocialLink[]>([])

  useEffect(() => {
    fetch('/api/social-links')
      .then(r => r.json())
      .then((data: SocialLink[]) => setSocialLinks(Array.isArray(data) ? data : []))
      .catch(() => {})
  }, [])

  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col">
      <nav className="border-b border-gray-800 bg-gray-800/50 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <Link to="/" className="text-xl font-bold text-cyan-400 hover:text-cyan-300 transition">
              Sharful Islam
            </Link>
            <div className="flex items-center space-x-6">
              <NavLink
                to="/about/me"
                className={({ isActive }) =>
                  `text-sm transition ${isActive ? 'text-white' : 'text-gray-300 hover:text-white'}`
                }
              >
                About
              </NavLink>
              <NavLink
                to="/contact"
                className={({ isActive }) =>
                  `text-sm transition ${isActive ? 'text-white' : 'text-gray-300 hover:text-white'}`
                }
              >
                Contact
              </NavLink>
              <a href="/docs" className="text-sm text-gray-300 hover:text-white transition">
                API Docs
              </a>
              {socialLinks.map(link => (
                <a
                  key={link.id}
                  href={link.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-gray-300 hover:text-white transition"
                >
                  {link.label}
                </a>
              ))}
              <a href="/auth/login" className="text-sm text-gray-300 hover:text-white transition">
                Login
              </a>
            </div>
          </div>
        </div>
      </nav>

      <main className="flex-1">
        <Outlet />
      </main>

      <footer className="border-t border-gray-800 bg-gray-800/50 mt-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
            <div>
              <h3 className="text-lg font-semibold mb-4 text-cyan-400">Sharful Islam</h3>
              <p className="text-gray-400 text-sm">Full-Stack SaaS Engineer · Portfolio</p>
            </div>
            <div>
              <h3 className="text-lg font-semibold mb-4 text-cyan-400">Links</h3>
              <ul className="space-y-2 text-sm">
                <li>
                  <a
                    href="https://github.com/shantopagla/deploy-baba"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-gray-400 hover:text-white transition"
                  >
                    GitHub Repository
                  </a>
                </li>
                <li>
                  <a
                    href="/api/openapi.json"
                    className="text-gray-400 hover:text-white transition"
                  >
                    OpenAPI Spec
                  </a>
                </li>
              </ul>
            </div>
            <div>
              <h3 className="text-lg font-semibold mb-4 text-cyan-400">License</h3>
              <p className="text-gray-400 text-sm">Dual-licensed under MIT or Apache-2.0</p>
              <p className="text-gray-400 text-sm mt-2">
                Built by{' '}
                <a
                  href="https://github.com/shantopagla"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-cyan-400 hover:text-cyan-300"
                >
                  Shanto
                </a>
              </p>
            </div>
          </div>
          <div className="border-t border-gray-700 pt-6 text-center text-gray-400 text-sm">
            <p>&copy; 2026 Sharful Islam. Licensed under MIT OR Apache-2.0</p>
          </div>
        </div>
      </footer>
    </div>
  )
}
