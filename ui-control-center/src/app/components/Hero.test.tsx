import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Hero from './Hero';

describe('Hero', () => {
  it('renders the title', () => {
    render(<Hero />);
    expect(screen.getByText('Principal Systems Engineer')).toBeInTheDocument();
  });

  it('renders the pitch line', () => {
    render(<Hero />);
    expect(screen.getByText(/Infrastructure is/)).toBeInTheDocument();
    expect(screen.getByText(/until it breaks/)).toBeInTheDocument();
  });

  it('renders the CTA button', () => {
    render(<Hero />);
    expect(screen.getByText('View Projects')).toBeInTheDocument();
  });

  it('renders social links', () => {
    render(<Hero />);
    const githubLink = screen.getByTitle('GitHub');
    expect(githubLink).toBeInTheDocument();
    expect(githubLink).toHaveAttribute('href', 'https://github.com/keaganryanoconnell');

    const linkedinLink = screen.getByTitle('LinkedIn');
    expect(linkedinLink).toBeInTheDocument();

    const emailLink = screen.getByTitle('Email');
    expect(emailLink).toBeInTheDocument();
  });

  it('renders availability badge', () => {
    render(<Hero />);
    expect(screen.getByText(/available for principal/)).toBeInTheDocument();
  });
});
