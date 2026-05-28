"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import Sparkline from "./Sparkline";

type ContainerState = 'created' | 'running' | 'paused' | 'stopped' | 'dead';

interface Container {
  id: string;
  pid: number | null;
  state: ContainerState;
  rootfs: string;
  hostname: string;
  memoryLimitMb: number;
  cpuLimit: number;
  pidsMax: number;
  command: string;
  readonlyRootfs: boolean;
  networkMode: string;
  portMappings: string;
  createdAt: number;
  exitedAt: number | null;
  exitCode: number | null;
  cpuUsage: number;
  memoryUsage: number;
  pidsCurrent: number;
  iops: number;
}

interface ContainerEvent {
  id: string;
  type: string;
  message: string;
  timestamp: number;
}

interface ViewContainerRuntimeProps {
  chaosMode: { partitionSplit: boolean; malformedFrames: boolean; crashNode2: boolean; fuzzerRunning: boolean; };
}

const CONTAINER_NAMES = [
  'web-server', 'api-gateway', 'db-replica', 'cache-node', 'worker-pool',
  'log-shipper', 'metrics-collector', 'dns-resolver', 'auth-service', 'file-store',
];

const ROOTFS_IMAGES = ['alpine:3.20', 'ubuntu:24.04', 'debian:12', 'busybox:1.36', 'fedora:40'];

function randomHex(len: number): string {
  const chars = 'abcdef0123456789';
  let result = '';
  for (let i = 0; i < len; i++) result += chars[Math.floor(Math.random() * chars.length)];
  return result;
}

function generateMockContainers(count: number, chaosMode: ViewContainerRuntimeProps['chaosMode']): Container[] {
  const containers: Container[] = [];
  const baseTime = Date.now() - 3600000;

  for (let i = 0; i < count; i++) {
    const id = `ctr-${randomHex(8)}`;
    const isCrashed = chaosMode.crashNode2 && i === 1;
    const isDegraded = chaosMode.partitionSplit && i > 2;
    const isFuzzed = chaosMode.fuzzerRunning && i === 4;

    let state: ContainerState = 'running';
    if (isCrashed) state = 'dead';
    else if (isDegraded) state = 'paused';
    else if (isFuzzed && Math.random() > 0.7) state = 'stopped';

    const cpu = state === 'dead' ? 0 : isFuzzed ? 65 + Math.random() * 30 : 8 + Math.random() * 25;
    const memory = state === 'dead' ? 0 : state === 'paused' ? Math.floor(Math.random() * 16) : 32 + Math.floor(Math.random() * 192);
    const pids = state === 'dead' ? 0 : Math.floor(Math.random() * 50) + 2;
    const iops = state === 'dead' ? 0 : state === 'paused' ? 0 : 500 + Math.floor(Math.random() * 3000);

    containers.push({
      id,
      pid: state === 'dead' ? null : 10000 + Math.floor(Math.random() * 50000),
      state,
      rootfs: ROOTFS_IMAGES[i % ROOTFS_IMAGES.length],
      hostname: CONTAINER_NAMES[i % CONTAINER_NAMES.length],
      memoryLimitMb: [64, 128, 256, 512, 1024][i % 5],
      cpuLimit: [0.5, 1.0, 1.5, 2.0, 4.0][i % 5],
      pidsMax: 256,
      command: i === 0 ? '/bin/sh -c "nginx -g daemon off;"' :
               i === 1 ? '/usr/bin/node server.js' :
               i === 2 ? '/usr/bin/postgres -D /data' :
               i === 3 ? '/usr/bin/redis-server' :
               i === 4 ? '/usr/bin/python3 worker.py' :
               '/bin/sh',
      readonlyRootfs: i % 2 === 0,
      networkMode: i === 2 ? 'host' : 'bridge',
      portMappings: i === 0 ? '80:80,443:443' : i === 1 ? '3000:3000' : i === 2 ? '5432:5432' : '',
      createdAt: baseTime - (count - i) * 120000,
      exitedAt: state === 'stopped' || state === 'dead' ? baseTime + i * 30000 : null,
      exitCode: state === 'dead' ? 137 : state === 'stopped' ? 0 : null,
      cpuUsage: cpu,
      memoryUsage: memory,
      pidsCurrent: pids,
      iops,
    });
  }

  return containers;
}

