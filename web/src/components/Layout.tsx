import { useEffect, useState } from 'react'
import { Link, NavLink, Outlet } from 'react-router-dom'
import SvgIcon from './SvgIcon'

interface SocialLink {
  url: string
  label: string
}

const SOCIAL_ICON_MAP: Record<string, string> = {
  linkedin: 'linkedin',
  github: 'github',
}

function NavIcon({
  to,
  icon,
  label,
  external,
}: {
  to: string
  icon: string
  label: string
  external?: boolean
}) {
  const cls = "p-2 rounded-lg transition"

  if (external) {
    return (
      <div className="group relative">
        <a
          href={to}
          target="_blank"
          rel="noopener noreferrer"
          className={`${cls} text-gray-400 hover:text-white focus-visible:ring-2 focus-visible:ring-cyan-500 focus-visible:outline-none`}
          aria-label={label}
        >
          <SvgIcon name={icon} className="w-5 h-5" />
        </a>
        <span className="absolute top-full mt-1 left-1/2 -translate-x-1/2 px-2 py-1 text-xs text-white bg-gray-700 rounded
                         opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition pointer-events-none whitespace-nowrap z-50">
          {label}
        </span>
      </div>
    )
  }

  return (
    <div className="group relative">
      <NavLink
        to={to}
        className={({ isActive }) =>
          `${cls} ${isActive ? 'text-cyan-400' : 'text-gray-400 hover:text-white'} focus-visible:ring-2 focus-visible:ring-cyan-500 focus-visible:outline-none`
        }
        aria-label={label}
      >
        <SvgIcon name={icon} className="w-5 h-5" />
      </NavLink>
      <span className="absolute top-full mt-1 left-1/2 -translate-x-1/2 px-2 py-1 text-xs text-white bg-gray-700 rounded
                       opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition pointer-events-none whitespace-nowrap z-50">
        {label}
      </span>
    </div>
  )
}

function MobileNavItem({
  to,
  icon,
  label,
  external,
  onClose,
}: {
  to: string
  icon: string
  label: string
  external?: boolean
  onClose: () => void
}) {
  const cls = "flex items-center gap-3 px-4 py-3 text-sm rounded-lg transition"

  if (external) {
    return (
      <a
        href={to}
        target="_blank"
        rel="noopener noreferrer"
        className={`${cls} text-gray-300 hover:bg-gray-800 hover:text-white`}
        onClick={onClose}
      >
        <SvgIcon name={icon} className="w-5 h-5 text-gray-400" />
        {label}
      </a>
    )
  }

  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        `${cls} ${isActive ? 'text-cyan-400 bg-gray-800' : 'text-gray-300 hover:bg-gray-800 hover:text-white'}`
      }
      onClick={onClose}
    >
      <SvgIcon name={icon} className="w-5 h-5" />
      {label}
    </NavLink>
  )
}

export default function Layout() {
  const [socialLinks, setSocialLinks] = useState<SocialLink[]>([])
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)

  useEffect(() => {
    fetch('/api/social-links')
      .then(r => r.json())
      .then((data: SocialLink[]) => setSocialLinks(Array.isArray(data) ? data : []))
      .catch(() => {})
  }, [])

  const closeMenu = () => setMobileMenuOpen(false)

  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col">
      <nav className="border-b border-gray-800 bg-gray-900/80 backdrop-blur-sm sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <Link to="/" className="text-xl font-bold text-cyan-400 hover:text-cyan-300 transition">
              Sharful Islam
            </Link>

            {/* Desktop nav */}
            <div className="hidden sm:flex items-center gap-3">
              <NavIcon to="/about/me" icon="user" label="About" />
              <NavIcon to="/contact" icon="envelope" label="Contact" />
              <NavIcon to="/docs" icon="document-text" label="API Docs" external />
              {socialLinks.map(link => (
                <NavIcon
                  key={link.url}
                  to={link.url}
                  icon={SOCIAL_ICON_MAP[link.label.toLowerCase()] ?? 'document'}
                  label={link.label}
                  external
                />
              ))}
              <NavIcon to="/auth/login" icon="key" label="Login" external />
            </div>

            {/* Mobile hamburger */}
            <button
              className="sm:hidden p-2 rounded-lg text-gray-400 hover:text-white transition
                         focus-visible:ring-2 focus-visible:ring-cyan-500 focus-visible:outline-none"
              onClick={() => setMobileMenuOpen(o => !o)}
              aria-label={mobileMenuOpen ? 'Close menu' : 'Open menu'}
              aria-expanded={mobileMenuOpen}
            >
              <SvgIcon name={mobileMenuOpen ? 'x-mark' : 'menu'} className="w-6 h-6" />
            </button>
          </div>
        </div>

        {/* Mobile menu panel */}
        {mobileMenuOpen && (
          <div className="sm:hidden border-t border-gray-800 bg-gray-900 px-4 py-3 space-y-1">
            <MobileNavItem to="/about/me" icon="user" label="About" onClose={closeMenu} />
            <MobileNavItem to="/contact" icon="envelope" label="Contact" onClose={closeMenu} />
            <MobileNavItem to="/docs" icon="document-text" label="API Docs" external onClose={closeMenu} />
            {socialLinks.map(link => (
              <MobileNavItem
                key={link.url}
                to={link.url}
                icon={SOCIAL_ICON_MAP[link.label.toLowerCase()] ?? 'document'}
                label={link.label}
                external
                onClose={closeMenu}
              />
            ))}
            <MobileNavItem to="/auth/login" icon="key" label="Login" external onClose={closeMenu} />
          </div>
        )}
      </nav>

      <main className="flex-1">
        <Outlet />
      </main>

      <footer className="border-t border-gray-800 bg-gray-800/50 mt-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
            <div>
              <h3 className="text-lg font-semibold mb-4 text-cyan-400">Sharful Islam</h3>
              <p className="text-gray-400 text-sm">AI Systems Engineer &middot; Portfolio</p>
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
            <p>&copy; 2026 Sharful Islam. Licensed under MIT</p>
          </div>
        </div>
      </footer>
    </div>
  )
}
