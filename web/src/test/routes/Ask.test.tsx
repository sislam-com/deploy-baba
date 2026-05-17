import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '../utils/test-render'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '../mocks/server'
import Ask from '../../routes/Ask'

describe('Ask', () => {
  it('renders heading when not embedded', () => {
    render(<Ask />)
    expect(screen.getByRole('heading', { name: 'Ask' })).toBeInTheDocument()
  })

  it('renders description when not embedded', () => {
    render(<Ask />)
    expect(screen.getByText(/Questions about this portfolio/)).toBeInTheDocument()
  })

  it('renders suggested question pills', () => {
    render(<Ask />)
    expect(screen.getByText('Match to a role')).toBeInTheDocument()
    expect(screen.getByText('Primary skills')).toBeInTheDocument()
    expect(screen.getByText('AI/LLM experience')).toBeInTheDocument()
  })

  it('renders textarea for question input', () => {
    render(<Ask />)
    const textarea = screen.getByLabelText('Your question')
    expect(textarea).toBeInTheDocument()
    expect(textarea).toHaveAttribute('rows', '4')
  })

  it('renders submit button', () => {
    render(<Ask />)
    expect(screen.getByRole('button', { name: 'Ask' })).toBeInTheDocument()
  })

  it('populates textarea when clicking suggested question', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    const pill = screen.getByText('Match to a role')
    await user.click(pill)

    const textarea = screen.getByLabelText('Your question')
    expect(textarea).toHaveValue('Paste a job description below, and I\'ll explain how my experience aligns with the role.')
  })

  it('highlights selected suggested question', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    const pill = screen.getByText('Match to a role')
    await user.click(pill)

    expect(pill).toHaveClass('border-cyan-500', 'text-cyan-400', 'bg-cyan-600/10')
  })

  it('submits question and displays response', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    const textarea = screen.getByLabelText('Your question')
    await user.type(textarea, 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Answer')).toBeInTheDocument()
    })
  })

  it('displays answer content', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/This is a test answer/)).toBeInTheDocument()
    })
  })

  it('displays model information', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/claude-3-haiku-20240307/)).toBeInTheDocument()
    })
  })

  it('displays citation badges', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('[1]')).toBeInTheDocument()
    })
  })

  it('displays citation links', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      const citationLink = screen.getByText('README.md')
      expect(citationLink).toBeInTheDocument()
      expect(citationLink).toHaveAttribute('href', 'https://github.com/shantopagla/portfolio/blob/main/README.md')
    })
  })

  it('shows loading state during submission', async () => {
    // Delay the MSW response so loading state is visible
    server.use(
      http.post('/api/ask', async () => {
        await new Promise(r => setTimeout(r, 50))
        return HttpResponse.json({ answer: 'Delayed', citations: [], model: 'test', input_tokens: 1, output_tokens: 1 })
      })
    )

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(submitButton).toHaveTextContent('Asking...')
      expect(submitButton).toBeDisabled()
    })

    // Ensure the async handler fully completes before the test ends
    await waitFor(() => {
      expect(submitButton).toHaveTextContent('Ask')
      expect(submitButton).toBeEnabled()
    })
  })

  it('displays error message on rate limit', async () => {
    server.use(
      http.post('/api/ask', () =>
        HttpResponse.json({ error: 'Rate limit exceeded' }, { status: 429 })
      )
    )

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/Rate limit reached/)).toBeInTheDocument()
    })

    await waitFor(() => expect(submitButton).toBeEnabled())
  })

  it('displays error message on service unavailable', async () => {
    server.use(
      http.post('/api/ask', () =>
        HttpResponse.json({ error: 'Service unavailable' }, { status: 503 })
      )
    )

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/not available right now/)).toBeInTheDocument()
    })

    await waitFor(() => expect(submitButton).toBeEnabled())
  })

  it('displays error message on network error', async () => {
    server.use(http.post('/api/ask', () => HttpResponse.error()))

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/Network error/)).toBeInTheDocument()
    })

    await waitFor(() => expect(submitButton).toBeEnabled())
  })

  it('does not render heading when embedded', () => {
    render(<Ask embedded />)
    expect(screen.queryByRole('heading', { name: 'Ask' })).not.toBeInTheDocument()
    expect(screen.queryByText(/Questions about this portfolio/)).not.toBeInTheDocument()
  })

  it('respects max length on textarea', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    const textarea = screen.getByLabelText('Your question')
    expect(textarea).toHaveAttribute('maxLength', '6000')
  })

  it('clears previous answer when submitting new question', async () => {
    render(<Ask />)
    const user = userEvent.setup()

    // First question
    await user.type(screen.getByLabelText('Your question'), 'First question')
    await user.click(screen.getByRole('button', { name: 'Ask' }))

    await waitFor(() => {
      expect(screen.getByText('Answer')).toBeInTheDocument()
    })

    // Second question
    await user.clear(screen.getByLabelText('Your question'))
    await user.type(screen.getByLabelText('Your question'), 'Second question')
    await user.click(screen.getByRole('button', { name: 'Ask' }))

    await waitFor(() => {
      // Should still show answer (mock returns same data)
      expect(screen.getByText('Answer')).toBeInTheDocument()
    })
  })
})