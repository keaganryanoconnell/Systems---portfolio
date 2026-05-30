import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import LiveWidgets from './LiveWidgets';

describe('LiveWidgets', () => {
  it('renders the section heading', () => {
    render(<LiveWidgets />);
    expect(screen.getByText('Live Data')).toBeInTheDocument();
  });

  it('renders the section title', () => {
    render(<LiveWidgets />);
    expect(screen.getByText('Real-Time Dashboard')).toBeInTheDocument();
  });

  it('renders GitHub widget heading', () => {
    render(<LiveWidgets />);
    expect(screen.getByText('GitHub Commits')).toBeInTheDocument();
  });

  it('renders Spotify widget heading', () => {
    render(<LiveWidgets />);
    expect(screen.getByText('Spotify')).toBeInTheDocument();
  });

  it('renders Local Dashboard widget heading', () => {
    render(<LiveWidgets />);
    expect(screen.getByText('Local Dashboard')).toBeInTheDocument();
  });
});
