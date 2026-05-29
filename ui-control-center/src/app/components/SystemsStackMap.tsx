"use client";

import { useEffect, useRef, useState } from "react";
import { Database, Network, Rocket, Cpu, Layers, HardDrive, Radio, Wifi } from "lucide-react";

interface CrateNode {
  id: string; label: string; tier: number; category: string;
  x: number; y: number; color: string;
  problem: string; primitives: string; metric: string;
}

const NODES: CrateNode[] = [
  { id:"lsm", label:"LSM Engine", tier:1, category:"Storage", x:60,y:80, color:"#3fb950", problem:"Row-locking causes DB timeouts during burst ingestion", primitives:"MemTable, SSTable, Compaction Pipeline", metric:"3.1:1 compression, 17 tests" },
  { id:"sql", label:"SQL Engine", tier:1, category:"Storage", x:200,y:80, color:"#58a6ff", problem:"No in-browser relational query parser exists", primitives:"Recursive Descent Parser, AST, Query Planner", metric:"6 statement types, 14 expression types" },
  { id:"columnar", label:"Columnar Engine", tier:1, category:"Storage", x:340,y:80, color:"#58a6ff", problem:"JSON serialization tax kills 60fps at scale", primitives:"bytemuck, LRU pool, vectorized scan", metric:"17 tests, 256MB heap cap" },
  { id:"lob", label:"LOB Engine", tier:1, category:"Storage", x:480,y:80, color:"#3fb950", problem:"Mutex-based matching kills throughput at 1M orders", primitives:"OrderPool, price-time priority, CAS", metric:"p50=200ns, 1M orders in 221ns avg" },
  { id:"telemetry", label:"Telemetry Agg", tier:1, category:"Storage", x:620,y:80, color:"#d2991d", problem:"Edge gateways OOM from unbounded UDP buffers", primitives:"Gorilla delta-of-delta, packet ring", metric:"3.1:1 ratio, 512KB ring, 256MB cap" },
  { id:"sensor", label:"Sensor Buffer", tier:1, category:"Storage", x:740,y:80, color:"#d2991d", problem:"LiDAR/Camera streams need deterministic merge", primitives:"MPMC CAS, Acquire/Release+fence", metric:"30K frames, TSAN data-race-free" },
  { id:"raft", label:"Raft KV", tier:2, category:"Distributed", x:60,y:190, color:"#8b5cf6", problem:"Network partitions corrupt replicated state", primitives:"AppendEntries, RequestVote, FSM", metric:"Quorum commit, election 150-300ms" },
  { id:"broker", label:"Log Broker", tier:2, category:"Distributed", x:200,y:190, color:"#d2991d", problem:"High-throughput streams crash on GC pauses", primitives:"Segmented logs, lock-free SPSC, CRC32C", metric:"17 tests, 20B frame header" },
  { id:"orchestrator", label:"Compute Orchestrator", tier:2, category:"Distributed", x:340,y:190, color:"#3fb950", problem:"No actor-based task scheduler for distributed work", primitives:"Actor model, SWIM gossip, OpenTelemetry", metric:"7 tests, Docker 8MB image" },
  { id:"gateway", label:"API Gateway", tier:2, category:"Distributed", x:480,y:190, color:"#8b5cf6", problem:"No TLS entry point for internal RPC routing", primitives:"axum, tokio, rustls, 8 REST routes", metric:"TLS 1.3, CSP headers, CORS" },
  { id:"protocol", label:"Common Protocol", tier:2, category:"Distributed", x:620,y:190, color:"#58a6ff", problem:"8 crates had incompatible message formats", primitives:"30B frames, 20 msg types, trace_id", metric:"1 test, 16MB max frame" },
  { id:"platform", label:"Platform Nodes", tier:2, category:"Distributed", x:740,y:190, color:"#8b5cf6", problem:"No decentralized cluster health monitoring", primitives:"SWIM gossip, epoll proxy, LSM storage", metric:"UDP membership, HTTP telemetry" },
  { id:"container", label:"Container Engine", tier:1, category:"Storage", x:60,y:300, color:"#f85149", problem:"Process isolation without Docker overhead", primitives:"clone(), cgroups v2, seccomp-BPF", metric:"~120 syscalls, 5 caps, NO_NEW_PRIVS" },
  { id:"core", label:"Core Sys", tier:1, category:"Storage", x:200,y:300, color:"#58a6ff", problem:"Thread coordination without mutex contention", primitives:"Lock-free SPSC, zero-alloc logger", metric:"50ns push/pop, criterion bench" },
  { id:"tauri", label:"Tauri Desktop", tier:2, category:"Distributed", x:340,y:300, color:"#d2991d", problem:"Electron is 120MB+, needs systems-level shell", primitives:"Tauri 1.5, IPC bridge, native menus", metric:"8MB binary, dialog+shell API" },
  { id:"admin", label:"Admin Tools", tier:3, category:"Capstone", x:400,y:400, color:"#d2991d", problem:"No terminal dashboard for cluster monitoring", primitives:"Zero-dep HTTP, hand-rolled JSON, ANSI TUI", metric:"11 tests, raw TCP client" },
  { id:"render", label:"Render Engine", tier:1, category:"Storage", x:480,y:300, color:"#d2991d", problem:"CPU-bound coordinate projection at 1M+ points", primitives:"wgpu compute, WGSL shader, GPU dispatch", metric:"1M points/dispatch, 256 threads/workgroup" },
];

