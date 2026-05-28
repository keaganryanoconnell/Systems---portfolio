"use client";

export interface NodeTelemetry {
  nodeId: number;
  role: 'Leader' | 'Follower' | 'Candidate';
  status: 'Healthy' | 'Degraded' | 'Offline';
  cpu: number; // 0-100
  arenaMemoryAllocated: number; // MB
  arenaMemoryTotal: number; // MB
  activeFdPool: number;
  replicationLag: number; // ms
  lsmStorageBytes: bigint;
  iops: number;
}

/**
 * Safely calls a Tauri IPC command, wrapping it in error-propagation boundaries
 * and falling back to a default value if not running in Tauri or if the backend fails.
 */
export async function safeInvoke<T>(command: string, args: Record<string, unknown> | undefined, fallback: T): Promise<T> {
  if (typeof window === "undefined") return fallback;
  try {
    // Check for Tauri globals
    const isTauri = 
      (window as any).__TAURI_INTERNALS__ !== undefined || 
      (window as any).__TAURI__ !== undefined;
      
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<T>(command, args);
    }
  } catch (error) {
    console.warn(`[Tauri IPC Fallback] Error invoking command '${command}'. Node offline or Web sandbox mode.`, error);
  }
  return fallback;
}

/**
 * Zero-Copy Binary Decode frontend parser.
 * Receives telemetry payloads as raw Uint8Array byte streams,
 * decoding metrics into state variables without heavy JSON serialization overhead.
 * 
 * Each Node entry is exactly 32 bytes:
 * - Bytes 0-3: Magic Bytes (0xAABBCCDD)
 * - Byte 4: Node ID (uint8)
 * - Byte 5: Role (uint8: 0=Leader, 1=Follower, 2=Candidate)
 * - Byte 6: Status (uint8: 0=Healthy, 1=Degraded, 2=Offline)
 * - Byte 7: CPU (uint8, 0-100)
 * - Bytes 8-11: Arena memory allocated (uint32)
 * - Bytes 12-15: Arena memory total (uint32)
 * - Bytes 16-17: Active file descriptors (uint16)
 * - Bytes 18-19: Replication lag (uint16, ms)
 * - Bytes 20-27: LSM storage bytes (uint64, big-endian)
 * - Bytes 28-31: IOPS (uint32)
 */
export function decodeNodeTelemetry(buffer: Uint8Array): NodeTelemetry[] {
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const nodes: NodeTelemetry[] = [];
  const nodeSize = 32;
  const numNodes = Math.floor(buffer.length / nodeSize);

  for (let i = 0; i < numNodes; i++) {
    const offset = i * nodeSize;
    
    // Verify magic bytes
    try {
      const magic = view.getUint32(offset);
      if (magic !== 0xAABBCCDD) {
        continue; // Invalid packet/misalignment, skip
      }
    } catch {
      break; // Index out of bounds
    }

    const nodeId = view.getUint8(offset + 4);
    
    const roleVal = view.getUint8(offset + 5);
    const role: NodeTelemetry['role'] = 
      roleVal === 0 ? 'Leader' : roleVal === 1 ? 'Follower' : 'Candidate';

    const statusVal = view.getUint8(offset + 6);
    const status: NodeTelemetry['status'] = 
      statusVal === 0 ? 'Healthy' : statusVal === 1 ? 'Degraded' : 'Offline';

    const cpu = view.getUint8(offset + 7);
    const arenaMemoryAllocated = view.getUint32(offset + 8);
    const arenaMemoryTotal = view.getUint32(offset + 12);
    const activeFdPool = view.getUint16(offset + 16);
    const replicationLag = view.getUint16(offset + 18);
    
    let lsmStorageBytes = 0n;
    try {
      lsmStorageBytes = view.getBigUint64(offset + 20);
    } catch {
      // Fallback if BigInt64 is not supported or misaligned
      const high = view.getUint32(offset + 20);
      const low = view.getUint32(offset + 24);
      lsmStorageBytes = (BigInt(high) << 32n) + BigInt(low);
    }
    
    const iops = view.getUint32(offset + 28);

    nodes.push({
      nodeId,
      role,
      status,
      cpu,
      arenaMemoryAllocated,
      arenaMemoryTotal,
      activeFdPool,
      replicationLag,
      lsmStorageBytes,
      iops
    });
  }

  return nodes;
}

/**
 * Encodes Node telemetry structure into raw binary bytes.
 * Used for testing, mock streams, and fallback high-performance simulations.
 */
export function encodeNodeTelemetry(nodes: NodeTelemetry[]): Uint8Array {
  const nodeSize = 32;
  const buffer = new Uint8Array(nodes.length * nodeSize);
  const view = new DataView(buffer.buffer);

  nodes.forEach((node, i) => {
    const offset = i * nodeSize;
    // Magic bytes
    view.setUint32(offset, 0xAABBCCDD);
    // Node ID
    view.setUint8(offset + 4, node.nodeId);
    // Role
    const roleVal = node.role === 'Leader' ? 0 : node.role === 'Follower' ? 1 : 2;
    view.setUint8(offset + 5, roleVal);
    // Status
    const statusVal = node.status === 'Healthy' ? 0 : node.status === 'Degraded' ? 1 : 2;
    view.setUint8(offset + 6, statusVal);
    // CPU
    view.setUint8(offset + 7, Math.min(100, Math.max(0, node.cpu)));
    // Arena allocated & total
    view.setUint32(offset + 8, node.arenaMemoryAllocated);
    view.setUint32(offset + 12, node.arenaMemoryTotal);
    // Active FD pool
    view.setUint16(offset + 16, node.activeFdPool);
    // Replication lag
    view.setUint16(offset + 18, node.replicationLag);
    // LSM storage bytes
    try {
      view.setBigUint64(offset + 20, node.lsmStorageBytes);
    } catch {
      const high = Number(node.lsmStorageBytes >> 32n);
      const low = Number(node.lsmStorageBytes & 0xFFFFFFFFn);
      view.setUint32(offset + 20, high);
      view.setUint32(offset + 24, low);
    }
    // IOPS
    view.setUint32(offset + 28, node.iops);
  });

  return buffer;
}

