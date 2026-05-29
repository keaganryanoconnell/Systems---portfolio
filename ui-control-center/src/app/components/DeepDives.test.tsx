import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import DeepDives from './DeepDives';

describe('DeepDives', () => {
  it('renders the section heading', () => {
    render(<DeepDives />);
    expect(screen.getByText('How It Works Under the Hood')).toBeInTheDocument();
  });

  it('renders all 4 deep dive titles', () => {
    render(<DeepDives />);
    expect(screen.getByText(/Building a Container Runtime/)).toBeInTheDocument();
    expect(screen.getByText(/The Binary Protocol/)).toBeInTheDocument();
    expect(screen.getByText(/Lock-Free Ring Buffers/)).toBeInTheDocument();
    expect(screen.getByText(/Seccomp-BPF/)).toBeInTheDocument();
  });

  it('expands content when clicked', async () => {
    render(<DeepDives />);
    const firstTitle = screen.getByText(/Building a Container Runtime/);
    fireEvent.click(firstTitle);

    await waitFor(() => {
      expect(screen.getByText(/clone\(\) syscall with 5 namespace flags/)).toBeInTheDocument();
    });
  });

  it('collapses content when clicked again', async () => {
    render(<DeepDives />);
    const title = screen.getByText(/Building a Container Runtime/);
    fireEvent.click(title);
    fireEvent.click(title);

    await waitFor(() => {
      expect(screen.queryByText(/clone\(\) syscall with 5 namespace flags/)).not.toBeInTheDocument();
    });
  });

  it('renders all summaries', () => {
    render(<DeepDives />);
    expect(screen.getByText(/How I implemented namespace isolation/)).toBeInTheDocument();
    expect(screen.getByText(/Design decisions behind the zero-copy/)).toBeInTheDocument();
    expect(screen.getByText(/How the SPSC queue in core-sys/)).toBeInTheDocument();
    expect(screen.getByText(/How the container engine uses Berkeley Packet Filter/)).toBeInTheDocument();
  });
});
