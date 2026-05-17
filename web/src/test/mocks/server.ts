import { setupServer } from 'msw/node'
import { handlers } from './handlers'

// Setup MSW server for Node.js environment (Vitest)
export const server = setupServer(...handlers)

// Setup and teardown for test lifecycle
export const setupMSW = () => {
  let originalFetch: typeof fetch
  beforeAll(() => {
    server.listen({ onUnhandledRequest: 'error' })
    originalFetch = globalThis.fetch
  })
  beforeEach(() => {
    // Restore MSW's fetch interceptor so tests that mock global.fetch
    // don't leak their mock to subsequent tests in the same file.
    if (originalFetch) {
      globalThis.fetch = originalFetch
    }
  })
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())
}
