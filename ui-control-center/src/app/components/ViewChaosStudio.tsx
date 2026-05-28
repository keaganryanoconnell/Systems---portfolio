"use client";

import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { FixedSizeList as List } from "react-window";

interface ViewChaosStudioProps {
  chaosMode: {
    partitionSplit: boolean;
    malformedFrames: boolean;
    crashNode2: boolean;
    fuzzerRunning: boolean;
  };
  setChaosMode: React.Dispatch<React.SetStateAction<{
    partitionSplit: boolean;
    malformedFrames: boolean;
    crashNode2: boolean;
    fuzzerRunning: boolean;
  }>>;
}

interface LogEntry {
  id: number;
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'SWIM' | 'EPOLL' | 'CHAOS' | 'TAURI';
  message: string;
}

const LEVEL_COLORS: Record<LogEntry['level'], string> = {
  INFO: 'text-text-soft',
  WARN: 'text-gold',
  ERROR: 'text-red',
  SWIM: 'text-green',
  EPOLL: 'text-blue',
  CHAOS: 'text-red-400',
  TAURI: 'text-purple-400',
};

const LEVEL_BG: Record<LogEntry['level'], string> = {
  INFO: '',
  WARN: 'bg-gold-bg',
  ERROR: 'bg-red-bg',
  SWIM: 'bg-green-bg',
  EPOLL: 'bg-blue-bg',
  CHAOS: 'bg-red-900/10',
  TAURI: 'bg-purple-900/10',
};

const LOG_TEMPLATES: { level: LogEntry['level']; message: string }[] = [
  { level: 'SWIM', message: 'Consensus state verified. 5 peers connected.' },
  { level: 'EPOLL', message: 'epoll_wait returned 12 file descriptors ready for I/O.' },
  { level: 'INFO', message: 'SPSC ring buffer flushed: 4096 entries (zero-copy).' },
  { level: 'TAURI', message: 'Desktop IPC listener registered on channel telemetry_events.' },
  { level: 'INFO', message: 'Arena allocator pool validated. 0 memory leaks detected.' },
  { level: 'SWIM', message: 'Node 4 RTT: 6.85ms (heartbeat ACK round-trip).' },
  { level: 'EPOLL', message: 'L4 load balancer selected backend node 3 for new connection.' },
  { level: 'INFO', message: 'LSM compaction filter checkpoint: L0→L1 merge at 78%.' },
  { level: 'WARN', message: 'Replication lag increasing on follower node 5 (current: 142ms).' },
  { level: 'CHAOS', message: 'Chaos agent armed. Awaiting injection trigger.' },
  { level: 'TAURI', message: 'Window state saved. GPU compute command buffer submitted.' },
  { level: 'SWIM', message: 'Suspicion detected: node 2 probe timeout. Indirect probe dispatched.' },
  { level: 'INFO', message: 'Request throughput: 48,201 req/s (p99 latency: 1.02ms).' },
  { level: 'EPOLL', message: 'Connection pool: 4,890 active sockets, 12 pending close.' },
  { level: 'WARN', message: 'Write amplification factor: 2.4x (compaction overhead).' },
  { level: 'SWIM', message: 'Membership list updated: 5 active, 0 suspected, 0 dead.' },
  { level: 'INFO', message: 'B+Tree root page flushed to disk (L0 SSTable).' },
  { level: 'TAURI', message: 'Tauri IPC bandwidth: 12.4 MB/s (telemetry + log stream).' },
  { level: 'ERROR', message: 'I/O timeout on node 2 replication channel (retry 3/5).' },
  { level: 'EPOLL', message: 'HTTP proxy routing table rebuilt. 24 upstream routes loaded.' },
  { level: 'WARN', message: 'Memory arena utilization at 82% on node 1 (triggering GC).' },
  { level: 'CHAOS', message: 'Network partition simulation: isolating nodes 3, 4, 5 from quorum.' },
  { level: 'INFO', message: 'Zero-allocation telemetry pipeline: 120Hz sample rate clean.' },
  { level: 'SWIM', message: 'Gossip propagation complete. Full state convergence in 1.2ms.' },
  { level: 'TAURI', message: 'Drag-and-drop IPC handler registered for file transfer.' },
  { level: 'INFO', message: 'Database page cache hit ratio: 94.8% (2.4k lookups/sec).' },
  { level: 'ERROR', message: 'Malformed frame received on port 4002. CRC mismatch. Dropped.' },
  { level: 'EPOLL', message: 'TLS handshake completed for client 10.0.1.42:34512.' },
  { level: 'WARN', message: 'Fuzzer stress test: write latency increased to 8.4ms (p99).' },
  { level: 'CHAOS', message: 'Crash injection complete. Node 2 daemon terminated (SIGKILL).' },
];

