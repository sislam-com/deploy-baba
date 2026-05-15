import { setupServer } from 'msw/node'
import { handlers } from './handlers'

// Setup MSW server for Node.js environment (Vitest)
export const server = setupServer(...handlers)

// Setup and teardown for test lifecycle
export const setupMSW = () => {
  beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())
}