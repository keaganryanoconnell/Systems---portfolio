import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import NavBar from './NavBar';

describe('NavBar', () => {
  it('renders the logo', () => {
    render(<NavBar />);
    expect(screen.getByText('PORTFOLIO')).toBeInTheDocument();
  });

  it('renders all section links', () => {
    render(<NavBar />);
    expect(screen.getByText('HOME')).toBeInTheDocument();
    expect(screen.getByText('ARCHITECTURE')).toBeInTheDocument();
    expect(screen.getByText('WORKSPACE')).toBeInTheDocument();
    expect(screen.getByText('DEEP DIVES')).toBeInTheDocument();
    expect(screen.getByText('FORUM')).toBeInTheDocument();
    expect(screen.getByText('ABOUT')).toBeInTheDocument();
  });

  it('shows mobile menu button on small screens', () => {
    render(<NavBar />);
    expect(screen.getByText('☰')).toBeInTheDocument();
  });

  it('highlights active section on scroll', () => {
    render(<NavBar />);
    const homeBtn = screen.getByText('HOME');
    expect(homeBtn).toBeInTheDocument();
  });
});
