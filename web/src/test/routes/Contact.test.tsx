import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '../utils/test-render'
import userEvent from '@testing-library/user-event'
import Contact from '../../routes/Contact'

// Mock crypto.subtle.digest for PoW testing
global.crypto = {
  subtle: {
    digest: vi.fn(async () => {
      // Return a mock hash that will satisfy the PoW difficulty
      const hash = new Uint8Array(32)
      hash[0] = 0 // Set first byte to 0 to satisfy difficulty=1
      return hash
    }),
  },
} as any

describe('Contact', () => {
  it('renders page heading', () => {
    render(<Contact />)
    expect(screen.getByText('Contact')).toBeInTheDocument()
  })

  it('renders page description', () => {
    render(<Contact />)
    expect(screen.getByText('Send me a message. I typically respond within a day or two.')).toBeInTheDocument()
  })

  it('renders form fields', () => {
    render(<Contact />)
    expect(screen.getByLabelText('Name')).toBeInTheDocument()
    expect(screen.getByLabelText('Email')).toBeInTheDocument()
    expect(screen.getByLabelText('Subject')).toBeInTheDocument()
    expect(screen.getByLabelText('Message')).toBeInTheDocument()
  })

  it('renders submit button', () => {
    render(<Contact />)
    expect(screen.getByRole('button', { name: 'Send Message' })).toBeInTheDocument()
  })

  it('validates required fields', async () => {
    render(<Contact />)
    const user = userEvent.setup()

    const submitButton = screen.getByRole('button', { name: 'Send Message' })
    await user.click(submitButton)

    // Browser validation should prevent submission
    const nameInput = screen.getByLabelText('Name')
    expect(nameInput).toBeInvalid()
  })

  it('submits form with valid data', async () => {
    render(<Contact />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Name'), 'Test User')
    await user.type(screen.getByLabelText('Email'), 'test@example.com')
    await user.type(screen.getByLabelText('Subject'), 'Test Subject')
    await user.type(screen.getByLabelText('Message'), 'Test message content')

    const submitButton = screen.getByRole('button', { name: 'Send Message' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/Message sent/)).toBeInTheDocument()
    })
  })

  it('shows loading state during submission', async () => {
    render(<Contact />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Name'), 'Test User')
    await user.type(screen.getByLabelText('Email'), 'test@example.com')
    await user.type(screen.getByLabelText('Subject'), 'Test Subject')
    await user.type(screen.getByLabelText('Message'), 'Test message content')

    const submitButton = screen.getByRole('button', { name: 'Send Message' })
    await user.click(submitButton)

    // Button should be disabled during submission
    expect(submitButton).toBeDisabled()
  })

  it('displays success message after successful submission', async () => {
    render(<Contact />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Name'), 'Test User')
    await user.type(screen.getByLabelText('Email'), 'test@example.com')
    await user.type(screen.getByLabelText('Subject'), 'Test Subject')
    await user.type(screen.getByLabelText('Message'), 'Test message content')

    const submitButton = screen.getByRole('button', { name: 'Send Message' })
    await user.click(submitButton)

    await waitFor(() => {
      const successMessage = screen.getByText(/Message sent/)
      expect(successMessage).toBeInTheDocument()
      expect(successMessage).toHaveClass('bg-green-900/60')
    })
  })

  it('displays error message on submission failure', async () => {
    // Mock a failed API response
    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: false,
        json: () => Promise.resolve({ success: false, message: 'API Error' }),
      })
    )

    render(<Contact />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Name'), 'Test User')
    await user.type(screen.getByLabelText('Email'), 'test@example.com')
    await user.type(screen.getByLabelText('Subject'), 'Test Subject')
    await user.type(screen.getByLabelText('Message'), 'Test message content')

    const submitButton = screen.getByRole('button', { name: 'Send Message' })
    await user.click(submitButton)

    await waitFor(() => {
      const errorMessage = screen.getByText(/API Error/)
      expect(errorMessage).toBeInTheDocument()
      expect(errorMessage).toHaveClass('bg-red-900/60')
    })
  })

  it('resets form after successful submission', async () => {
    render(<Contact />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Name'), 'Test User')
    await user.type(screen.getByLabelText('Email'), 'test@example.com')
    await user.type(screen.getByLabelText('Subject'), 'Test Subject')
    await user.type(screen.getByLabelText('Message'), 'Test message content')

    const submitButton = screen.getByRole('button', { name: 'Send Message' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/Message sent/)).toBeInTheDocument()
    })

    // Form should be reset
    expect(screen.getByLabelText('Name')).toHaveValue('')
    expect(screen.getByLabelText('Email')).toHaveValue('')
    expect(screen.getByLabelText('Subject')).toHaveValue('')
    expect(screen.getByLabelText('Message')).toHaveValue('')
  })

  it('shows character count for message field', async () => {
    render(<Contact />)
    const user = userEvent.setup()

    const messageInput = screen.getByLabelText('Message')
    await user.type(messageInput, 'Test')

    expect(screen.getByText('4/5000')).toBeInTheDocument()
  })

  it('has honeypot field hidden from users', () => {
    render(<Contact />)
    const honeypot = screen.getByLabelText('website')
    expect(honeypot).toHaveStyle({ display: 'none' })
    expect(honeypot).toHaveAttribute('tabIndex', '-1')
  })

  it('sets correct page title', () => {
    render(<Contact />)
    expect(document.title).toBe('Contact — Portfolio')
  })
})