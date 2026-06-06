import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { BrowserRouter, MemoryRouter } from 'react-router-dom'
import type React from 'react'
import { HelmetProvider } from 'react-helmet-async'
import CoverLetter from '../../routes/CoverLetter'
import { useAgentStream } from '../../hooks/useAgentStream'

// Mock the hook
vi.mock('../../hooks/useAgentStream', () => ({
  useAgentStream: vi.fn(),
}))

describe('CoverLetter', () => {
  const mockGenerate = vi.fn()
  const mockCancel = vi.fn()

  function renderWithProviders(ui: React.ReactElement) {
    return render(
      <HelmetProvider>
        <BrowserRouter>
          {ui}
        </BrowserRouter>
      </HelmetProvider>
    )
  }

  beforeEach(() => {
    vi.resetAllMocks()
    // Default mock state - all agents pending
    ;(useAgentStream as any).mockReturnValue({
      agents: [
        { name: 'cover_letter_writer', label: 'Cover Letter Writer', description: 'Test', status: 'pending' },
        { name: 'pdf_uploader', label: 'PDF Converter', description: 'Test', status: 'pending' },
        { name: 'link_generator', label: 'Link Generator', description: 'Test', status: 'pending' },
      ],
      result: null,
      error: null,
      isStreaming: false,
      generate: mockGenerate,
      cancel: mockCancel,
      reset: vi.fn(),
    })
  })

  it('renders the page title and description', () => {
    renderWithProviders(<CoverLetter />)

    expect(screen.getByText('AI Cover Letter Generator')).toBeInTheDocument()
    expect(screen.getByText(/Paste a job description/)).toBeInTheDocument()
  })

  it('renders the job description textarea', () => {
    renderWithProviders(<CoverLetter />)

    expect(screen.getByLabelText('Job Description')).toBeInTheDocument()
  })

  it('shows agent cards in initial pending state', () => {
    renderWithProviders(<CoverLetter />)

    expect(screen.getByText('Cover Letter Writer')).toBeInTheDocument()
    expect(screen.getByText('PDF Converter')).toBeInTheDocument()
    expect(screen.getByText('Link Generator')).toBeInTheDocument()
  })

  it('shows initial workflow placeholder when not streaming', () => {
    renderWithProviders(<CoverLetter />)

    expect(screen.getByText('Multi-Agent Workflow')).toBeInTheDocument()
  })

  it('handles form submission', async () => {
    renderWithProviders(<CoverLetter />)

    const textarea = screen.getByLabelText('Job Description')
    fireEvent.change(textarea, { target: { value: 'a'.repeat(100) } })

    const submitButton = screen.getByText('Generate Cover Letter')
    fireEvent.click(submitButton)

    await waitFor(() => {
      expect(mockGenerate).toHaveBeenCalledWith('a'.repeat(100))
    })
  })

  it('disables submit when job description is too short', () => {
    renderWithProviders(<CoverLetter />)

    const textarea = screen.getByLabelText('Job Description')
    fireEvent.change(textarea, { target: { value: 'short' } })

    const submitButton = screen.getByText('Generate Cover Letter')
    expect(submitButton).toBeDisabled()
  })

  it('shows streaming state when isStreaming is true', () => {
    ;(useAgentStream as any).mockReturnValue({
      agents: [
        { name: 'cover_letter_writer', label: 'Cover Letter Writer', description: 'Test', status: 'running', detail: 'Working...' },
        { name: 'pdf_uploader', label: 'PDF Converter', description: 'Test', status: 'pending' },
        { name: 'link_generator', label: 'Link Generator', description: 'Test', status: 'pending' },
      ],
      result: null,
      error: null,
      isStreaming: true,
      generate: mockGenerate,
      cancel: mockCancel,
      reset: vi.fn(),
    })

    renderWithProviders(<CoverLetter />)

    expect(screen.getByText('Generating...')).toBeInTheDocument()
    expect(screen.getByText('Cancel')).toBeInTheDocument()
    expect(screen.getByText('Working...')).toBeInTheDocument()
  })

  it('shows result when available', () => {
    ;(useAgentStream as any).mockReturnValue({
      agents: [
        { name: 'cover_letter_writer', label: 'Cover Letter Writer', description: 'Test', status: 'completed' },
        { name: 'pdf_uploader', label: 'PDF Converter', description: 'Test', status: 'completed' },
        { name: 'link_generator', label: 'Link Generator', description: 'Test', status: 'completed' },
      ],
      result: {
        download_url: 'https://test.com/download.pdf',
        preview_html: '<p>Test cover letter</p>',
        summary: 'Generated successfully',
      },
      error: null,
      isStreaming: false,
      generate: mockGenerate,
      cancel: mockCancel,
      reset: vi.fn(),
    })

    renderWithProviders(<CoverLetter />)

    expect(screen.getByText('Cover Letter Preview')).toBeInTheDocument()
    expect(screen.getByText('Download PDF')).toBeInTheDocument()
  })

  it('shows error when error occurs', () => {
    ;(useAgentStream as any).mockReturnValue({
      agents: [
        { name: 'cover_letter_writer', label: 'Cover Letter Writer', description: 'Test', status: 'failed' },
        { name: 'pdf_uploader', label: 'PDF Converter', description: 'Test', status: 'pending' },
        { name: 'link_generator', label: 'Link Generator', description: 'Test', status: 'pending' },
      ],
      result: null,
      error: 'Something went wrong',
      isStreaming: false,
      generate: mockGenerate,
      cancel: mockCancel,
      reset: vi.fn(),
    })

    renderWithProviders(<CoverLetter />)

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
  })

  it('calls cancel when cancel button is clicked', async () => {
    ;(useAgentStream as any).mockReturnValue({
      agents: [
        { name: 'cover_letter_writer', label: 'Cover Letter Writer', description: 'Test', status: 'running' },
        { name: 'pdf_uploader', label: 'PDF Converter', description: 'Test', status: 'pending' },
        { name: 'link_generator', label: 'Link Generator', description: 'Test', status: 'pending' },
      ],
      result: null,
      error: null,
      isStreaming: true,
      generate: mockGenerate,
      cancel: mockCancel,
      reset: vi.fn(),
    })

    renderWithProviders(<CoverLetter />)

    const cancelButton = screen.getByText('Cancel')
    fireEvent.click(cancelButton)

    expect(mockCancel).toHaveBeenCalled()
  })

  it('shows character count', () => {
    renderWithProviders(<CoverLetter />)

    const textarea = screen.getByLabelText('Job Description')
    fireEvent.change(textarea, { target: { value: 'a'.repeat(50) } })

    expect(screen.getByText('50 / 10,000 characters')).toBeInTheDocument()
  })

  it('pre-fills textarea and auto-generates when navigated with location state', async () => {
    const jd = 'a'.repeat(100)
    render(
      <HelmetProvider>
        <MemoryRouter initialEntries={[{ pathname: '/cover-letter', state: { jobDescription: jd } }]}>
          <CoverLetter />
        </MemoryRouter>
      </HelmetProvider>
    )

    const textarea = screen.getByLabelText('Job Description')
    expect(textarea).toHaveValue(jd)

    await waitFor(() => {
      expect(mockGenerate).toHaveBeenCalledWith(jd)
    })
  })

  it('does not auto-generate when location state has short JD', () => {
    render(
      <HelmetProvider>
        <MemoryRouter initialEntries={[{ pathname: '/cover-letter', state: { jobDescription: 'short' } }]}>
          <CoverLetter />
        </MemoryRouter>
      </HelmetProvider>
    )

    expect(mockGenerate).not.toHaveBeenCalled()
  })
})