const CHAOS_LOGS: { level: LogEntry['level']; message: string }[] = [
  { level: 'CHAOS', message: 'Network partition active: split-brain condition detected. Candidate election triggered.' },
  { level: 'CHAOS', message: 'Malformed frame filter injecting 12% noise into replicator stream. CRC errors expected.' },
  { level: 'CHAOS', message: 'Property-based fuzzer running: 10,000 random mutations/sec. Arena stress test active.' },
  { level: 'CHAOS', message: 'Quorum lost: only 3/5 nodes reachable. Consensus algorithm in recovery mode.' },
  { level: 'SWIM', message: 'Indirect probe ACK received. Node 2 confirmed DEAD. Updating membership list.' },
  { level: 'ERROR', message: 'WAL fsync failure on partitioned node. Transaction log pending replay.' },
  { level: 'WARN', message: 'Replication lag critical: 999ms on partitioned nodes 3, 4, 5.' },
  { level: 'INFO', message: 'Operator intervention required: type "chaos" in TTY for status overview.' },
];

let logIdCounter = 0;

function generateLogEntry(chaosMode: ViewChaosStudioProps['chaosMode']): LogEntry {
  const now = new Date();
  const timestamp = now.toTimeString().split(' ')[0] + '.' + String(now.getMilliseconds()).padStart(3, '0');

  let template: { level: LogEntry['level']; message: string };

  // Inject chaos-specific logs with higher probability when chaos modes active
  if (chaosMode.partitionSplit && Math.random() < 0.15) {
    template = CHAOS_LOGS[0];
  } else if (chaosMode.malformedFrames && Math.random() < 0.15) {
    template = CHAOS_LOGS[1];
  } else if (chaosMode.fuzzerRunning && Math.random() < 0.12) {
    template = CHAOS_LOGS[2];
  } else if (chaosMode.crashNode2 && Math.random() < 0.10) {
    template = CHAOS_LOGS[4];
  } else {
    template = LOG_TEMPLATES[Math.floor(Math.random() * LOG_TEMPLATES.length)];
  }

  return { id: ++logIdCounter, timestamp, level: template.level, message: template.message };
}

// Row renderer component for react-window
const LogRow = ({ index, style, data }: { index: number; style: React.CSSProperties; data: { logs: LogEntry[] } }) => {
  const log = data.logs[index];
  if (!log) return null;

  const colorClass = LEVEL_COLORS[log.level];
  const bgClass = LEVEL_BG[log.level];

  return (
    <div style={style} className={`flex items-center gap-3 px-3 py-0.5 text-[10px] font-mono border-b border-border/30 ${bgClass}`}>
      <span className="text-text-soft w-[110px] shrink-0 select-none">{log.timestamp}</span>
      <span className={`w-[52px] shrink-0 font-bold tracking-wider ${colorClass}`}>
        [{log.level.padEnd(5, ' ')}]
      </span>
      <span className={`truncate ${colorClass.replace('text-', 'text-')} text-text`}>
        {log.message}
      </span>
    </div>
  );
};

