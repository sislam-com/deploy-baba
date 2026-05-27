import { describe, it, expect } from 'vitest'
import { render, screen } from '../../utils/test-render'
import ChangePassword from '../../../routes/auth/ChangePassword'

describe('ChangePassword', () => {
  it('shows redirect message without challenge state', () => {
    render(<ChangePassword />, { router: 'memory', route: '/auth/change-password' })
    expect(screen.getByText(/no active challenge/i)).toBeInTheDocument()
  })
})