/**
 * Mock generator for system nodes telemetry (Leader & Followers)
 */
export function generateMockNodesTelemetry(chaosMode: {
  partitionSplit: boolean;
  malformedFrames: boolean;
  crashNode2: boolean;
  fuzzerRunning: boolean;
}): NodeTelemetry[] {
  const nodesCount = 5;
  const nodes: NodeTelemetry[] = [];

  for (let i = 1; i <= nodesCount; i++) {
    const isNode2Crashed = i === 2 && chaosMode.crashNode2;
    const isPartitioned = chaosMode.partitionSplit && i > 2; // Nodes 3, 4, 5 are partitioned from 1, 2
    
    let role: NodeTelemetry['role'] = 'Follower';
    if (i === 1) role = 'Leader';
    if (i === 3 && isPartitioned) role = 'Candidate'; // Split-brain election trigger
    
    let status: NodeTelemetry['status'] = 'Healthy';
    if (isNode2Crashed) status = 'Offline';
    else if (isPartitioned) status = 'Degraded';
    else if (chaosMode.fuzzerRunning && Math.random() > 0.7) status = 'Degraded';

    let cpu = 12 + (i * 4) + Math.floor(Math.random() * 8 - 4);
    if (isNode2Crashed) cpu = 0;
    else if (chaosMode.fuzzerRunning) cpu = 80 + Math.floor(Math.random() * 15);
    else if (status === 'Degraded') cpu = 45 + Math.floor(Math.random() * 10);

    let lag = isNode2Crashed ? 0 : i === 1 ? 0 : 2 + i * 3 + Math.floor(Math.random() * 4 - 2);
    if (isPartitioned && i > 2) lag = 999; // Extreme replication lag during partition

    let iops = isNode2Crashed ? 0 : 15000 + Math.floor(Math.random() * 2000 - 1000);
    if (chaosMode.fuzzerRunning && !isNode2Crashed) iops = 85000 + Math.floor(Math.random() * 5000);

    let lsmStorage = 124890000000n + BigInt(i * 1234500) + BigInt(Math.floor(Math.random() * 50000));
    if (isNode2Crashed) lsmStorage = 124891234500n; // Frozen

    nodes.push({
      nodeId: i,
      role,
      status,
      cpu,
      arenaMemoryAllocated: isNode2Crashed ? 0 : 142 + (i * 24) + Math.floor(Math.random() * 10 - 5),
      arenaMemoryTotal: isNode2Crashed ? 0 : 1024,
      activeFdPool: isNode2Crashed ? 0 : 48 + i * 4 + Math.floor(Math.random() * 4 - 2),
      replicationLag: lag,
      lsmStorageBytes: lsmStorage,
      iops
    });
  }

  return nodes;
}

/**
 * Fetches node telemetry from the Tauri backend or falls back to mock data.
 * Prioritizes native IPC when running inside the Tauri desktop shell.
 */
export async function fetchNodeTelemetry(
  chaosMode: { partitionSplit: boolean; malformedFrames: boolean; crashNode2: boolean; fuzzerRunning: boolean }
): Promise<NodeTelemetry[]> {
  if (typeof window === "undefined") return generateMockNodesTelemetry(chaosMode);

  try {
    const isTauri =
      (window as any).__TAURI_INTERNALS__ !== undefined ||
      (window as any).__TAURI__ !== undefined;

    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");

      // Sync chaos mode to backend so telemetry reflects current state
      await invoke("set_chaos_mode", {
        partitionSplit: chaosMode.partitionSplit,
        malformedFrames: chaosMode.malformedFrames,
        crashNode2: chaosMode.crashNode2,
        fuzzerRunning: chaosMode.fuzzerRunning,
      });

      // Fetch live telemetry from the Rust backend
      const raw = await invoke<any[]>("get_node_telemetry");

      // Map the backend's snake_case fields to frontend's camelCase interface
      return raw.map((n: any): NodeTelemetry => ({
        nodeId: n.node_id,
        role: n.role,
        status: n.status,
        cpu: n.cpu,
        arenaMemoryAllocated: n.arena_memory_allocated,
        arenaMemoryTotal: n.arena_memory_total,
        activeFdPool: n.active_fd_pool,
        replicationLag: n.replication_lag,
        lsmStorageBytes: typeof n.lsm_storage_bytes === 'bigint'
          ? n.lsm_storage_bytes
          : BigInt(n.lsm_storage_bytes),
        iops: n.iops,
      }));
    }
  } catch (error) {
    console.warn("[Tauri IPC] Failed to fetch telemetry, falling back to mock:", error);
  }

  return generateMockNodesTelemetry(chaosMode);
}
