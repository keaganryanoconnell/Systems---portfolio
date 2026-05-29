"use client";

import { useState } from "react";
import { Construction } from "lucide-react";

const PROJECTS = [
  { id:"lsm", name:"LSM Storage Engine", lang:"Rust", tier:1, status:"STABLE",
    problem:"Row-locking contention causes database timeouts during burst ingestion.",
    primitives:"MemTable, SSTable, Compaction Pipeline, BTreeMap, AtomicU64",
    metric:"3.1:1 compression ratio, 128-pt blocks, 17 tests passing, 0 memory leaks" },
  { id:"sql", name:"SQL Query Engine", lang:"Rust", tier:1, status:"STABLE",
    problem:"No in-browser relational query parser exists for WASM OLAP workloads.",
    primitives:"Recursive Descent Parser, AST, Query Planner, QueryExecutor, Catalog",
    metric:"6 statement types (SELECT/INSERT/CREATE/DROP/DELETE/UPDATE), 14 expression types" },
  { id:"columnar", name:"WASM Columnar Engine", lang:"Rust/WASM", tier:1, status:"STABLE",
    problem:"JSON serialization tax kills 60fps rendering at million-row scale in browser.",
    primitives:"bytemuck, UnsafeCell Vecs, EngineMemoryManager LRU, vectorized filter scan",
    metric:"17 tests, 256MB heap cap, zero-copy ingestion via ArrayBuffer pointer cast" },
  { id:"lob", name:"Limit Order Book Engine", lang:"Rust", tier:1, status:"STABLE",
    problem:"Mutex-based order matching kills throughput at 1M+ orders per second.",
    primitives:"OrderPool (1M slots), price-time priority, CAS slot claiming",
    metric:"p50=200ns, p99=3.6us, avg=221ns per order at 1M entries" },
  { id:"telemetry", name:"Telemetry Edge Aggregator", lang:"Rust", tier:1, status:"STABLE",
    problem:"Edge gateways OOM from unbounded UDP buffers under high-volume sensor streams.",
    primitives:"Gorilla delta-of-delta, packet ring (512KB), bounded LogBuffer",
    metric:"3.1:1 compression, 100K packet integration test, 256MB mem cap" },
  { id:"sensor", name:"Sensor Fusion MPMC Buffer", lang:"Rust", tier:1, status:"STABLE",
    problem:"LiDAR/Camera/IMU streams need deterministic merge without mutex overhead.",
    primitives:"MPMC CAS write cursor, Acquire/Release+fence(SeqCst), CPU affinity",
    metric:"30K frames, TSAN data-race-free verified, cross-platform affinity" },
  { id:"container", name:"Container Runtime Engine", lang:"Rust", tier:1, status:"ACTIVE",
    problem:"Process isolation without Docker's 100MB+ daemon overhead on edge devices.",
    primitives:"clone(NEWPID|NEWNS|NEWUTS|NEWNET), cgroups v2, seccomp-BPF, OverlayFS",
    metric:"~120 syscall whitelist, 5 capabilities kept from ~40, NO_NEW_PRIVS enforced" },
  { id:"core", name:"Core Systems Library", lang:"Rust", tier:1, status:"STABLE",
    problem:"Thread coordination without mutex contention in hot-path telemetry logging.",
    primitives:"Lock-free SPSC queue (const-generic), zero-alloc JSON telemetry logger",
    metric:"50ns push/pop pair, Criterion benchmarks, OnceLock static integration" },
  { id:"raft", name:"Raft Distributed KV Store", lang:"Rust", tier:2, status:"STABLE",
    problem:"Network partitions silently corrupt replicated state across cluster nodes.",
    primitives:"AppendEntries, RequestVote, RequestVote, FSM replication, randomized elections",
    metric:"Quorum commit (N/2+1), election timeout 150-300ms, deterministic sim router" },
  { id:"broker", name:"Distributed Log Broker", lang:"Rust", tier:2, status:"STABLE",
    problem:"High-throughput message streams crash on GC pauses from unbounded allocations.",
    primitives:"Segmented append-only logs, lock-free SPSC ring buffer, CRC32C per-message",
    metric:"17 tests (13 unit + 4 integration), 20-byte binary frame header" },
  { id:"orchestrator", name:"Cloud Compute Orchestrator", lang:"Rust", tier:2, status:"STABLE",
    problem:"No actor-based distributed task scheduler for heterogeneous compute workloads.",
    primitives:"Actor model (tokio tasks + mpsc), SWIM gossip, OpenTelemetry OTLP",
    metric:"7 tests, Docker multi-stage build (8MB scratch), Terraform IaC" },
  { id:"gateway", name:"API Gateway & Reverse Proxy", lang:"Rust", tier:2, status:"STABLE",
    problem:"No unified TLS entry point for routing SQL/compute/cluster RPCs internally.",
    primitives:"axum (tokio), rustls TLS 1.3, 8 REST endpoints, CORS, trace middleware",
    metric:"10 service routes, CSP/HSTS/X-Frame-Options headers, Prometheus /metrics" },
  { id:"protocol", name:"Common IPC Protocol", lang:"Rust", tier:2, status:"STABLE",
    problem:"8 isolated crates each defined incompatible binary wire formats.",
    primitives:"30-byte frame (magic+len+ver+type+trace_id+bincode), 20 MessageType variants",
    metric:"1 roundtrip test, 16MB max frame with overflow protection, magic validation" },
  { id:"platform", name:"Platform Nodes Daemon", lang:"Rust", tier:2, status:"ACTIVE",
    problem:"No decentralized cluster health monitoring without a coordinator SPOF.",
    primitives:"SWIM gossip protocol (UDP), epoll HTTP proxy, LSM storage integration",
    metric:"UDP membership convergence <500ms, security headers on HTTP responses" },
  { id:"tauri", name:"Tauri Desktop Shell", lang:"Rust", tier:2, status:"STABLE",
    problem:"Electron is 120MB+ — needs systems-level desktop integration for Rust tools.",
    primitives:"Tauri 1.5, native IPC commands, system tray, menu bar, file dialogues",
    metric:"8MB binary footprint, hardened CSP + limited allowlist (dialog+shell only)" },
  { id:"admin", name:"Admin Tools TUI Dashboard", lang:"Rust", tier:3, status:"STABLE",
    problem:"No terminal dashboard for live cluster monitoring in headless environments.",
    primitives:"Zero-dependency HTTP client, hand-rolled JSON parser, ANSI escape TUI",
    metric:"11 tests, real-time 1s polling, color-coded SWIM peer status display" },
  { id:"render", name:"WGSL Compute Render Engine", lang:"Rust/WASM", tier:1, status:"ACTIVE",
    problem:"CPU-bound coordinate projection kills framerate at 1M+ point clouds.",
    primitives:"wgpu compute pipeline, WGSL spatial_transform shader, bytemuck zero-copy",
    metric:"1M points/dispatch, 256 threads/workgroup, 5 GPU buffers" },
  { id:"ingest", name:"Ingestion Server Pipeline", lang:"Rust", tier:2, status:"ACTIVE",
    problem:"TCP read() buffer copies consume 40% CPU at 10Gbps line rate on edge nodes.",
    primitives:"io_uring zero-copy recv, IngestBuffer (16MB), tokio async, Pipeline stats",
    metric:"port 8400, 16MB max frame, blocks_processed counter" },
  { id:"syncsvr", name:"Real-Time CRDT Sync Server", lang:"Rust", tier:3, status:"ACTIVE",
    problem:"WebSocket head-of-line blocking throttles concurrent CRDT delta sync at scale.",
    primitives:"QUIC stream multiplexing, SessionManager (256 peers), SyncEngine, delta merge",
    metric:"port 9400, 256 max peers, SyncDelta protocol, cross-platform fallback" },
];

