import { describe, it, expect } from 'vitest'
import { render, screen } from '../utils/test-render'
import NotFound from '../../routes/NotFound'

describe('NotFound', () => {
  it('renders 404 heading', () => {
    render(<NotFound />, { router: 'memory', route: '/404' })
    const heading = screen.getByText('404')
    expect(heading).toBeInTheDocument()
  })

  it('renders page not found message', () => {
    render(<NotFound />, { router: 'memory', route: '/404' })
    const message = screen.getByText('Page not found.')
    expect(message).toBeInTheDocument()
  })

  it('renders back to home link', () => {
    render(<NotFound />, { router: 'memory', route: '/404' })
    const link = screen.getByText('← Back home')
    expect(link).toBeInTheDocument()
    expect(link).toHaveAttribute('href', '/')
  })

  it('applies correct styling classes', () => {
    const { container } = render(<NotFound />, { router: 'memory', route: '/404' })
    const containerDiv = container.querySelector('div')
    expect(containerDiv).toHaveClass('max-w-4xl', 'mx-auto', 'px-4', 'py-12', 'text-center')
  })
})