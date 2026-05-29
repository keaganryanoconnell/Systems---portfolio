import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import MemoryMapPanel from './MemoryMapPanel';

describe('MemoryMapPanel', () => {
  it('renders the MEMORY MAP heading', () => {
    render(<MemoryMapPanel />);
    expect(screen.getByText('MEMORY MAP')).toBeInTheDocument();
  });

  it('renders the SharedArrayBuffer size label', () => {
    render(<MemoryMapPanel />);
    expect(screen.getByText(/SharedArrayBuffer/)).toBeInTheDocument();
    expect(screen.getByText(/128MB/)).toBeInTheDocument();
  });

  it('renders all 4 memory regions', () => {
    render(<MemoryMapPanel />);
    expect(screen.getByText('CONTROL RING')).toBeInTheDocument();
    expect(screen.getAllByText('INGEST BUFFER').length).toBe(2);
    expect(screen.getAllByText('WASM HEAP').length).toBe(2);
    expect(screen.getAllByText('RESULT BUFFER').length).toBe(2);
  });

  it('renders the stats grid', () => {
    render(<MemoryMapPanel />);
    expect(screen.getByText('CACHE LINE')).toBeInTheDocument();
    expect(screen.getByText('ALIGNMENT')).toBeInTheDocument();
    expect(screen.getByText('PAGES')).toBeInTheDocument();
    expect(screen.getByText('WORKERS')).toBeInTheDocument();
  });

  it('renders byte offset hex values for each region', () => {
    render(<MemoryMapPanel />);
    expect(screen.getByText('0x00000000')).toBeInTheDocument();
  });
});
