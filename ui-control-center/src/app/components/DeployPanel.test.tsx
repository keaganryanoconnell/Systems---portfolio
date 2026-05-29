import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import DeployPanel from './DeployPanel';

describe('DeployPanel', () => {
  it('renders the DEPLOY TOPOLOGY heading', () => {
    render(<DeployPanel />);
    expect(screen.getByText('DEPLOY TOPOLOGY')).toBeInTheDocument();
  });

  it('renders docker compose command hint', () => {
    render(<DeployPanel />);
    expect(screen.getByText(/docker compose up/)).toBeInTheDocument();
    expect(screen.getByText(/12 services/)).toBeInTheDocument();
    expect(screen.getByText(/3-node Raft cluster/)).toBeInTheDocument();
  });

  it('renders all service names', () => {
    render(<DeployPanel />);
    expect(screen.getByText('api-gateway')).toBeInTheDocument();
    expect(screen.getByText('raft-kv-0')).toBeInTheDocument();
    expect(screen.getByText('raft-kv-1')).toBeInTheDocument();
    expect(screen.getByText('raft-kv-2')).toBeInTheDocument();
    expect(screen.getByText('log-broker')).toBeInTheDocument();
    expect(screen.getByText('compute-orchestrator')).toBeInTheDocument();
    expect(screen.getByText('container-engine')).toBeInTheDocument();
  });

  it('renders the summary stats grid', () => {
    render(<DeployPanel />);
    expect(screen.getByText('SERVICES')).toBeInTheDocument();
    expect(screen.getByText('HEALTHY')).toBeInTheDocument();
    expect(screen.getByText('STORAGE')).toBeInTheDocument();
    expect(screen.getByText('CMD')).toBeInTheDocument();
  });

  it('shows 12 services count', () => {
    render(<DeployPanel />);
    const serviceCounts = screen.getAllByText('12');
    expect(serviceCounts.length).toBeGreaterThan(0);
  });
});
