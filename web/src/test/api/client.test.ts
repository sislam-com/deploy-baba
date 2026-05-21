import { describe, it, expect } from 'vitest'
import { apiClient } from '../../api/client'

describe('apiClient', () => {
  it('is exported as a singleton', () => {
    expect(apiClient).toBeDefined()
    expect(typeof apiClient).toBe('object')
  })

  it('has GET and POST method helpers', () => {
    expect(typeof apiClient.GET).toBe('function')
    expect(typeof apiClient.POST).toBe('function')
  })
})
