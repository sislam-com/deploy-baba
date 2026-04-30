import { describe, it, expect } from 'vitest'

describe('SPA scaffold', () => {
  it('has a valid environment', () => {
    expect(typeof window).toBe('object')
  })
})
