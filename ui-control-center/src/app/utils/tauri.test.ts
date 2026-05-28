import { describe, it, expect } from 'vitest';
import {
  encodeNodeTelemetry,
  decodeNodeTelemetry,
  generateMockNodesTelemetry,
  safeInvoke,
  type NodeTelemetry,
} from './tauri';

describe('encodeNodeTelemetry / decodeNodeTelemetry', () => {
  const sampleNodes: NodeTelemetry[] = [
    {
      nodeId: 1,
      role: 'Leader',
      status: 'Healthy',
      cpu: 42,
      arenaMemoryAllocated: 512,
      arenaMemoryTotal: 1024,
      activeFdPool: 64,
      replicationLag: 3,
      lsmStorageBytes: 123456789012n,
      iops: 15000,
    },
    {
      nodeId: 2,
      role: 'Follower',
      status: 'Degraded',
      cpu: 78,
      arenaMemoryAllocated: 256,
      arenaMemoryTotal: 512,
      activeFdPool: 32,
      replicationLag: 999,
      lsmStorageBytes: 987654321098n,
      iops: 85000,
    },
    {
      nodeId: 3,
      role: 'Candidate',
      status: 'Offline',
      cpu: 0,
      arenaMemoryAllocated: 0,
      arenaMemoryTotal: 0,
      activeFdPool: 0,
      replicationLag: 0,
      lsmStorageBytes: 0n,
      iops: 0,
    },
  ];

  it('roundtrips encode then decode with exact fidelity', () => {
    const encoded = encodeNodeTelemetry(sampleNodes);
    const decoded = decodeNodeTelemetry(encoded);

    expect(decoded).toHaveLength(sampleNodes.length);

    decoded.forEach((node, i) => {
      const original = sampleNodes[i];
      expect(node.nodeId).toBe(original.nodeId);
      expect(node.role).toBe(original.role);
      expect(node.status).toBe(original.status);
      expect(node.cpu).toBe(original.cpu);
      expect(node.arenaMemoryAllocated).toBe(original.arenaMemoryAllocated);
      expect(node.arenaMemoryTotal).toBe(original.arenaMemoryTotal);
      expect(node.activeFdPool).toBe(original.activeFdPool);
      expect(node.replicationLag).toBe(original.replicationLag);
      expect(node.lsmStorageBytes).toBe(original.lsmStorageBytes);
      expect(node.iops).toBe(original.iops);
    });
  });

  it('produces exactly 32 bytes per node', () => {
    const encoded = encodeNodeTelemetry(sampleNodes);
    expect(encoded.byteLength).toBe(sampleNodes.length * 32);
  });

  it('skips nodes with invalid magic bytes', () => {
    const garbage = new Uint8Array(64);
    garbage.fill(0xFF);
    const decoded = decodeNodeTelemetry(garbage);
    expect(decoded).toHaveLength(0);
  });

  it('clamps CPU values to 0-100 on encode', () => {
    const overflows: NodeTelemetry[] = [{
      nodeId: 1,
      role: 'Leader',
      status: 'Healthy',
      cpu: 200,
      arenaMemoryAllocated: 0,
      arenaMemoryTotal: 0,
      activeFdPool: 0,
      replicationLag: 0,
      lsmStorageBytes: 0n,
      iops: 0,
    }];
    const encoded = encodeNodeTelemetry(overflows);
    const decoded = decodeNodeTelemetry(encoded);
    expect(decoded[0].cpu).toBe(100);
  });

  it('handles partially filled buffer gracefully', () => {
    const partial = new Uint8Array(16);
    const dec = decodeNodeTelemetry(partial);
    expect(dec).toHaveLength(0);
  });
});

describe('generateMockNodesTelemetry', () => {
  it('generates exactly 5 nodes with normal mode', () => {
    const nodes = generateMockNodesTelemetry({
      partitionSplit: false,
      malformedFrames: false,
      crashNode2: false,
      fuzzerRunning: false,
    });
    expect(nodes).toHaveLength(5);
  });

  it('node 1 is Leader when no partition', () => {
    const nodes = generateMockNodesTelemetry({
      partitionSplit: false,
      malformedFrames: false,
      crashNode2: false,
      fuzzerRunning: false,
    });
    expect(nodes[0].role).toBe('Leader');
  });

  it('node 2 is Offline when crashNode2 is active', () => {
    const nodes = generateMockNodesTelemetry({
      partitionSplit: false,
      malformedFrames: false,
      crashNode2: true,
      fuzzerRunning: false,
    });
    expect(nodes[1].status).toBe('Offline');
    expect(nodes[1].cpu).toBe(0);
  });

  it('node 3 becomes Candidate during partition split', () => {
    const nodes = generateMockNodesTelemetry({
      partitionSplit: true,
      malformedFrames: false,
      crashNode2: false,
      fuzzerRunning: false,
    });
    expect(nodes[2].role).toBe('Candidate');
  });

  it('partitioned nodes have Degraded status', () => {
    const nodes = generateMockNodesTelemetry({
      partitionSplit: true,
      malformedFrames: false,
      crashNode2: false,
      fuzzerRunning: false,
    });
    expect(nodes[3].status).toBe('Degraded');
    expect(nodes[4].status).toBe('Degraded');
  });

  it('fuzzer spikes CPU and IOPS', () => {
    const nodes = generateMockNodesTelemetry({
      partitionSplit: false,
      malformedFrames: false,
      crashNode2: false,
      fuzzerRunning: true,
    });
    for (const n of nodes) {
      expect(n.iops).toBeGreaterThanOrEqual(80000);
    }
  });
});

describe('safeInvoke', () => {
  it('returns fallback when Tauri is not available', async () => {
    const result = await safeInvoke('get_telemetry', undefined, { healthy: true });
    expect(result).toEqual({ healthy: true });
  });

  it('returns fallback on invalid command', async () => {
    const result = await safeInvoke('nonexistent_command', { arg: 1 }, 'default');
    expect(result).toBe('default');
  });
});