function generateContainerEvents(containers: Container[]): ContainerEvent[] {
  const events: ContainerEvent[] = [];
  const baseTime = Date.now() - 3600000;

  const eventTypes = ['created', 'started', 'healthcheck_failed', 'oom_killed', 'stopped'];

  for (const c of containers) {
    events.push({
      id: c.id,
      type: 'created',
      message: `Container ${c.id} created (rootfs: ${c.rootfs}, hostname: ${c.hostname})`,
      timestamp: baseTime + Math.floor(Math.random() * 100000),
    });
    events.push({
      id: c.id,
      type: 'started',
      message: `Container started (PID ${c.pid}, ${c.networkMode} networking)`,
      timestamp: baseTime + Math.floor(Math.random() * 100000) + 500,
    });
    if (c.state === 'stopped') {
      events.push({
        id: c.id,
        type: 'stopped',
        message: `Container exited with code ${c.exitCode}`,
        timestamp: c.exitedAt!,
      });
    }
    if (c.state === 'dead') {
      events.push({
        id: c.id,
        type: 'oom_killed',
        message: `OOM killer invoked: process exceeded ${c.memoryLimitMb}MB limit`,
        timestamp: c.exitedAt!,
      });
    }
    if (Math.random() > 0.7) {
      events.push({
        id: c.id,
        type: 'healthcheck_failed',
        message: `Health check failed (exit code 1): TCP connect to port ${3000 + Math.floor(Math.random() * 5000)} timed out`,
        timestamp: baseTime + Math.floor(Math.random() * 100000) + 2000,
      });
    }
  }

  events.sort((a, b) => a.timestamp - b.timestamp);
  return events;
}

function formatTime(ts: number): string {
  const d = new Date(ts);
  return d.toTimeString().split(' ')[0] + '.' + String(d.getMilliseconds()).padStart(3, '0');
}

type FormView = 'list' | 'create' | 'detail';

