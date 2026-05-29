import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import ArchMap from './ArchMap';

beforeEach(() => {
  class MockResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
  vi.stubGlobal('ResizeObserver', MockResizeObserver);
});

describe('ArchMap', () => {
  it('renders the section heading', () => {
    render(<ArchMap />);
    expect(screen.getByText('System Architecture')).toBeInTheDocument();
  });

  it('renders the title', () => {
    render(<ArchMap />);
    expect(screen.getByText('How Everything Connects')).toBeInTheDocument();
  });

  it('renders the description with crate count', () => {
    render(<ArchMap />);
    expect(screen.getByText(/Twenty crates working together/)).toBeInTheDocument();
  });

  it('renders the canvas element', () => {
    const { container } = render(<ArchMap />);
    const canvas = container.querySelector('canvas');
    expect(canvas).not.toBeNull();
  });

  it('renders all key node labels in the legend', () => {
    render(<ArchMap />);
    expect(screen.getByText('Control Center')).toBeInTheDocument();
    expect(screen.getByText('API Gateway')).toBeInTheDocument();
    expect(screen.getByText('Container Engine')).toBeInTheDocument();
    expect(screen.getByText('Raft KV')).toBeInTheDocument();
    expect(screen.getByText('Render Engine')).toBeInTheDocument();
    expect(screen.getByText('Columnar Engine')).toBeInTheDocument();
    expect(screen.getByText('CRDT Engine')).toBeInTheDocument();
    expect(screen.getByText('Ingestion Server')).toBeInTheDocument();
    expect(screen.getByText('Sync Server')).toBeInTheDocument();
  });
});
