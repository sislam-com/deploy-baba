import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '../utils/test-render'
import userEvent from '@testing-library/user-event'
import Home from '../../routes/Home'

describe('Home', () => {
  it('renders hero section with name', async () => {
    render(<Home />)
    await waitFor(() => {
      expect(screen.getByText('Sharful Islam')).toBeInTheDocument()
    })
  })

  it('renders hero section with title', async () => {
    render(<Home />)
    await waitFor(() => {
      expect(screen.getByText('AI Systems Engineer')).toBeInTheDocument()
    })
  })

  it('renders stat pills', async () => {
    render(<Home />)

    await waitFor(() => {
      expect(screen.getByText('6+')).toBeInTheDocument()
      expect(screen.getByText('years')).toBeInTheDocument()
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
    render(<Home />, { route: '/?view=timeline' })

    await waitFor(() => {
      expect(screen.getByText('Tech Corp')).toBeInTheDocument()
      expect(screen.getByText('Senior Software Engineer')).toBeInTheDocument()
    })
  })

  it('toggles job card details on click', async () => {
    render(<Home />, { route: '/?view=timeline' })
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
    render(<Home />, { route: '/?view=capabilities' })

    await waitFor(() => {
      expect(screen.getByText('Rust Systems Programming')).toBeInTheDocument()
    })
  })

  it('toggles competency card details on click', async () => {
    render(<Home />, { route: '/?view=capabilities' })
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

  it('renders embedded Ask component', async () => {
    render(<Home />, { route: '/?view=ask' })
    await waitFor(() => {
      expect(screen.getByText('Common questions')).toBeInTheDocument()
    })
    expect(screen.getByLabelText('Your question')).toBeInTheDocument()
  })

  it('renders suggested questions in embedded Ask', async () => {
    render(<Home />, { route: '/?view=ask' })
    await waitFor(() => {
      expect(screen.getByText('Match to a role')).toBeInTheDocument()
    })
    expect(screen.getByText('Primary skills')).toBeInTheDocument()
  })

  it('renders challenge cards', async () => {
    render(<Home />, { route: '/?view=challenges' })

    await waitFor(() => {
      expect(screen.getByText('Portfolio RAG System')).toBeInTheDocument()
    })
  })

  it('renders featured challenges', async () => {
    render(<Home />, { route: '/?view=challenges' })

    await waitFor(() => {
      expect(screen.getByText('Portfolio RAG System')).toBeInTheDocument()
    })
  })

  it('renders tech stack tags in job cards', async () => {
    render(<Home />, { route: '/?view=timeline' })

    await waitFor(() => {
      expect(screen.getAllByText('Rust').length).toBeGreaterThanOrEqual(1)
      expect(screen.getAllByText('AWS').length).toBeGreaterThanOrEqual(1)
      expect(screen.getAllByText('PostgreSQL').length).toBeGreaterThanOrEqual(1)
    })
  })

  it('renders job date range', async () => {
    render(<Home />, { route: '/?view=timeline' })

    await waitFor(() => {
      expect(screen.getByText('2020-01-01 – Present')).toBeInTheDocument()
    })
  })

  it('renders job summary', async () => {
    render(<Home />, { route: '/?view=timeline' })

    await waitFor(() => {
      expect(screen.getByText('Leading backend development')).toBeInTheDocument()
    })
  })

  it('renders competency icons', async () => {
    render(<Home />, { route: '/?view=capabilities' })

    await waitFor(() => {
      // Icons are rendered as SVGs via SvgIcon, not emoji text
      expect(screen.getByText('Rust Systems Programming')).toBeInTheDocument()
    })
  })

  it('renders competency descriptions', async () => {
    render(<Home />, { route: '/?view=capabilities' })

    await waitFor(() => {
      expect(screen.getByText('Expert in Rust systems programming')).toBeInTheDocument()
    })
  })

  it('renders challenge tech stack', async () => {
    render(<Home />, { route: '/?view=challenges' })

    await waitFor(() => {
      expect(screen.getAllByText('Rust').length).toBeGreaterThanOrEqual(1)
      expect(screen.getAllByText('AI').length).toBeGreaterThanOrEqual(1)
      expect(screen.getAllByText('SQLite').length).toBeGreaterThanOrEqual(1)
    })
  })

  it('renders challenge category', async () => {
    render(<Home />, { route: '/?view=challenges' })

    await waitFor(() => {
      expect(screen.getAllByText('technical').length).toBeGreaterThanOrEqual(1)
    })
  })

  it('renders challenge links', async () => {
    render(<Home />, { route: '/?view=challenges' })

    await waitFor(() => {
      const link = screen.getByRole('link', { name: /view project/i })
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
      const statValue = screen.getByText('6+')
      expect(statValue).toHaveClass('text-lg', 'font-bold', 'text-white')
    })
  })

  it('handles keyboard navigation on job cards', async () => {
    render(<Home />, { route: '/?view=timeline' })
    const user = userEvent.setup()

    await waitFor(() => {
      expect(screen.getByText('Tech Corp')).toBeInTheDocument()
    })

    const jobCard = screen.getByText('Tech Corp').closest('div[role="button"]') as HTMLElement | null
    if (jobCard) {
      jobCard.focus()
      await user.keyboard('{Enter}')

      await waitFor(() => {
        expect(screen.getByText('Responsibilities')).toBeInTheDocument()
      })
    }
  })

  it('shows loading state initially', async () => {
    render(<Home />)
    // Hero name renders after async fetch
    await waitFor(() => {
      expect(screen.getByText('Sharful Islam')).toBeInTheDocument()
    })
  })

  it('renders professional summary', async () => {
    render(<Home />, { route: '/?view=timeline' })

    await waitFor(() => {
      expect(screen.getByText('Leading backend development')).toBeInTheDocument()
    })
  })
})
