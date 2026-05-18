import { render, RenderOptions } from '@testing-library/react'
import { ReactElement } from 'react'
import { BrowserRouter, MemoryRouter, Route, Routes } from 'react-router-dom'
import { HelmetProvider } from 'react-helmet-async'

// Custom render with router and helmet providers
interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  router?: 'memory' | 'browser'
  route?: string
  routes?: { path: string; element?: ReactElement }[]
}

export function renderWithProviders(
  ui: ReactElement,
  {
    router = 'memory',
    route = '/',
    routes,
    ...renderOptions
  }: CustomRenderOptions = {}
) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => {
    if (router === 'memory') {
      if (routes) {
        return (
          <HelmetProvider>
            <MemoryRouter initialEntries={[route]} future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
              <Routes>
                {routes.map(r => (
                  <Route key={r.path} path={r.path} element={r.element ?? children} />
                ))}
              </Routes>
            </MemoryRouter>
          </HelmetProvider>
        )
      }
      return (
        <HelmetProvider>
          <MemoryRouter initialEntries={[route]} future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>{children}</MemoryRouter>
        </HelmetProvider>
      )
    }

    return (
      <HelmetProvider>
        <BrowserRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>{children}</BrowserRouter>
      </HelmetProvider>
    )
  }

  return {
    ...render(ui, { wrapper: Wrapper, ...renderOptions }),
  }
}

// Re-export everything from RTL
export * from '@testing-library/react'
export { default as userEvent } from '@testing-library/user-event'

// Override render so tests importing from this module get the wrapped version
export { renderWithProviders as render }