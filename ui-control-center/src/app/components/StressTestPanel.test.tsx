import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import StressTestPanel from './StressTestPanel';

describe('StressTestPanel', () => {
  it('renders the STRESS TEST heading', () => {
    render(<StressTestPanel />);
    expect(screen.getByText('STRESS TEST')).toBeInTheDocument();
  });

  it('renders the FIRE button', () => {
    render(<StressTestPanel />);
    expect(screen.getByText('FIRE')).toBeInTheDocument();
  });

  it('renders concurrency slider label', () => {
    render(<StressTestPanel />);
    expect(screen.getByText('CONCURRENCY')).toBeInTheDocument();
  });

  it('renders query count presets', () => {
    render(<StressTestPanel />);
    expect(screen.getByText('100')).toBeInTheDocument();
    expect(screen.getByText('500')).toBeInTheDocument();
    expect(screen.getByText('1K')).toBeInTheDocument();
    expect(screen.getByText('5K')).toBeInTheDocument();
    expect(screen.getByText('10K')).toBeInTheDocument();
  });

  it('renders latency percentile labels', () => {
    render(<StressTestPanel />);
    expect(screen.getByText('LATENCY PERCENTILES')).toBeInTheDocument();
    expect(screen.getByText('p50')).toBeInTheDocument();
    expect(screen.getByText('p99')).toBeInTheDocument();
    expect(screen.getByText('p999')).toBeInTheDocument();
  });

  it('renders metrics grid labels', () => {
    render(<StressTestPanel />);
    expect(screen.getByText('DISPATCHED')).toBeInTheDocument();
    expect(screen.getByText('COMPLETED')).toBeInTheDocument();
    expect(screen.getByText('THROUGHPUT')).toBeInTheDocument();
    expect(screen.getByText('QUEUE')).toBeInTheDocument();
  });

  it('shows initial ready state log', () => {
    render(<StressTestPanel />);
    expect(screen.getByText(/READY/)).toBeInTheDocument();
  });

  it('starts stress test when FIRE is clicked', () => {
    render(<StressTestPanel />);
    const btn = screen.getByText('FIRE');
    fireEvent.click(btn);
    expect(screen.getByText('BURNING...')).toBeInTheDocument();
    expect(screen.getByText(/STRESS TEST STARTED/)).toBeInTheDocument();
  });
});