const TIER_COLORS = ["#3fb950", "#8b5cf6", "#d2991d"];
const EDGES: [string, string][] = [
  ["capstone","raft"],["capstone","broker"],["capstone","orchestrator"],["capstone","gateway"],
  ["raft","lsm"],["raft","columnar"],["broker","lsm"],["orchestrator","container"],
  ["orchestrator","sensor"],["gateway","sql"],["sql","lsm"],["sql","raft"],
  ["protocol","raft"],["protocol","broker"],["render","columnar"],
];

export default function SystemsStackMap({ onSelect }: { onSelect: (id: string) => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const [hovered, setHovered] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);

  return (
    <div className="cyber-panel p-4 h-full flex flex-col overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 mb-3 border-b border-border">
        SYSTEMS STACK MAP
      </div>

      {["Tier 3: Capstone", "Tier 2: Distributed & Consensus", "Tier 1: Storage & Data"].map((tier, ti) => (
        <div key={tier} className="mb-4">
          <div className="text-[8px] font-mono text-text-muted font-bold tracking-wider uppercase mb-2 px-1"
               style={{ color: TIER_COLORS[ti] }}>
            {tier}
          </div>
          <div className="space-y-1">
            {NODES.filter(n => n.tier === 3 - ti).map(n => (
              <button
                key={n.id}
                onClick={() => { setSelected(n.id); onSelect(n.id); }}
                onMouseEnter={() => setHovered(n.id)}
                onMouseLeave={() => setHovered(null)}
                className={`w-full text-left px-2.5 py-2 rounded border transition-all ${
                  selected === n.id
                    ? "bg-bg border-border-hover"
                    : hovered === n.id
                    ? "bg-surface/50 border-border/50"
                    : "bg-transparent border-transparent"
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className="w-1.5 h-1.5 rounded-full shrink-0" style={{ background: n.color }} />
                  <span className="text-[10px] font-mono font-bold text-text">{n.label}</span>
                  <span className="text-[7px] font-mono text-text-muted">{n.category}</span>
                </div>
                {selected === n.id && (
                  <div className="mt-2 pl-4 text-[8px] font-mono leading-relaxed space-y-1 border-l-2 border-border/50 ml-1">
                    <div><span className="text-gold">Problem:</span> <span className="text-text-soft">{n.problem}</span></div>
                    <div><span className="text-gold">Primitives:</span> <span className="text-text-soft">{n.primitives}</span></div>
                    <div><span className="text-gold">Metric:</span> <span className="text-green">{n.metric}</span></div>
                  </div>
                )}
              </button>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