export default function ViewContainerRuntime({ chaosMode }: ViewContainerRuntimeProps) {
  const [containers, setContainers] = useState<Container[]>([]);
  const [events, setEvents] = useState<ContainerEvent[]>([]);
  const [selectedContainer, setSelectedContainer] = useState<Container | null>(null);
  const [formView, setFormView] = useState<FormView>('list');
  const [isCreating, setIsCreating] = useState(false);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  const [liveClock, setLiveClock] = useState('');
  const [filterText, setFilterText] = useState('');
  const [expandedCard, setExpandedCard] = useState<string | null>(null);
  const eventsEndRef = useRef<HTMLDivElement>(null);
  const [showHelp, setShowHelp] = useState(false);

  const showToast = useCallback((message: string, type: 'success' | 'error' | 'info' = 'info') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }, []);

  // Initialize mock containers
  useEffect(() => {
    const initialContainers = generateMockContainers(5, chaosMode);
    setContainers(initialContainers);
    setEvents(generateContainerEvents(initialContainers));
  }, []);

  // Live clock
  useEffect(() => {
    const interval = setInterval(() => {
      const now = new Date();
      setLiveClock(now.toTimeString().split(' ')[0] + '.' + String(now.getMilliseconds()).padStart(3, '0') + 'Z');
    }, 50);
    return () => clearInterval(interval);
  }, []);

  // Live stats updates
  useEffect(() => {
    const interval = setInterval(() => {
      setContainers(prev => prev.map(c => {
        if (c.state === 'dead') return c;
        if (c.state === 'paused') return c;
        const isFuzzed = chaosMode.fuzzerRunning && c.hostname === 'worker-pool';
        return {
          ...c,
          cpuUsage: isFuzzed ? 60 + Math.random() * 35 : Math.max(0, c.cpuUsage + (Math.random() - 0.5) * 6),
          memoryUsage: Math.max(0, Math.min(c.memoryLimitMb, c.memoryUsage + (Math.random() - 0.5) * 12)),
          pidsCurrent: Math.max(1, c.pidsCurrent + Math.floor((Math.random() - 0.5) * 4)),
          iops: Math.max(0, c.iops + Math.floor((Math.random() - 0.5) * 200)),
        };
      }));
    }, 800);
    return () => clearInterval(interval);
  }, [chaosMode]);

  // Auto-scroll events
  useEffect(() => {
    eventsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [events.length]);

  // Update events when chaos changes
  useEffect(() => {
    const newEvent: ContainerEvent = {
      id: 'system',
      type: chaosMode.partitionSplit ? 'warning' : chaosMode.fuzzerRunning ? 'info' : 'system',
      message: chaosMode.partitionSplit ? 'Network partition simulated — container-to-container communication disrupted' :
               chaosMode.fuzzerRunning ? 'Property-based fuzzer active — container resource usage spiking' :
               chaosMode.crashNode2 ? 'Container ctr-a1b2c3d4 (db-replica) killed via SIGKILL' :
               chaosMode.malformedFrames ? 'Malformed IPC frames detected — container event stream corrupted' :
               'System stable: all containers nominal',
      timestamp: Date.now(),
    };
    setEvents(prev => [...prev, newEvent]);
  }, [chaosMode]);

  const createContainer = () => {
    setIsCreating(true);
    const rootfs = ROOTFS_IMAGES[Math.floor(Math.random() * ROOTFS_IMAGES.length)];
    const hostname = CONTAINER_NAMES[Math.floor(Math.random() * CONTAINER_NAMES.length)];
    const newContainer: Container = {
      id: `ctr-${randomHex(8)}`,
      pid: 50000 + Math.floor(Math.random() * 10000),
      state: 'created',
      rootfs,
      hostname,
      memoryLimitMb: [64, 128, 256, 512][Math.floor(Math.random() * 4)],
      cpuLimit: [0.5, 1.0, 1.5][Math.floor(Math.random() * 3)],
      pidsMax: 256,
      command: '/bin/sh',
      readonlyRootfs: Math.random() > 0.5,
      networkMode: Math.random() > 0.3 ? 'bridge' : 'host',
      portMappings: '',
      createdAt: Date.now(),
      exitedAt: null,
      exitCode: null,
      cpuUsage: 0,
      memoryUsage: 0,
      pidsCurrent: 1,
      iops: 0,
    };

    setContainers(prev => [...prev, newContainer]);
    setEvents(prev => [...prev, {
      id: newContainer.id,
      type: 'created',
      message: `Container ${newContainer.id} created (rootfs: ${rootfs}, hostname: ${hostname})`,
      timestamp: Date.now(),
    }]);
    setIsCreating(false);
    setFormView('list');
    showToast(`Container ${newContainer.id} created`, 'success');
  };

  const startContainer = (id: string) => {
    setContainers(prev => prev.map(c => {
      if (c.id !== id) return c;
      return { ...c, state: 'running' as ContainerState, pid: 50000 + Math.floor(Math.random() * 50000) };
    }));
    setEvents(prev => [...prev, {
      id, type: 'started',
      message: `Container ${id} started — ${cgroups_desc(id)}`,
      timestamp: Date.now(),
    }]);
    showToast(`Container ${id} started`, 'success');
  };

  const stopContainer = (id: string) => {
    setContainers(prev => prev.map(c => {
      if (c.id !== id) return c;
      return { ...c, state: 'stopped' as ContainerState, exitedAt: Date.now(), exitCode: 0 };
    }));
    setEvents(prev => [...prev, {
      id, type: 'stopped',
      message: `Container ${id} exited with code 0`,
      timestamp: Date.now(),
    }]);
    showToast(`Container ${id} stopped`, 'info');
  };

  const killContainer = (id: string) => {
    setContainers(prev => prev.map(c => {
      if (c.id !== id) return c;
      return { ...c, state: 'dead' as ContainerState, exitedAt: Date.now(), exitCode: 137 };
    }));
    setEvents(prev => [...prev, {
      id, type: 'oom_killed',
      message: `Container ${id} killed via SIGKILL (signal 9)`,
      timestamp: Date.now(),
    }]);
    showToast(`Container ${id} killed`, 'error');
  };

  const pauseContainer = (id: string) => {
    setContainers(prev => prev.map(c => {
      if (c.id !== id) return c;
      return { ...c, state: 'paused' as ContainerState };
    }));
    setEvents(prev => [...prev, {
      id, type: 'system',
      message: `Container ${id} paused (cgroup freezer)`,
      timestamp: Date.now(),
    }]);
    showToast(`Container ${id} paused`, 'info');
  };

  const resumeContainer = (id: string) => {
    setContainers(prev => prev.map(c => {
      if (c.id !== id) return c;
      return { ...c, state: 'running' as ContainerState };
    }));
    setEvents(prev => [...prev, {
      id, type: 'system',
      message: `Container ${id} resumed (cgroup unfrozen)`,
      timestamp: Date.now(),
    }]);
    showToast(`Container ${id} resumed`, 'success');
  };

  const removeContainer = (id: string) => {
    setContainers(prev => prev.filter(c => c.id !== id));
    setEvents(prev => [...prev, {
      id, type: 'system',
      message: `Container ${id} removed — state, cgroup, and network cleaned up`,
      timestamp: Date.now(),
    }]);
    if (selectedContainer?.id === id) setSelectedContainer(null);
    showToast(`Container ${id} removed`, 'info');
  };

  const cgroups_desc = (id: string) => {
    const c = containers.find(x => x.id === id);
    if (!c) return 'memory.max=none cpu.weight=100 pids.max=256';
    return `memory.max=${c.memoryLimitMb}M cpu.max=${c.cpuLimit} pids.max=${c.pidsMax}`;
  };

  const filteredContainers = containers.filter(c =>
    c.id.toLowerCase().includes(filterText.toLowerCase()) ||
    c.hostname.toLowerCase().includes(filterText.toLowerCase()) ||
    c.state.toLowerCase().includes(filterText.toLowerCase())
  );

  const stateColor = (state: ContainerState) => {
    switch (state) {
      case 'running': return 'text-green';
      case 'paused': return 'text-gold';
      case 'stopped': return 'text-text-soft';
      case 'dead': return 'text-red';
      case 'created': return 'text-blue';
    }
  };

  const stateBg = (state: ContainerState) => {
    switch (state) {
      case 'running': return 'bg-green-bg border-green-border';
      case 'paused': return 'bg-gold-bg border-gold-border';
      case 'stopped': return 'bg-surface border-border';
      case 'dead': return 'bg-red-bg border-red-border';
      case 'created': return 'bg-blue-bg border-blue-border';
    }
  };

  const stateDot = (state: ContainerState) => {
    switch (state) {
      case 'running': return 'bg-green animate-pulse';
      case 'paused': return 'bg-gold';
      case 'stopped': return 'bg-neutral-500';
      case 'dead': return 'bg-red';
      case 'created': return 'bg-blue';
    }
  };

  return (
    <div className="flex flex-col gap-5">
      {/* Toast notification */}
      {toast && (
        <div className={`fixed top-4 right-4 z-50 px-4 py-2 rounded text-[10px] font-mono font-bold border shadow-lg transition-all duration-300 ${
          toast.type === 'success' ? 'bg-green-bg border-green-border text-green' :
          toast.type === 'error' ? 'bg-red-bg border-red-border text-red' :
          'bg-blue-bg border-blue-border text-blue'
        }`}>
          {toast.type === 'success' ? '✓ ' : toast.type === 'error' ? '✗ ' : '→ '}
          {toast.message}
        </div>
      )}

      {/* Live clock bar */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-[10px] font-mono text-blue font-bold tracking-widest">
            CONTAINER_ENGINE // RUNTIME_CTRL
          </span>
          <span className="text-[9px] font-mono text-text-soft">
            {containers.length} containers ({containers.filter(c => c.state === 'running').length} active)
          </span>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-[10px] font-mono text-green font-bold">{liveClock}</span>
        </div>
      </div>

      {/* Action buttons + search bar */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-2">
          <button
            onClick={createContainer}
            disabled={isCreating}
            className="bg-blue-bg border border-blue-border text-blue text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-blue/10 transition-all disabled:opacity-40"
          >
            + CONTAINER.RUN
          </button>
          {selectedContainer && selectedContainer.state === 'created' && (
            <button onClick={() => startContainer(selectedContainer.id)}
              className="bg-green-bg border border-green-border text-green text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-green/10 transition-all">
              ▶ START
            </button>
          )}
          {selectedContainer?.state === 'running' && (
            <>
              <button onClick={() => stopContainer(selectedContainer.id)}
                className="bg-gold-bg border border-gold-border text-gold text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-gold/10 transition-all">
                ■ STOP
              </button>
              <button onClick={() => killContainer(selectedContainer.id)}
                className="bg-red-bg border border-red-border text-red text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-red/10 transition-all">
                ⊘ KILL
              </button>
              <button onClick={() => pauseContainer(selectedContainer.id)}
                className="bg-gold-bg border border-gold-border text-gold text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-gold/10 transition-all">
                ⏸ PAUSE
              </button>
            </>
          )}
          {selectedContainer?.state === 'paused' && (
            <button onClick={() => resumeContainer(selectedContainer.id)}
              className="bg-green-bg border border-green-border text-green text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-green/10 transition-all">
              ▶ RESUME
            </button>
          )}
          {selectedContainer && (selectedContainer.state === 'stopped' || selectedContainer.state === 'dead') && (
            <button onClick={() => removeContainer(selectedContainer.id)}
              className="bg-red-bg border border-red-border text-red text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-red/10 transition-all">
              ✕ RM
            </button>
          )}
        </div>

        <div className="flex items-center gap-2">
          <input
            type="text"
            value={filterText}
            onChange={e => setFilterText(e.target.value)}
            placeholder="Filter by ID, name, or state..."
            className="bg-bg border border-border rounded px-2 py-1.5 text-[10px] font-mono text-text placeholder-neutral-600 outline-none focus:border-blue/50 transition-all w-[150px] lg:w-[200px]"
          />
          <button
            onClick={() => setShowHelp(!showHelp)}
            className="text-[10px] font-mono text-text-soft border border-border px-2 py-1.5 rounded hover:text-text transition-all"
          >
            ?
          </button>
        </div>
      </div>

      {/* Help panel */}
      {showHelp && (
        <div className="cyber-panel rounded p-4 text-[10px] font-mono leading-relaxed space-y-1">
          <div className="text-blue font-bold mb-2">CONTAINER ENGINE // KEYBOARD SHORTCUTS & COMMANDS</div>
          <div><span className="text-green">Ctrl+N</span> — Create new container</div>
          <div><span className="text-green">Ctrl+F</span> — Focus search filter</div>
          <div><span className="text-green">Ctrl+Shift+S</span> — Start selected container</div>
          <div><span className="text-green">Ctrl+Shift+K</span> — Kill selected container</div>
          <div><span className="text-green">Ctrl+Shift+R</span> — Remove selected container</div>
          <div><span className="text-green">?</span> — Toggle this help panel</div>
          <div className="text-text-soft mt-2">State transition diagram: CREATED → RUNNING ↔ PAUSED → STOPPED / DEAD</div>
        </div>
      )}

      {/* Main grid: Container list + Detail side panel */}
      <div className="flex flex-col lg:flex-row gap-4">
        {/* Container List */}
        <div className="flex-1 cyber-panel rounded overflow-hidden">
          <div className="px-3 py-2 border-b border-border">
            <span className="text-[10px] font-mono font-bold text-text-soft tracking-wider">
              CONTAINER_TABLE // {containers.length} ENTRIES
            </span>
          </div>

          {filteredContainers.length === 0 ? (
            <div className="flex items-center justify-center h-[200px] text-[10px] font-mono text-text-muted">
              {containers.length === 0 ? 'No containers. Click + CONTAINER.RUN to create one.' : 'No containers match the filter.'}
            </div>
          ) : (
            <div className="divide-y divide-border/50 max-h-[520px] overflow-y-auto">
              {filteredContainers.map(c => {
                const isSelected = selectedContainer?.id === c.id;
                const isExpanded = expandedCard === c.id;

                return (
                  <div
                    key={c.id}
                    className={`transition-all duration-100 cursor-pointer ${
                      isSelected
                        ? 'bg-blue-bg border-l-2 border-blue'
                        : 'border-l-2 border-transparent hover:bg-border/30'
                    }`}
                    onClick={() => setSelectedContainer(c)}
                  >
                    {/* Main row */}
                    <div className="px-3 py-2.5">
                      <div className="flex items-center justify-between mb-1">
                        <div className="flex items-center gap-2">
                          <span className={`h-2 w-2 rounded-full ${stateDot(c.state)}`} />
                          <span className="text-[10px] font-mono font-bold text-text">{c.id}</span>
                          <span className={`text-[8px] font-mono font-bold px-1.5 py-0.5 rounded border ${stateBg(c.state)} ${stateColor(c.state)}`}>
                            {c.state.toUpperCase()}
                          </span>
                          <span className="text-[9px] font-mono text-text-soft">{c.hostname}</span>
                        </div>
                        <div className="flex items-center gap-3">
                          <span className="text-[8px] font-mono text-text-soft">
                            PID {c.pid || '-'}
                          </span>
                          <button
                            onClick={(e) => { e.stopPropagation(); setExpandedCard(isExpanded ? null : c.id); }}
                            className="text-text-soft hover:text-text transition-colors"
                          >
                            {isExpanded ? '▲' : '▼'}
                          </button>
                        </div>
                      </div>

                      {/* Sparkline row */}
                      <div className="flex items-center gap-4 text-[9px] font-mono">
                        <span className={`text-green`}>CPU {c.cpuUsage.toFixed(1)}%</span>
                        <div className="w-16 h-3">
                          <Sparkline
                            data={Array.from({ length: 20 }, () => c.cpuUsage + (Math.random() - 0.5) * 10)}
                            color="green"
                            width={60} height={12}
                          />
                        </div>
                        <span className="text-gold">MEM {c.memoryUsage}/{c.memoryLimitMb}MB</span>
                        <span className="text-blue">{c.iops.toLocaleString()} IOPS</span>
                        {c.networkMode === 'host' && (
                          <span className="text-text-soft">HOST_NET</span>
                        )}
                      </div>
                    </div>

                    {/* Expanded details */}
                    {isExpanded && (
                      <div className="px-3 pb-3 pt-0 grid grid-cols-2 lg:grid-cols-4 gap-3 text-[9px] font-mono">
                        <div>
                          <span className="text-text-soft block">Rootfs</span>
                          <span className="text-text">{c.rootfs}</span>
                        </div>
                        <div>
                          <span className="text-text-soft block">Memory limit</span>
                          <span className="text-text">{c.memoryLimitMb}MB{c.readonlyRootfs ? ' (RO)' : ''}</span>
                        </div>
                        <div>
                          <span className="text-text-soft block">CPU max</span>
                          <span className="text-text">{c.cpuLimit} cores</span>
                        </div>
                        <div>
                          <span className="text-text-soft block">PIDs max</span>
                          <span className="text-text">{c.pidsMax} ({c.pidsCurrent} active)</span>
                        </div>
                        <div>
                          <span className="text-text-soft block">Command</span>
                          <span className="text-text truncate block" title={c.command}>{c.command}</span>
                        </div>
                        <div>
                          <span className="text-text-soft block">Network</span>
                          <span className="text-text">{c.networkMode === 'bridge' ? 'Bridge (10.88.0.x)' : 'Host'}</span>
                        </div>
                        <div>
                          <span className="text-text-soft block">Port mappings</span>
                          <span className="text-text">{c.portMappings || 'None'}</span>
                        </div>
                        <div>
                          <span className="text-text-soft block">Created</span>
                          <span className="text-text">{formatTime(c.createdAt)}</span>
                        </div>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Detail/Stats Panel */}
        <div className="w-full lg:w-[340px] xl:w-[400px] shrink-0 flex flex-col gap-4">
          {/* Selected Container Detail */}
          {selectedContainer ? (
            <div className="cyber-panel rounded p-3">
              <div className="text-[10px] font-mono font-bold text-text-soft tracking-wider mb-3 border-b border-border pb-2 flex items-center justify-between">
                <span>CONTAINER_DETAIL // {selectedContainer.id}</span>
                <button onClick={() => setSelectedContainer(null)} className="text-text-soft hover:text-text">✕</button>
              </div>

              {/* Metrics bars */}
              <div className="space-y-2.5 mb-3">
                <div>
                  <div className="flex justify-between text-[9px] font-mono mb-0.5">
                    <span className="text-green">CPU USAGE</span>
                    <span className="text-text">{selectedContainer.cpuUsage.toFixed(1)}%</span>
                  </div>
                  <div className="h-2 bg-bg rounded overflow-hidden">
                    <div className="h-full bg-green rounded transition-all duration-300" style={{ width: `${Math.min(100, selectedContainer.cpuUsage)}%` }} />
                  </div>
                </div>
                <div>
                  <div className="flex justify-between text-[9px] font-mono mb-0.5">
                    <span className="text-gold">MEMORY</span>
                    <span className="text-text">{selectedContainer.memoryUsage}MB / {selectedContainer.memoryLimitMb}MB</span>
                  </div>
                  <div className="h-2 bg-bg rounded overflow-hidden">
                    <div className="h-full bg-gold rounded transition-all duration-300" style={{ width: `${(selectedContainer.memoryUsage / selectedContainer.memoryLimitMb) * 100}%` }} />
                  </div>
                </div>
                <div>
                  <div className="flex justify-between text-[9px] font-mono mb-0.5">
                    <span className="text-blue">PIDS / IOPS</span>
                    <span className="text-text">{selectedContainer.pidsCurrent} / {selectedContainer.iops.toLocaleString()}</span>
                  </div>
                  <div className="h-2 bg-bg rounded overflow-hidden">
                    <div className="h-full bg-blue rounded transition-all duration-300" style={{ width: `${(selectedContainer.pidsCurrent / selectedContainer.pidsMax) * 100}%` }} />
                  </div>
                </div>
              </div>

              {/* Quick lifecycle action buttons on detail panel */}
              <div className="flex flex-wrap gap-1.5 border-t border-border pt-2.5">
                {selectedContainer.state === 'created' && (
                  <button onClick={() => startContainer(selectedContainer.id)}
                    className="flex-1 text-[9px] font-mono font-bold bg-green-bg border border-green-border text-green py-1.5 rounded hover:bg-green/10 transition-all">
                    ▶ START
                  </button>
                )}
                {selectedContainer.state === 'running' && (
                  <>
                    <button onClick={() => stopContainer(selectedContainer.id)}
                      className="flex-1 text-[9px] font-mono font-bold bg-gold-bg border border-gold-border text-gold py-1.5 rounded hover:bg-gold/10 transition-all">
                      ■ STOP
                    </button>
                    <button onClick={() => killContainer(selectedContainer.id)}
                      className="flex-1 text-[9px] font-mono font-bold bg-red-bg border border-red-border text-red py-1.5 rounded hover:bg-red/10 transition-all">
                      ⊘ KILL
                    </button>
                    <button onClick={() => pauseContainer(selectedContainer.id)}
                      className="flex-1 text-[9px] font-mono font-bold bg-gold-bg border border-gold-border text-gold py-1.5 rounded hover:bg-gold/10 transition-all">
                      ⏸ PAUSE
                    </button>
                  </>
                )}
                {selectedContainer.state === 'paused' && (
                  <button onClick={() => resumeContainer(selectedContainer.id)}
                    className="flex-1 text-[9px] font-mono font-bold bg-green-bg border border-green-border text-green py-1.5 rounded hover:bg-green/10 transition-all">
                      ▶ RESUME
                  </button>
                )}
                {(selectedContainer.state === 'stopped' || selectedContainer.state === 'dead') && (
                  <button onClick={() => removeContainer(selectedContainer.id)}
                    className="flex-1 text-[9px] font-mono font-bold bg-red-bg border border-red-border text-red py-1.5 rounded hover:bg-red/10 transition-all">
                    ✕ RM --force
                  </button>
                )}
              </div>
            </div>
          ) : (
            <div className="cyber-panel rounded p-4 flex items-center justify-center h-[100px] text-[10px] font-mono text-text-muted">
              Select a container to view details
            </div>
          )}

          {/* Aggregate Stats Mini-Panel */}
          <div className="cyber-panel rounded p-3">
            <div className="text-[10px] font-mono font-bold text-text-soft tracking-wider mb-2 border-b border-border pb-1">
              CLUSTER_AGGREGATE
            </div>
            <div className="grid grid-cols-2 gap-2 text-[9px] font-mono">
              <div>
                <span className="text-text-soft block">Active containers</span>
                <span className="text-green text-[14px] font-bold">{containers.filter(c => c.state === 'running').length}</span>
              </div>
              <div>
                <span className="text-text-soft block">Total memory</span>
                <span className="text-text text-[14px] font-bold">
                  {containers.reduce((s, c) => s + c.memoryUsage, 0)}MB
                </span>
              </div>
              <div>
                <span className="text-text-soft block">Total CPU avg</span>
                <span className="text-blue text-[14px] font-bold">
                  {(containers.reduce((s, c) => s + c.cpuUsage, 0) / Math.max(1, containers.length)).toFixed(1)}%
                </span>
              </div>
              <div>
                <span className="text-text-soft block">Total IOPS</span>
                <span className="text-gold text-[14px] font-bold">
                  {containers.reduce((s, c) => s + c.iops, 0).toLocaleString()}
                </span>
              </div>
            </div>
          </div>

          {/* Events/Logs Console */}
          <div className="cyber-panel rounded p-3 flex-1 min-h-[120px] max-h-[200px]">
            <div className="text-[10px] font-mono font-bold text-text-soft tracking-wider mb-2 border-b border-border pb-1">
              EVENTS_STREAM // {events.length} ENTRIES
            </div>
            <div className="overflow-y-auto h-[120px] space-y-0.5">
              {events.slice(-40).map((ev, i) => (
                <div key={i} className="flex gap-2 text-[8px] font-mono leading-relaxed">
                  <span className="text-text-muted shrink-0 w-[85px]">{formatTime(ev.timestamp)}</span>
                  <span className={
                    ev.type === 'created' ? 'text-blue font-bold shrink-0' :
                    ev.type === 'started' ? 'text-green font-bold shrink-0' :
                    ev.type === 'stopped' ? 'text-text-soft shrink-0' :
                    ev.type === 'oom_killed' ? 'text-red font-bold shrink-0' :
                    ev.type === 'healthcheck_failed' ? 'text-gold font-bold shrink-0' :
                    ev.type === 'warning' ? 'text-gold font-bold shrink-0' :
                    'text-text-soft shrink-0'
                  }>
                    [{ev.type.toUpperCase().padEnd(16, ' ')}]
                  </span>
                  <span className="text-text truncate">{ev.message}</span>
                </div>
              ))}
              <div ref={eventsEndRef} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
