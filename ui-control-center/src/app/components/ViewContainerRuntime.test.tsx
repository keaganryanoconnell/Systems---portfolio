import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import ViewContainerRuntime from './ViewContainerRuntime';

const defaultChaos = {
  partitionSplit: false,
  malformedFrames: false,
  crashNode2: false,
  fuzzerRunning: false,
};

describe('ViewContainerRuntime', () => {
  it('renders the container engine header', () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    expect(screen.getByText(/CONTAINER_ENGINE/)).toBeInTheDocument();
  });

  it('renders the create button', () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    expect(screen.getByText('+ CONTAINER.RUN')).toBeInTheDocument();
  });

  it('renders 5 initial containers', async () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    await waitFor(() => {
      expect(screen.getByText(/5 containers.*5 active/)).toBeInTheDocument();
    });
  });

  it('shows help panel when ? button is clicked', () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    const helpBtn = screen.getByText('?');
    fireEvent.click(helpBtn);
    expect(screen.getByText(/KEYBOARD SHORTCUTS/)).toBeInTheDocument();
  });

  it('creates a new container when + CONTAINER.RUN is clicked', async () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    const createBtn = screen.getByText('+ CONTAINER.RUN');
    fireEvent.click(createBtn);

    await waitFor(() => {
      const idElements = screen.getAllByText(/ctr-/);
      expect(idElements.length).toBeGreaterThanOrEqual(6);
    });
  });

  it('shows filter input', () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    expect(screen.getByPlaceholderText('Filter by ID, name, or state...')).toBeInTheDocument();
  });

  it('shows aggregate stats panel', () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    expect(screen.getByText('CLUSTER_AGGREGATE')).toBeInTheDocument();
  });

  it('shows events stream', () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    expect(screen.getByText(/EVENTS_STREAM/)).toBeInTheDocument();
  });

  it('shows detail panel prompt when nothing selected', () => {
    render(<ViewContainerRuntime chaosMode={defaultChaos} />);
    expect(screen.getByText('Select a container to view details')).toBeInTheDocument();
  });

  it('crashes db-replica container when crashNode2 chaos is active', () => {
    render(<ViewContainerRuntime chaosMode={{ ...defaultChaos, crashNode2: true }} />);
    const deadElements = screen.getAllByText('DEAD');
    expect(deadElements.length).toBeGreaterThanOrEqual(1);
  });
});
