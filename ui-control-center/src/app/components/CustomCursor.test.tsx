import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import CustomCursor from './CustomCursor';

beforeEach(() => {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
});

describe('CustomCursor', () => {
  it('renders without crashing', () => {
    const { container } = render(<CustomCursor />);
    expect(container).toBeTruthy();
  });

  it('renders cursor dot and ring', () => {
    const { container } = render(<CustomCursor />);
    const divs = container.querySelectorAll('.rounded-full');
    expect(divs.length).toBeGreaterThanOrEqual(2);
  });
});
