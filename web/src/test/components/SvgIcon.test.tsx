import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import SvgIcon from '../../components/SvgIcon'

describe('SvgIcon', () => {
  it('renders icon with correct path', () => {
    render(<SvgIcon name="user" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toBeInTheDocument()
    expect(svg).toHaveAttribute('viewBox', '0 0 20 20')
  })

  it('applies custom className', () => {
    render(<SvgIcon name="user" className="w-10 h-10" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toHaveClass('w-10', 'h-10')
  })

  it('uses default className when not provided', () => {
    render(<SvgIcon name="user" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toHaveClass('w-4', 'h-4')
  })

  it('renders filled icon for most icons', () => {
    render(<SvgIcon name="user" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toHaveAttribute('fill', 'currentColor')
    expect(svg).toHaveAttribute('stroke', 'none')
  })

  it('renders outline icon for brain', () => {
    render(<SvgIcon name="brain" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toHaveAttribute('fill', 'none')
    expect(svg).toHaveAttribute('stroke', 'currentColor')
  })

  it('renders outline icon for diamond', () => {
    render(<SvgIcon name="diamond" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toHaveAttribute('fill', 'none')
    expect(svg).toHaveAttribute('stroke', 'currentColor')
  })

  it('falls back to diamond icon for unknown name', () => {
    render(<SvgIcon name="unknown-icon" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toBeInTheDocument()
    // Should use diamond as fallback
    expect(svg).toHaveAttribute('fill', 'none')
  })

  it('sets aria-hidden to true', () => {
    render(<SvgIcon name="user" />)
    const svg = screen.getByRole('img', { hidden: true })
    expect(svg).toHaveAttribute('aria-hidden', 'true')
  })
})