export default function ProjectCards({ onSelect }: { onSelect: (id: string) => void }) {
  const [selected, setSelected] = useState<string | null>(null);
  const [tierFilter, setTierFilter] = useState<number | null>(null);

  const filtered = tierFilter ? PROJECTS.filter(p => p.tier === tierFilter) : PROJECTS;

  return (
    <div className="cyber-panel h-full flex flex-col overflow-hidden">
      <div className="p-4 pb-2 border-b border-border flex items-center justify-between">
        <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase">
          PROJECT REGISTRY
        </div>
        <div className="flex gap-1">
          {[null, 1, 2, 3].map(t => (
            <button key={t??"all"} onClick={() => setTierFilter(t)}
              className={`text-[7px] font-mono px-1.5 py-0.5 rounded border transition-all ${
                tierFilter===t ? "bg-gold-bg text-gold border-gold-border" : "text-text-muted border-transparent hover:text-text"
              }`}>
              {t ? `T${t}` : "ALL"}
            </button>
          ))}
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {filtered.map(p => (
          <button
            key={p.id}
            onClick={() => { setSelected(selected===p.id?null:p.id); onSelect(p.id); }}
            className={`w-full text-left p-3 rounded border transition-all ${
              selected===p.id ? "bg-bg border-border-hover" : "bg-bg/30 border-border/30 hover:border-border/50"
            }`}
          >
            <div className="flex items-center justify-between mb-1">
              <span className="text-[10px] font-mono font-bold text-text">{p.name}</span>
              <div className="flex items-center gap-1.5">
                <span className="text-[7px] font-mono text-text-muted">[{p.lang}]</span>
                <span className={`text-[7px] font-mono font-bold px-1 py-0.5 rounded ${p.status==="STABLE"?"bg-green/10 text-green":"bg-gold/10 text-gold"}`}>
                  {p.status}
                </span>
              </div>
            </div>
            {selected===p.id && (
              <div className="mt-2 space-y-1.5 text-[8px] font-mono leading-relaxed border-t border-border/50 pt-2">
                <div><span className="text-gold">Problem:</span> <span className="text-text-soft">{p.problem}</span></div>
                <div><span className="text-gold">Primitives:</span> <span className="text-text-soft">{p.primitives}</span></div>
                <div><span className="text-gold">Metric:</span> <span className="text-green">{p.metric}</span></div>
              </div>
            )}
          </button>
        ))}
      </div>
    </div>
  );
}