export default function ViewChaosStudio({ chaosMode, setChaosMode }: ViewChaosStudioProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isStreaming, setIsStreaming] = useState(true);
  const listRef = useRef<List>(null);
  const logEndRef = useRef<HTMLDivElement>(null);
  const autoScrollRef = useRef(true);

  // Generate log entries at high frequency (simulating real telemetry)
  useEffect(() => {
    if (!isStreaming) return;

    const burstCount = chaosMode.fuzzerRunning ? 8 : chaosMode.partitionSplit ? 5 : 3;

    const interval = setInterval(() => {
      setLogs(prev => {
        const newLogs: LogEntry[] = [];
        for (let i = 0; i < burstCount; i++) {
          newLogs.push(generateLogEntry(chaosMode));
        }
        const combined = [...prev, ...newLogs];
        // Keep max 10,000 entries in memory (react-window virtualizes, but we limit memory)
        return combined.length > 10000 ? combined.slice(-10000) : combined;
      });
    }, chaosMode.fuzzerRunning ? 50 : 120);

    return () => clearInterval(interval);
  }, [isStreaming, chaosMode]);

  // Auto-scroll behavior
  useEffect(() => {
    if (autoScrollRef.current && logs.length > 0) {
      listRef.current?.scrollToItem(logs.length - 1, 'end');
    }
  }, [logs.length]);

  // Handle scroll events to detect manual scroll-up
  const handleScroll = useCallback(({ scrollDirection, scrollOffset, scrollUpdateWasRequested }: any) => {
    if (!scrollUpdateWasRequested && scrollDirection === 'backward') {
      autoScrollRef.current = false;
    }
  }, []);

  const chaosToggles = [
    { key: 'partitionSplit' as const, label: 'Trigger Network Partition Split', icon: '⊗', description: 'Isolates nodes 3, 4, 5 from quorum — triggers candidate re-election' },
    { key: 'malformedFrames' as const, label: 'Inject Malformed Frames', icon: '⊕', description: 'Corrupts 12% of replication frames with CRC mismatch errors' },
    { key: 'crashNode2' as const, keyLabel: 'Crash Node 2', icon: '⊘', description: 'Sends SIGKILL to daemon on node 2 — removes from SWIM membership' },
    { key: 'fuzzerRunning' as const, label: 'Run Property-Based Fuzzer', icon: '◉', description: 'High-throughput random mutation engine (10k ops/sec) — stress tests LSM write path' },
  ];

  return (
    <div className="flex flex-col lg:flex-row gap-6">
      {/* Component A: Chaos Operations Panel - Left Side */}
      <div className="w-full lg:w-[380px] shrink-0 cyber-panel rounded overflow-hidden">
        <div className="px-4 py-2 border-b border-border">
          <span className="text-xs font-mono font-bold text-text-soft tracking-wider">CHAOS_ENGINE // INJECTION_CONTROLS</span>
        </div>
        <div className="p-4 space-y-4">
          {chaosToggles.map(({ key, label, icon, description }) => {
            const isActive = chaosMode[key];
            return (
              <div
                key={key}
                className={`cyber-panel rounded p-3 transition-all duration-200 cursor-pointer ${
                  isActive
                    ? key === 'crashNode2'
                      ? 'border-red/40 bg-red-bg'
                      : 'border-gold/40 bg-gold-bg'
                    : 'hover:border-border/80'
                }`}
                onClick={() => setChaosMode(prev => ({ ...prev, [key]: !prev[key] }))}
              >
                <div className="flex items-center justify-between mb-1.5">
                  <div className="flex items-center gap-2">
                    <span className={`text-lg ${isActive ? (key === 'crashNode2' ? 'text-red' : 'text-gold') : 'text-text-soft'}`}>
                      {icon}
                    </span>
                    <span className={`text-[11px] font-mono font-bold ${isActive ? (key === 'crashNode2' ? 'text-red' : 'text-gold') : 'text-text'}`}>
                      {label}
                    </span>
                  </div>
                  {/* Toggle switch */}
                  <div className={`w-9 h-5 rounded-full relative transition-colors duration-200 ${
                    isActive ? (key === 'crashNode2' ? 'bg-red/40' : 'bg-gold/40') : 'bg-border'
                  }`}>
                    <div className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full transition-transform duration-200 ${
                      isActive ? 'translate-x-4' : 'translate-x-0'
                    } ${isActive ? (key === 'crashNode2' ? 'bg-red' : 'bg-gold') : 'bg-neutral-500'}`} />
                  </div>
                </div>
                <p className="text-[9px] font-mono text-text-soft ml-7">{description}</p>
              </div>
            );
          })}

          {/* Danger zone indicator */}
          <div className={`border-t border-border pt-4 mt-6 ${Object.values(chaosMode).some(Boolean) ? '' : 'opacity-40'}`}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className={`h-1.5 w-1.5 rounded-full ${Object.values(chaosMode).some(Boolean) ? 'bg-red animate-pulse' : 'bg-neutral-600'}`} />
                <span className={`text-[9px] font-mono font-bold tracking-wider ${Object.values(chaosMode).some(Boolean) ? 'text-red' : 'text-text-soft'}`}>
                  CHAOS STATE: {Object.values(chaosMode).some(Boolean) ? 'ARMED' : 'STANDBY'}
                </span>
              </div>
              {Object.values(chaosMode).filter(Boolean).length > 0 && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setChaosMode({ partitionSplit: false, malformedFrames: false, crashNode2: false, fuzzerRunning: false });
                  }}
                  className="text-[8px] font-mono text-red border border-neon-pink-border px-2 py-0.5 rounded hover:bg-red-bg transition-colors"
                >
                  DISARM ALL
                </button>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Component B: Real-time Telemetry Log Stream Viewport */}
      <div className="flex-1 cyber-panel rounded overflow-hidden flex flex-col">
        <div className="flex items-center justify-between px-4 py-2 border-b border-border">
          <div className="flex items-center gap-3">
            <span className="text-xs font-mono font-bold text-text-soft tracking-wider">TELEMETRY_EVENT_STREAM // REALTIME_VIRTUALIZED</span>
            <span className={`text-[9px] font-mono ${isStreaming ? 'text-green' : 'text-gold'}`}>
              {isStreaming ? '● LIVE' : '○ PAUSED'}
            </span>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-[9px] font-mono text-text-soft">
              {logs.length.toLocaleString()} entries buffered
            </span>
            <button
              onClick={() => setIsStreaming(!isStreaming)}
              className="text-[9px] font-mono border border-border px-2 py-0.5 rounded text-text-soft hover:bg-border transition-colors"
            >
              {isStreaming ? 'PAUSE' : 'RESUME'}
            </button>
            <button
              onClick={() => { setLogs([]); autoScrollRef.current = true; }}
              className="text-[9px] font-mono border border-border px-2 py-0.5 rounded text-text-soft hover:bg-border transition-colors"
            >
              CLEAR
            </button>
          </div>
        </div>

        {/* react-window virtualized list */}
        <div className="flex-1 bg-bg overflow-hidden" style={{ minHeight: 320 }}>
          {logs.length === 0 ? (
            <div className="flex items-center justify-center h-full text-[10px] font-mono text-text-muted">
              Waiting for telemetry stream...
            </div>
          ) : (
            <List
              ref={listRef}
              height={400}
              itemCount={logs.length}
              itemSize={22}
              width="100%"
              itemData={{ logs }}
              onScroll={handleScroll}
              overscanCount={20}
            >
              {LogRow}
            </List>
          )}
        </div>
      </div>
    </div>
  );
}
