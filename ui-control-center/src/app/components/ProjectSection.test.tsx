import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import ProjectSection from './ProjectSection';

const sampleProps = {
  id: 'test',
  number: '01',
  title: 'Test Project',
  subtitle: 'A test project subtitle',
  description: 'This is a test project description.',
  highlights: ['Feature one', 'Feature two', 'Feature three'],
  tags: ['Rust', 'Linux', 'Security'],
  demo: <div data-testid="demo">Demo content</div>,
};

describe('ProjectSection', () => {
  it('renders the project number and title', () => {
    render(<ProjectSection {...sampleProps} />);
    expect(screen.getByText('01 — Project')).toBeInTheDocument();
    expect(screen.getByText('Test Project')).toBeInTheDocument();
  });

  it('renders the subtitle', () => {
    render(<ProjectSection {...sampleProps} />);
    expect(screen.getByText('A test project subtitle')).toBeInTheDocument();
  });

  it('renders the description', () => {
    render(<ProjectSection {...sampleProps} />);
    expect(screen.getByText('This is a test project description.')).toBeInTheDocument();
  });

  it('renders all highlights', () => {
    render(<ProjectSection {...sampleProps} />);
    expect(screen.getByText('Feature one')).toBeInTheDocument();
    expect(screen.getByText('Feature two')).toBeInTheDocument();
    expect(screen.getByText('Feature three')).toBeInTheDocument();
  });

  it('renders all tech tags', () => {
    render(<ProjectSection {...sampleProps} />);
    expect(screen.getByText('Rust')).toBeInTheDocument();
    expect(screen.getByText('Linux')).toBeInTheDocument();
    expect(screen.getByText('Security')).toBeInTheDocument();
  });

  it('renders the demo content', () => {
    render(<ProjectSection {...sampleProps} />);
    expect(screen.getByTestId('demo')).toBeInTheDocument();
    expect(screen.getByText('Demo content')).toBeInTheDocument();
  });

  it('renders GitHub link when provided', () => {
    render(<ProjectSection {...sampleProps} githubUrl="https://github.com/test/repo" />);
    expect(screen.getByText('View source →')).toBeInTheDocument();
  });

  it('does not render GitHub link when not provided', () => {
    render(<ProjectSection {...sampleProps} />);
    expect(screen.queryByText('View source →')).not.toBeInTheDocument();
  });
});
