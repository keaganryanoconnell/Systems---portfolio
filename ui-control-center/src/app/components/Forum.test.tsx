import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import Forum from './Forum';

describe('Forum', () => {
  it('renders the forum heading', () => {
    render(<Forum />);
    expect(screen.getByText('Engineering Discussions')).toBeInTheDocument();
  });

  it('renders discussion threads', () => {
    render(<Forum />);
    expect(screen.getByText(/Node 2 keeps crashing/)).toBeInTheDocument();
    expect(screen.getByText(/LSM compaction tuning/)).toBeInTheDocument();
    expect(screen.getByText(/Container engine seccomp/)).toBeInTheDocument();
  });

  it('renders category filter buttons', () => {
    render(<Forum />);
    expect(screen.getByText('All')).toBeInTheDocument();
    expect(screen.getByText('debugging')).toBeInTheDocument();
    expect(screen.getByText('performance')).toBeInTheDocument();
    expect(screen.getByText('security')).toBeInTheDocument();
  });

  it('renders search input', () => {
    render(<Forum />);
    expect(screen.getByPlaceholderText('Search discussions...')).toBeInTheDocument();
  });

  it('filters threads by search query', async () => {
    render(<Forum />);
    const searchInput = screen.getByPlaceholderText('Search discussions...');
    fireEvent.change(searchInput, { target: { value: 'seccomp' } });

    expect(screen.getByText(/Container engine seccomp/)).toBeInTheDocument();
    expect(screen.queryByText(/Node 2 keeps crashing/)).not.toBeInTheDocument();
  });

  it('opens thread detail view on click', () => {
    render(<Forum />);
    const threadTitle = screen.getByText(/Node 2 keeps crashing/);
    fireEvent.click(threadTitle);

    expect(screen.getByText(/Back to discussions/)).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Write a reply...')).toBeInTheDocument();
    expect(screen.getByText('Post Reply')).toBeInTheDocument();
  });

  it('renders New Discussion button', () => {
    render(<Forum />);
    expect(screen.getByText('New Discussion')).toBeInTheDocument();
  });
});
