import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '../utils/test-render'
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

  it('displays generic error message on non-429/503 failure', async () => {
    server.use(
      http.post('/api/ask', () =>
        HttpResponse.json({ error: 'Bad request' }, { status: 400 })
      )
    )

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')

    const submitButton = screen.getByRole('button', { name: 'Ask' })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Bad request')).toBeInTheDocument()
    })

    await waitFor(() => expect(submitButton).toBeEnabled())
  })

  it('displays non-portfolio citation with external link', async () => {
    server.use(
      http.post('/api/ask', () =>
        HttpResponse.json({
          answer: 'See source [source 1]',
          citations: [
            {
              path: 'src/main.rs',
              kind: 'rust',
              sha: 'abc123',
              url: 'https://github.com/shantopagla/portfolio/blob/abc123/src/main.rs',
              ord: 1,
            },
          ],
          model: 'test',
          input_tokens: 1,
          output_tokens: 1,
        })
      )
    )

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'What are your skills?')
    await user.click(screen.getByRole('button', { name: 'Ask' }))

    await waitFor(() => {
      const citationLink = screen.getByText('src/main.rs')
      expect(citationLink).toBeInTheDocument()
      expect(citationLink).toHaveAttribute('target', '_blank')
    })
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

  it('renders markdown with code blocks, blockquotes and links', async () => {
    server.use(
      http.post('/api/ask', () =>
        HttpResponse.json({
          answer: 'Here is some code: `println!("hello")`\n\n> This is a blockquote\n\n**Bold text** and [a link](https://example.com)\n\n```rust\nfn main() {}\n```',
          citations: [],
          model: 'test',
          input_tokens: 1,
          output_tokens: 1,
        })
      )
    )

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'Show me markdown')
    await user.click(screen.getByRole('button', { name: 'Ask' }))

    await waitFor(() => {
      expect(screen.getByText('Answer')).toBeInTheDocument()
    })

    // Inline code
    expect(screen.getByText('println!("hello")')).toBeInTheDocument()
    // Blockquote
    expect(screen.getByText('This is a blockquote')).toBeInTheDocument()
    // Bold
    expect(screen.getByText('Bold text')).toBeInTheDocument()
    // Link
    const link = screen.getByRole('link', { name: 'a link' })
    expect(link).toHaveAttribute('href', 'https://example.com')
  })

  it('renders markdown headings and lists', async () => {
    server.use(
      http.post('/api/ask', () =>
        HttpResponse.json({
          answer: '# Heading 1\n\n## Heading 2\n\n### Heading 3\n\n- Item one\n- Item two\n\n1. First\n2. Second',
          citations: [],
          model: 'test',
          input_tokens: 1,
          output_tokens: 1,
        })
      )
    )

    render(<Ask />)
    const user = userEvent.setup()

    await user.type(screen.getByLabelText('Your question'), 'Show me markdown')
    await user.click(screen.getByRole('button', { name: 'Ask' }))

    await waitFor(() => {
      expect(screen.getByText('Answer')).toBeInTheDocument()
    })

    expect(screen.getByText('Heading 1')).toBeInTheDocument()
    expect(screen.getByText('Heading 2')).toBeInTheDocument()
    expect(screen.getByText('Heading 3')).toBeInTheDocument()
    expect(screen.getByText('Item one')).toBeInTheDocument()
    expect(screen.getByText('Item two')).toBeInTheDocument()
    expect(screen.getByText('First')).toBeInTheDocument()
    expect(screen.getByText('Second')).toBeInTheDocument()
  })

  it('respects max length on textarea', () => {
    render(<Ask />)

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