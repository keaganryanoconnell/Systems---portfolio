import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, renderHook, act } from '@testing-library/react';
import { EngineWorkerProvider, useWorkerEngine } from './EngineWorkerProvider';

beforeEach(() => {
  vi.restoreAllMocks();

  Object.defineProperty(globalThis, 'SharedArrayBuffer', {
    value: undefined,
    writable: true,
    configurable: true,
  });
  Object.defineProperty(globalThis, 'Atomics', {
    value: undefined,
    writable: true,
    configurable: true,
  });
});

function Wrapper({ children }: { children: React.ReactNode }) {
  return <EngineWorkerProvider>{children}</EngineWorkerProvider>;
}

function TestConsumer() {
  const ctx = useWorkerEngine();
  return (
    <div>
      <span data-testid="mode">{ctx.mode}</span>
      <span data-testid="sab-available">{String(ctx.sharedBufferAvailable)}</span>
      <span data-testid="active-workers">{ctx.activeWorkers}</span>
      <span data-testid="live-connecting">{String(ctx.liveConnecting)}</span>
      <button data-testid="toggle-btn" onClick={ctx.toggleMode}>Toggle</button>
    </div>
  );
}

describe('EngineWorkerProvider', () => {
  it('starts in sim mode by default', () => {
    const { getByTestId } = render(<TestConsumer />, { wrapper: Wrapper });
    expect(getByTestId('mode').textContent).toBe('sim');
  });

  it('detects SharedArrayBuffer unavailable in test environment', () => {
    const { getByTestId } = render(<TestConsumer />, { wrapper: Wrapper });
    expect(getByTestId('sab-available').textContent).toBe('false');
  });

  it('stays in sim mode when toggle is clicked without SAB', () => {
    const { getByTestId } = render(<TestConsumer />, { wrapper: Wrapper });
    const btn = getByTestId('toggle-btn');
    fireEvent.click(btn);
    expect(getByTestId('mode').textContent).toBe('sim');
  });

  it('provides initial heap values', () => {
    const { result } = renderHook(() => useWorkerEngine(), { wrapper: Wrapper });
    expect(result.current.heapUsed).toBe(180 * 1024 * 1024);
    expect(result.current.heapMax).toBe(256 * 1024 * 1024);
  });

  it('has 0 active workers in sim mode', () => {
    const { getByTestId } = render(<TestConsumer />, { wrapper: Wrapper });
    expect(getByTestId('active-workers').textContent).toBe('0');
  });

  it('is not in liveConnecting state in sim mode', () => {
    const { getByTestId } = render(<TestConsumer />, { wrapper: Wrapper });
    expect(getByTestId('live-connecting').textContent).toBe('false');
  });
});
