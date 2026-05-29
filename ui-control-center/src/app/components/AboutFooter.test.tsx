import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { About, Footer } from './AboutFooter';

describe('About', () => {
  it('renders the about heading', () => {
    render(<About />);
    expect(screen.getByText('The Engineer Behind the Infrastructure')).toBeInTheDocument();
  });

  it('renders expertise tags', () => {
    render(<About />);
    expect(screen.getByText('Rust')).toBeInTheDocument();
    expect(screen.getByText('Linux Kernel')).toBeInTheDocument();
    expect(screen.getByText('Distributed Systems')).toBeInTheDocument();
  });

  it('renders contact links', () => {
    render(<About />);
    const githubLink = screen.getByText('GitHub');
    expect(githubLink.closest('a')).toHaveAttribute('href', 'https://github.com/keaganryanoconnell');

    const linkedinLink = screen.getByText('LinkedIn');
    expect(linkedinLink.closest('a')).toHaveAttribute('href', 'https://linkedin.com/in/keaganryanoconnell');

    const emailLink = screen.getByText('Email');
    expect(emailLink.closest('a')).toHaveAttribute('href', 'mailto:keaganryanoconnell@gmail.com');
  });

  it('renders project summary list', () => {
    render(<About />);
    expect(screen.getByText(/Container Runtime/)).toBeInTheDocument();
    expect(screen.getByText(/Distributed Log Broker/)).toBeInTheDocument();
    expect(screen.getByText(/Raft Distributed KV/)).toBeInTheDocument();
  });
});

describe('Footer', () => {
  it('renders copyright', () => {
    render(<Footer />);
    const year = new Date().getFullYear();
    expect(screen.getByText(new RegExp(String(year)))).toBeInTheDocument();
  });

  it('renders back to top button', () => {
    render(<Footer />);
    expect(screen.getByText('Back to top')).toBeInTheDocument();
  });
});
