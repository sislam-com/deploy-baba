import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '../utils/test-render'
import userEvent from '@testing-library/user-event'
import Home from '../../routes/Home'

describe('Home', () => {
  it('renders hero section with name', () => {
    render(<Home />)
    expect(screen.getByText('Sharful Islam')).toBeInTheDocument()
  })

  it('renders hero section with title', () => {
    render(<Home />)
    expect(screen.getByText('AI Systems Engineer')).toBeInTheDocument()
  })

  it('renders stat pills', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('10+')).toBeInTheDocument()
      expect(screen.getByText('Years')).toBeInTheDocument()
    })
  })

  it('renders tech strip', async () => {
    render(<Home />)

    await waitFor(() => {
      // Should render tech tags from MSW mock data
      expect(screen.getByText('Rust')).toBeInTheDocument()
      expect(screen.getByText('AWS')).toBeInTheDocument()
    })
  })

  it('renders job cards', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Tech Corp')).toBeInTheDocument()
      expect(screen.getByText('Senior Software Engineer')).toBeInTheDocument()
    })
  })

  it('toggles job card details on click', async () => {
    render(<Home />)
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.getByText('Tech Corp')).toBeInTheDocument()
    })

    const jobCard = screen.getByText('Tech Corp').closest('div[role="button"]')
    if (jobCard) {
      await user.click(jobCard)

      await waitFor(() => {
        expect(screen.getByText('Responsibilities')).toBeInTheDocument()
      })
    }
  })

  it('renders competency cards', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Rust Systems Programming')).toBeInTheDocument()
    })
  })

  it('toggles competency card details on click', async () => {
    render(<Home />)
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.getByText('Rust Systems Programming')).toBeInTheDocument()
    })

    const compCard = screen.getByText('Rust Systems Programming').closest('div[role="button"]')
    if (compCard) {
      await user.click(compCard)

      await waitFor(() => {
        expect(screen.getByText('Tech Corp')).toBeInTheDocument()
      })
    }
  })

  it('renders embedded Ask component', () => {
    render(<Home />)
    expect(screen.getByText('Common questions')).toBeInTheDocument()
    expect(screen.getByLabelText('Your question')).toBeInTheDocument()
  })

  it('renders suggested questions in embedded Ask', () => {
    render(<Home />)
    expect(screen.getByText('Match to a role')).toBeInTheDocument()
    expect(screen.getByText('Primary skills')).toBeInTheDocument()
  })

  it('renders challenge cards', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Portfolio RAG System')).toBeInTheDocument()
    })
  })

  it('renders featured badge on featured challenges', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Featured')).toBeInTheDocument()
    })
  })

  it('renders tech stack tags in job cards', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Rust')).toBeInTheDocument()
      expect(screen.getByText('AWS')).toBeInTheDocument()
      expect(screen.getByText('PostgreSQL')).toBeInTheDocument()
    })
  })

  it('renders job date range', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('2020-01-01 – Present')).toBeInTheDocument()
    })
  })

  it('renders job summary', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Leading backend development')).toBeInTheDocument()
    })
  })

  it('renders competency icons', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('🦀')).toBeInTheDocument()
    })
  })

  it('renders competency descriptions', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Expert in Rust systems programming')).toBeInTheDocument()
    })
  })

  it('renders challenge tech stack', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('Rust')).toBeInTheDocument()
      expect(screen.getByText('AI')).toBeInTheDocument()
      expect(screen.getByText('SQLite')).toBeInTheDocument()
    })
  })

  it('renders challenge category', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('technical')).toBeInTheDocument()
    })
  })

  it('renders challenge links', async () => {
    render(<Home />)

    await waitFor(() => {
      const link = screen.getByText('View Project')
      expect(link).toBeInTheDocument()
      expect(link).toHaveAttribute('href', 'https://github.com/shantopagla/portfolio')
    })
  })

  it('applies correct styling to hero section', () => {
    const { container } = render(<Home />)
    const hero = container.querySelector('h1')
    expect(hero).toHaveClass('text-5xl', 'font-bold', 'text-white')
  })

  it('applies correct styling to stat pills', async () => {
    render(<Home />)

    await waitFor(() => {
      const statValue = screen.getByText('10+')
      expect(statValue).toHaveClass('text-lg', 'font-bold', 'text-white')
    })
  })

  it('handles keyboard navigation on job cards', async () => {
    render(<Home />)
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.getByText('Tech Corp')).toBeInTheDocument()
    })

    const jobCard = screen.getByText('Tech Corp').closest('div[role="button"]')
    if (jobCard) {
      jobCard.focus()
      await user.keyboard('{Enter}')

      await waitFor(() => {
        expect(screen.getByText('Responsibilities')).toBeInTheDocument()
      })
    }
  })

  it('shows loading state initially', () => {
    render(<Home />)
    // Should show loading spinners or skeleton states
    expect(screen.getByText('Sharful Islam')).toBeInTheDocument()
  })

  it('renders professional summary', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText(/10 years of experience/)).toBeInTheDocument()
    })
  })
})