"use client";

import { useState, useMemo, useEffect } from "react";
import { 
  Terminal, Key, Cpu, Network, Container, Database, FlaskConical, 
  Github, ExternalLink, Activity, HardDrive, Play, Check, 
  AlertCircle, RefreshCw, Layers
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { type NodeTelemetry } from "../utils/tauri";

// Import existing visualizers
import ViewContainerRuntime from "./ViewContainerRuntime";
import ViewClusterNodes from "./ViewClusterNodes";
import ViewSqlAnalyzer from "./ViewSqlAnalyzer";
import ViewChaosStudio from "./ViewChaosStudio";
import XTermTerminal from "./XTermTerminal";

interface ProjectWorkspaceProps {
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
  nodes: NodeTelemetry[];
  history: {
    cpu: Record<number, number[]>;
    memory: Record<number, number[]>;
    fd: Record<number, number[]>;
  };
}

export default function ProjectWorkspace({
  chaosMode,
  setChaosMode,
  nodes,
  history,
}: ProjectWorkspaceProps) {
  const [activeProjectId, setActiveProjectId] = useState("container");

  // Project data definition
  const projects = useMemo(() => [
    {
      id: "container",
      title: "Linux Container Runtime",
      subtitle: "clone() Namespace Isolation, cgroups v2, & seccomp-BPF",
      category: "Isolation & Runtimes",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/container-engine",
      status: "SIMULATED",
      lang: "Rust",
      icon: Container,
      color: "green",
      themeColor: "#3fb950",
      stats: { loc: "3.4K", coverage: "92%", size: "1.4MB", threads: "Single/Multi" },
      githubUrl: "https://github.com",
      description: "A production-grade Linux container runtime built entirely from first principles using direct Linux kernel APIs. Implements namespaces, cgroups v2 resource control, OverlayFS filesystems, and irreversible privilege drop order.",
      highlights: [
        "clone() with CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS | CLONE_NEWNET | CLONE_NEWIPC",
        "cgroups v2 limits: memory.max/high, cpu.max (CFS), io.max (rbps/riops)",
        "Security sequence: NO_NEW_PRIVS → drop 35+ capabilities → load seccomp BPF",
        "Pivot_root isolation with 11 masked paths and 4 read-only paths",
        "veth networking pairs with cbr0 bridge and iptables NAT MASQUERADE rules"
      ]
    },
    {
      id: "protocol",
      title: "Common IPC Protocol",
      subtitle: "Unified Binary Frame Protocol & Shared Types",
      category: "Messaging & Protocols",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/common-protocol",
      status: "ONLINE",
      lang: "Rust",
      icon: Layers,
      color: "blue",
      themeColor: "#58a6ff",
      stats: { loc: "1.1K", coverage: "95%", size: "420KB", threads: "N/A" },
      githubUrl: "https://github.com",
      description: "The unified inter-process communication layer connecting all 12 workspace crates. A 30-byte binary frame protocol with magic-byte validation, 20 message types covering SQL/Raft/Storage/Compute, and distributed trace ID propagation.",
      highlights: [
        "30-byte frame: [4B magic][4B len][2B ver][4B type][16B trace_id][bincode payload]",
        "20 MessageType variants: SqlQuery, RaftAppend, StoragePut, ComputeTask, BrokerProduce, etc.",
        "MessageEnvelope with trace_id propagation for distributed tracing across network hops",
        "FrameDecoder with 16MB max frame size, buffer overflow protection, magic validation",
        "Shared types: AppendEntriesArgs, PutRequest, MacroTask, NodeTelemetry, SqlQuery/Result"
      ]
    },
    {
      id: "gateway",
      title: "API Gateway & Reverse Proxy",
      subtitle: "HTTP/TLS 1.3 with axum on tokio",
      category: "Networking & Consensus",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/api-gateway",
      status: "ONLINE",
      lang: "Rust",
      icon: Network,
      color: "purple",
      themeColor: "#8b5cf6",
      stats: { loc: "1.0K", coverage: "N/A", size: "500KB", threads: "tokio async" },
      githubUrl: "https://github.com",
      description: "The public entry point for the entire distributed platform. An axum-based HTTP/TLS 1.3 server that routes SQL queries to the sql-engine, compute jobs to the orchestrator, and exposes cluster health, metrics, and telemetry endpoints.",
      highlights: [
        "8 REST routes: /v1/sql/query, /v1/jobs, /v1/cluster/nodes, /v1/cluster/health, /v1/metrics",
        "axum on tokio with CORS middleware and TraceLayer for request/response logging",
        "trace_id propagation via x-trace-id header for end-to-end distributed tracing",
        "Health check endpoints: /health (liveness), /ready (readiness)",
        "Prometheus-compatible /v1/metrics scraping endpoint"
      ]
    },
    {
      id: "sql",
      title: "SQL Query Engine & Parser",
      subtitle: "Recursive Descent Parser + Query Planner + Executor",
      category: "Data & Storage",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/sql-engine",
      status: "ONLINE",
      lang: "Rust",
      icon: Terminal,
      color: "blue",
      themeColor: "#58a6ff",
      stats: { loc: "1.5K", coverage: "N/A", size: "600KB", threads: "N/A" },
      githubUrl: "https://github.com",
      description: "A recursive descent SQL parser and query engine. Tokenizes SQL input into 25 token types, builds an AST with 6 statement variants and 14 expression types, plans queries via a QueryPlanner, and executes them against an in-memory Catalog.",
      highlights: [
        "Recursive descent parser: SELECT, INSERT, CREATE TABLE, DROP, DELETE, UPDATE",
        "25 token types: keywords, identifiers, string/int/float/bool literals, operators",
        "14 AST expression types: Column, Eq, Neq, Lt, Gt, And, Or, literals, etc.",
        "QueryPlanner: Statement → QueryPlan (6 variant types)",
        "QueryExecutor with Catalog: CREATE TABLE writes schema, SELECT reads data"
      ]
    },
    {
      id: "orchestrator",
      title: "Cloud Compute Orchestrator",
      subtitle: "Actor Model, SWIM Gossip & OpenTelemetry",
      category: "Distributed Systems",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/compute-orchestrator",
      status: "SIMULATED",
      lang: "Rust",
      icon: Cpu,
      color: "gold",
      themeColor: "#d2991d",
      stats: { loc: "2.2K", coverage: "85%", size: "1.8MB", threads: "tokio tasks" },
      githubUrl: "https://github.com",
      description: "A cloud-native distributed compute layer. Implements an actor-driven concurrency model with tokio tasks and mpsc mailboxes, SWIM gossip protocol for decentralized cluster membership, task scheduling with workload splitting, and OpenTelemetry tracing.",
      highlights: [
        "Actor system: tokio::spawn tasks with mpsc::channel mailboxes, ProcessId addressing",
        "SWIM gossip: Ping/Ack/PingReq protocol, Alive/Suspect/Dead states, metadata dissemination",
        "Task scheduler: MacroTask → MicroTask splitter, resource-aware node placement scoring",
        "OpenTelemetry OTLP export with tonic, distributed trace context propagation",
        "Docker multi-stage build (scratch, ~8MB) + Terraform IaC (VPC, EC2, SG) + GitHub Actions CD"
      ]
    },
    {
      id: "broker",
      title: "Distributed Log Broker",
      subtitle: "Lock-Free SPSC Ring Buffers & Binary TCP Protocol",
      category: "Messaging & Protocols",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/log-broker",
      status: "ONLINE",
      lang: "Rust",
      icon: Database,
      color: "gold",
      themeColor: "#d2991d",
      stats: { loc: "1.8K", coverage: "88%", size: "820KB", threads: "2 (SPSC)" },
      githubUrl: "https://github.com",
      description: "A high-capacity real-time pub-sub messaging engine. Features segmented append-only commit logs, pre-allocated index slabs, a custom binary TCP protocol, and zero-allocation socket streams.",
      highlights: [
        "Segmented append-only files ({base_offset:020}.log) with CRC32C checks",
        "Lock-free SPSC ring buffer using AtomicUsize head/tail and Acquire/Release fences",
        "Pre-allocated 10K-entry index slabs with O(log n) binary search lookup",
        "Custom binary TCP protocol with 16-byte fixed headers",
        "Non-blocking socket event loop with mio-based connection FSM state machine"
      ]
    },
    {
      id: "consensus",
      title: "Cluster Consensus Engine",
      subtitle: "SWIM Gossip Membership & Raft Log Replication",
      category: "Distributed Systems",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/platform-nodes",
      status: "SIMULATED",
      lang: "Rust",
      icon: Network,
      color: "purple",
      themeColor: "#8b5cf6",
      stats: { loc: "4.2K", coverage: "85%", size: "2.1MB", threads: "Event-driven" },
      githubUrl: "https://github.com",
      description: "A full distributed cluster runtime implementing SWIM Gossip membership failure detection, Raft replication logs, and an epoll-based low-latency HTTP control proxy.",
      highlights: [
        "SWIM Gossip membership convergence under 500ms for failure detection",
        "Raft election safety and append entry replication log state machine",
        "epoll event-driven proxy server with zero memory allocations on hot path",
        "Animated bezier replication map visualizing active consensus pulse particles"
      ]
    },
    {
      id: "raft",
      title: "Raft Distributed KV Store",
      subtitle: "Quorum Commit Replication & Network Split-Brain Chaos",
      category: "Distributed Systems",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/raft-kv",
      status: "SIMULATED",
      lang: "Rust",
      icon: Network,
      color: "purple",
      themeColor: "#8b5cf6",
      stats: { loc: "1.2K", coverage: "91%", size: "540KB", threads: "Tokio Async Tasks" },
      githubUrl: "https://github.com",
      description: "A fault-tolerant cluster of independent key-value servers that coordinate mutations using the Raft consensus algorithm. It guarantees strict linearizability and data durability even during split-brain partitions and node drops.",
      highlights: [
        "Explicit 3-state FSM representing Follower, Candidate, and Leader roles",
        "Randomized election timeouts (150ms-300ms) ensuring fast election convergence",
        "RPC layer schemas for RequestVote and AppendEntries payloads",
        "Replication log commitment requiring strict majority consensus quorum (N/2 + 1)",
        "Deterministic memory channel network router simulating latency, drops, and partitions"
      ]
    },
    {
      id: "storage",
      title: "LSM SQL Storage Engine",
      subtitle: "Interactive B+Tree Page Map & LSM Compaction",
      category: "Data & Storage",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/lsm-engine",
      status: "SIMULATED",
      lang: "Rust",
      icon: Layers,
      color: "blue",
      themeColor: "#58a6ff",
      stats: { loc: "2.9K", coverage: "90%", size: "1.1MB", threads: "Compactor Thread" },
      githubUrl: "https://github.com",
      description: "A dual-mode database engine. KV mode handles PUT/GET/DELETE operations updating MemTable SSTable pipelines; SQL mode parses and executes relational queries in memory.",
      highlights: [
        "LSM Storage pipeline: MemTable → L0 SSTable → L1 SSTable compaction",
        "SQL Query engine with SELECT, INSERT, and CREATE TABLE parsing",
        "Canvas-rendered B+Tree page map displaying root, internal, and leaf node traversal",
        "Page-cache tracking with amber dirty glows and blue active node highlights"
      ]
    },
    {
      id: "chaos",
      title: "Chaos Studio & Terminal",
      subtitle: "Failure Injection Framework & Virtualized Log Stream",
      category: "Systems Tools",
      path: "/c/Users/keaga/OneDrive/Documents/Main Project App/ui-control-center",
      status: "ONLINE",
      lang: "React",
      icon: FlaskConical,
      color: "cyan",
      themeColor: "#00f0ff",
      stats: { loc: "2.5K", coverage: "80%", size: "N/A", threads: "Web Worker" },
      githubUrl: "https://github.com",
      description: "A chaos engineering workbench providing controlled network partition split injection, malformed IPC frames injection, node crashes, and stress testing. Integrates xterm.js.",
      highlights: [
        "Four interactive chaos toggles triggering instant cluster failure modes",
        "High-performance virtualized log stream rendering up to 10K events (react-window)",
        "xterm.js embedded terminal terminal providing interactive CLI utilities",
        "Dynamic burst rates adapting telemetry poll load based on fuzzer activation"
      ]
    },
    {
      id: "shell",
      title: "Custom UNIX Shell",
      subtitle: "Process Redirections, Pipes, & Background Job Control",
      category: "Systems Tools",
      path: "/c/Users/keaga/OneDrive/Documents/Project 3 Rust",
      status: "ONLINE",
      lang: "Rust",
      icon: Terminal,
      color: "cyan",
      themeColor: "#00f0ff",
      stats: { loc: "1.2K", coverage: "86%", size: "480KB", threads: "Async Reaper" },
      githubUrl: "https://github.com",
      description: "A custom POSIX-like command shell written in Rust. Features pipeline redirection chaining, environment variable expansion, and a background task execution manager.",
      highlights: [
        "shlex tokenization preserving nested double and single quotes",
        "Standard file descriptor chaining (dup2) mapping pipes between child processes",
        "I/O redirects (< input, > output, >> append, 2> stderr)",
        "Interactive async job manager using non-blocking child.try_wait() reaping",
        "Ctrl+C SIGINT signal trapping avoiding shell exit while running programs"
      ]
    },
    {
      id: "bitcask",
      title: "Bitcask KV Database",
      subtitle: "Memory-Mapped I/O, CRC32C, & Compaction Compaction",
      category: "Data & Storage",
      path: "/c/Users/keaga/OneDrive/Documents/Project 4 Rust",
      status: "ONLINE",
      lang: "Rust",
      icon: Key,
      color: "gold",
      themeColor: "#d2991d",
      stats: { loc: "1.5K", coverage: "94%", size: "680KB", threads: "Compaction Worker" },
      githubUrl: "https://github.com",
      description: "A high-performance log-structured key-value storage engine in Rust. Uses append-only log files for O(1) writes, a fast in-memory KeyDir hash map index for O(1) reads, and automated log compaction merging.",
      highlights: [
        "Memory-mapped files (memmap2) bypassing kernel buffers for low-latency reads",
        "CRC32Fast checksum validation protecting log data integrity from disk rot",
        "KeyDir index hash map storing offsets, value sizes, and timestamps",
        "Bootstrapping serialization saving the index map for sub-millisecond startups",
        "Ratatui TUI dashboard rendering real-time metrics and database operations"
      ]
    },
    {
      id: "async",
      title: "Async Systems Runtime",
      subtitle: "Work-Stealing Executor & Epoll Net Reactor",
      category: "Networking & Consensus",
      path: "/c/Users/keaga/OneDrive/Documents/Project 5 Rust",
      status: "ONLINE",
      lang: "Rust",
      icon: Cpu,
      color: "purple",
      themeColor: "#8b5cf6",
      stats: { loc: "2.8K", coverage: "84%", size: "1.2MB", threads: "4 Worker Threads" },
      githubUrl: "https://github.com",
      description: "A custom async/await programming runtime built on raw epoll sockets. Includes a work-stealing thread-pool Executor, a non-blocking network Reactor, and an HTTP load balancer proxy.",
      highlights: [
        "Thread-pool Executor using crossbeam work-stealing queues for load balancing",
        "Reactor multiplexing I/O events using raw mio/epoll registration loops",
        "HTTP/1.1 socket parsing pipeline with zero-copy header validation",
        "TCP reverse proxy distributing incoming socket descriptors round-robin"
      ]
    }
  ], []);

  const activeProject = useMemo(() => {
    return projects.find((p) => p.id === activeProjectId) || projects[0];
  }, [activeProjectId, projects]);

  return (
    <section id="workspace" className="section relative">
      <div className="absolute top-0 right-1/4 w-[400px] h-[300px] rounded-full bg-blue/5 blur-[120px] pointer-events-none" />
      <div className="absolute bottom-10 left-10 w-[300px] h-[200px] rounded-full bg-gold/5 blur-[100px] pointer-events-none" />

      <div className="section-heading">Systems Console</div>
      <h2 className="section-title">Projects Workspace</h2>
      <p className="text-text-soft text-base max-w-2xl mb-12">
        An interactive developer workspace linking production systems code to live simulated consoles. 
        Select a project from the registry tree to inspect metadata, highlights, and run live interactive visualizers.
      </p>

      <div className="flex flex-col lg:flex-row gap-6 items-start">
        {/* LEFT COLUMN: Project Selector Tree */}
        <div className="w-full lg:w-[280px] shrink-0 space-y-4">
          <div className="cyber-panel p-4">
            <div className="text-[10px] font-mono font-bold text-text-muted tracking-wider uppercase mb-3 pb-2 border-b border-border">
              PROJECT REGISTRY
            </div>
            <div className="space-y-4">
              {/* Group projects by category */}
              {Array.from(new Set(projects.map((p) => p.category))).map((cat) => (
                <div key={cat} className="space-y-1">
                  <div className="text-[9px] font-mono font-bold text-gold/80 tracking-widest uppercase pl-1.5">
                    {cat}
                  </div>
                  <div className="space-y-0.5">
                    {projects
                      .filter((p) => p.category === cat)
                      .map((p) => {
                        const Icon = p.icon;
                        const isActive = p.id === activeProjectId;
                        return (
                          <button
                            key={p.id}
                            onClick={() => setActiveProjectId(p.id)}
                            className={`w-full flex items-center justify-between px-2.5 py-2 rounded-md text-left transition-all ${
                              isActive
                                ? "bg-gold-bg text-gold border border-gold-border/40 shadow-sm"
                                : "text-text-soft hover:text-text hover:bg-surface border border-transparent"
                            }`}
                          >
                            <div className="flex items-center gap-2 min-w-0">
                              <Icon size={13} className={isActive ? "text-gold" : "text-text-soft"} />
                              <span className="text-[11px] font-mono font-semibold truncate">
                                {p.title.replace("Linux ", "").replace("LSM ", "").replace("Async ", "").replace("Docker ", "").replace("Raft ", "")}
                              </span>
                            </div>
                            <div className="flex items-center gap-1.5 shrink-0 pl-1">
                              <span className="w-1 h-1 rounded-full bg-green animate-pulse-subtle" />
                              <span className="text-[8px] font-mono text-text-muted font-bold">
                                {p.lang}
                              </span>
                            </div>
                          </button>
                        );
                      })}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Developer status card */}
          <div className="cyber-panel p-4 text-[10px] font-mono text-text-soft leading-relaxed space-y-2">
            <div className="text-[10px] font-bold text-text mb-1 border-b border-border pb-1.5 flex items-center justify-between">
              <span>WORKSPACE STATUS</span>
              <span className="inline-flex w-2 h-2 rounded-full bg-green animate-pulse" />
            </div>
            <div>Target: <span className="text-text">x86_64-unknown-linux-gnu</span></div>
            <div>Compiler: <span className="text-text">rustc 1.80.0-nightly</span></div>
            <div>SDK Interface: <span className="text-text">Tauri IPC Bridge</span></div>
            <div>Working Dir: <span className="text-blue break-all">/c/Users/keaga/OneDrive/Documents/</span></div>
          </div>
        </div>

        {/* RIGHT COLUMN: Project Details & Active Visualizer */}
        <div className="flex-1 min-w-0 w-full space-y-6">
          {/* Metadata Header Block */}
          <div className="cyber-panel p-6">
            <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 mb-4 pb-4 border-b border-border">
              <div>
                <span className="text-[9px] font-mono font-bold text-gold px-2 py-0.5 rounded bg-gold-bg border border-gold-border/40 uppercase tracking-widest">
                  {activeProject.category}
                </span>
                <h3 className="text-2xl font-extrabold text-text mt-1.5">
                  {activeProject.title}
                </h3>
                <p className="text-xs text-text-soft font-mono mt-0.5">
                  {activeProject.subtitle}
                </p>
              </div>

              <div className="flex items-center gap-3 shrink-0">
                <a
                  href={activeProject.githubUrl}
                  target="_blank"
                  className="flex items-center gap-1.5 px-3 py-1.5 bg-surface border border-border rounded-md text-[10px] font-mono font-bold text-text-soft hover:text-text hover:border-border-hover transition-colors"
                >
                  <Github size={12} /> Source
                </a>
                <span className="text-[10px] font-mono text-text-muted select-none">|</span>
                <div className="flex items-center gap-1.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-green animate-pulse-subtle" />
                  <span className="text-[9px] font-mono font-bold text-green tracking-wider uppercase">
                    {activeProject.status}
                  </span>
                </div>
              </div>
            </div>

            {/* Stats matrix grid */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
              <div className="bg-bg/40 border border-border/30 rounded-lg p-2.5">
                <span className="text-[9px] font-mono text-text-muted block">SOURCE_LOC</span>
                <span className="text-[13px] font-mono font-bold text-text">{activeProject.stats.loc} Lines</span>
              </div>
              <div className="bg-bg/40 border border-border/30 rounded-lg p-2.5">
                <span className="text-[9px] font-mono text-text-muted block">TEST_COVERAGE</span>
                <span className="text-[13px] font-mono font-bold text-green">{activeProject.stats.coverage}</span>
              </div>
              <div className="bg-bg/40 border border-border/30 rounded-lg p-2.5">
                <span className="text-[9px] font-mono text-text-muted block">BINARY_FOOTPRINT</span>
                <span className="text-[13px] font-mono font-bold text-gold">{activeProject.stats.size}</span>
              </div>
              <div className="bg-bg/40 border border-border/30 rounded-lg p-2.5">
                <span className="text-[9px] font-mono text-text-muted block">CONCURRENCY_MODEL</span>
                <span className="text-[13px] font-mono font-bold text-blue">{activeProject.stats.threads}</span>
              </div>
            </div>

            <div className="grid md:grid-cols-12 gap-6">
              <div className="md:col-span-7 space-y-4">
                <div>
                  <h4 className="text-[10px] font-mono font-bold text-text-muted tracking-wider uppercase mb-2">
                    PROJECT_DESCRIPTION
                  </h4>
                  <p className="text-sm text-text-soft leading-relaxed">
                    {activeProject.description}
                  </p>
                </div>
                {activeProject.id === "raft" && (
                  <div className="border border-border/40 rounded-lg overflow-hidden bg-bg/20 p-2.5">
                    <span className="text-[8px] font-mono text-text-muted block mb-1">RAFT_CONSTRUCTS_FLOW.PNG</span>
                    <img src="/raft_consensus_flow.png" alt="Raft Protocol Flow" className="w-full h-auto rounded opacity-80 border border-border/30" />
                  </div>
                )}
                <div className="text-[10px] font-mono text-text-muted bg-bg/50 border border-border/50 rounded-md px-3 py-2 flex items-center gap-2">
                  <span className="text-gold">📂</span>
                  <span className="truncate" title={activeProject.path}>{activeProject.path}</span>
                </div>
              </div>

              <div className="md:col-span-5">
                <h4 className="text-[10px] font-mono font-bold text-text-muted tracking-wider uppercase mb-2">
                  TECHNICAL_HIGHLIGHTS
                </h4>
                <ul className="space-y-2">
                  {activeProject.highlights.map((h, i) => (
                    <li key={i} className="flex items-start gap-2 text-xs text-text-soft">
                      <span className="text-gold mt-0.5 shrink-0">▸</span>
                      <span>{h}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>

          {/* ACTIVE SIMULATION INTERACTION PANEL */}
          <div className="cyber-panel p-5 relative overflow-hidden min-h-[380px]">
            <div className="text-[10px] font-mono font-bold text-text-soft tracking-wider mb-4 pb-2 border-b border-border/40 flex items-center justify-between">
              <span className="flex items-center gap-1.5">
                <Activity size={12} className="text-blue animate-pulse-subtle" />
                ACTIVE_WIDGET // INTERACTIVE_SIMULATOR
              </span>
              <span className="text-[8px] font-mono text-text-muted uppercase">
                {activeProject.title} Output stream
              </span>
            </div>

            <AnimatePresence mode="wait">
              <motion.div
                key={activeProjectId}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                transition={{ duration: 0.2 }}
                className="w-full h-full"
              >
                {activeProjectId === "container" && (
                  <ViewContainerRuntime chaosMode={chaosMode} />
                )}
                {activeProjectId === "broker" && (
                  <div className="cyber-panel p-6 min-h-[300px] flex flex-col items-center justify-center text-center">
                    <div className="text-4xl mb-4">📡</div>
                    <h4 className="text-lg font-bold text-text mb-2">Distributed Log Broker</h4>
                    <p className="text-sm text-text-soft max-w-md mb-4">
                      A high-capacity messaging engine with segmented append-only logs,
                      lock-free ring buffers, and a custom binary TCP protocol — built
                      in Rust with mio and parking_lot.
                    </p>
                    <div className="flex flex-wrap gap-1.5 justify-center mb-4">
                      {["Rust", "mio", "TCP", "Lock-free SPSC", "CRC32C", "Binary Protocol", "Consumer Cursors"].map((t) => (
                        <span key={t} className="tag">{t}</span>
                      ))}
                    </div>
                    <div className="grid grid-cols-3 gap-4 text-xs font-mono text-text-muted">
                      <div className="text-center">
                        <div className="text-green text-lg font-bold">17</div>
                        <div>Unit Tests</div>
                      </div>
                      <div className="text-center">
                        <div className="text-gold text-lg font-bold">12</div>
                        <div>Source Files</div>
                      </div>
                      <div className="text-center">
                        <div className="text-blue text-lg font-bold">20B</div>
                        <div>Frame Header</div>
                      </div>
                    </div>
                  </div>
                )}
                {activeProjectId === "consensus" && (
                  <ViewClusterNodes nodes={nodes} history={history} chaosMode={chaosMode} />
                )}
                {activeProjectId === "raft" && (
                  <RaftSimulator />
                )}
                {activeProjectId === "storage" && (
                  <ViewSqlAnalyzer chaosMode={chaosMode} />
                )}
                {activeProjectId === "chaos" && (
                  <div className="flex flex-col gap-5">
                    <ViewChaosStudio chaosMode={chaosMode} setChaosMode={setChaosMode} />
                    <div className="h-[260px]">
                      <XTermTerminal chaosMode={chaosMode} />
                    </div>
                  </div>
                )}
                {activeProjectId === "shell" && (
                  <ShellSimulator />
                )}
                {activeProjectId === "bitcask" && (
                  <BitcaskSimulator />
                )}
                {activeProjectId === "async" && (
                  <AsyncRuntimeSimulator />
                )}
              </motion.div>
            </AnimatePresence>
          </div>
        </div>
      </div>
    </section>
  );
}

// ==========================================
// 1. SHELL PIPELINE SIMULATOR
// ==========================================
function ShellSimulator() {
  const samples = [
    {
      cmd: "cat log.txt | grep 'OOM' | wc -l > oom_count.txt",
      desc: "Count OOM events and write to file"
    },
    {
      cmd: "cargo build --release && ./target/release/custom_shell &",
      desc: "Compile workspace and launch in background"
    },
    {
      cmd: "tail -f syslog.log | grep --line-buffered 'error' >> errors.log",
      desc: "Append errors live using line-buffered grep"
    }
  ];

  const [inputCommand, setInputCommand] = useState(samples[0].cmd);
  const [step, setStep] = useState(0);
  const [animating, setAnimating] = useState(false);

  // Parse command visual blocks
  const parsedPipeline = useMemo(() => {
    // Very basic parsing for visuals
    const hasBg = inputCommand.trim().endsWith("&");
    const cleanCmd = hasBg ? inputCommand.trim().slice(0, -1).trim() : inputCommand.trim();
    
    // Find output redirection
    let redirectOut: string | null = null;
    let redirectIn: string | null = null;
    let mainPipeline = cleanCmd;

    if (mainPipeline.includes(">")) {
      const parts = mainPipeline.split(">");
      mainPipeline = parts[0].trim();
      redirectOut = parts[1].trim();
    }
    if (mainPipeline.includes("<")) {
      const parts = mainPipeline.split("<");
      mainPipeline = parts[0].trim();
      redirectIn = parts[1].trim();
    }

    const commands = mainPipeline.split("|").map((c) => {
      const tokens = c.trim().split(/\s+/);
      return {
        binary: tokens[0],
        args: tokens.slice(1).join(" ")
      };
    });

    return { commands, redirectIn, redirectOut, hasBg };
  }, [inputCommand]);

  const triggerAnimation = () => {
    setStep(0);
    setAnimating(true);
    let currentStep = 0;
    const interval = setInterval(() => {
      currentStep += 1;
      setStep(currentStep);
      if (currentStep >= 4) {
        clearInterval(interval);
        setAnimating(false);
      }
    }, 1200);
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <span className="text-[10px] font-mono text-text-soft">SELECT PRESET COMMAND:</span>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
          {samples.map((s, idx) => (
            <button
              key={idx}
              onClick={() => { setInputCommand(s.cmd); setStep(0); }}
              className={`p-2 rounded-md border text-left font-mono text-[9px] transition-all ${
                inputCommand === s.cmd
                  ? "bg-blue-bg border-blue/40 text-blue"
                  : "bg-surface/50 border-border/50 text-text-soft hover:text-text"
              }`}
            >
              <div className="font-bold truncate">{s.cmd}</div>
              <div className="text-[8px] opacity-70 mt-0.5">{s.desc}</div>
            </button>
          ))}
        </div>
      </div>

      <div className="flex gap-2">
        <input
          type="text"
          value={inputCommand}
          onChange={(e) => { setInputCommand(e.target.value); setStep(0); }}
          className="flex-1 bg-bg border border-border rounded-md px-3 py-2 text-xs font-mono text-text placeholder-text-muted focus:outline-none focus:border-blue-border"
          placeholder="Type a Unix pipeline..."
        />
        <button
          onClick={triggerAnimation}
          disabled={animating || !inputCommand.trim()}
          className="flex items-center gap-1.5 px-4 py-2 bg-blue text-white text-[10px] font-mono font-bold rounded-md hover:bg-blue/90 disabled:opacity-40 transition-colors"
        >
          <Play size={12} /> RUN_PIPELINE
        </button>
      </div>

      {/* Visual representation card */}
      <div className="cyber-panel bg-bg/50 p-4 border border-border/60 min-h-[160px] flex flex-col justify-between">
        {/* Step-by-step parser status */}
        <div className="grid grid-cols-4 gap-2 mb-6 text-[9px] font-mono text-center">
          {[
            { label: "1. Tokenize", desc: "shlex syntax parse" },
            { label: "2. Redirect", desc: "Setup stdio files" },
            { label: "3. Pipes (dup2)", desc: "Chain file desc" },
            { label: "4. Exec & Reap", desc: "fork() & try_wait()" }
          ].map((s, idx) => (
            <div
              key={idx}
              className={`p-1.5 rounded border transition-all ${
                step > idx
                  ? "bg-green-bg border-green-border text-green font-bold"
                  : step === idx
                  ? "bg-blue-bg border-blue text-blue font-bold animate-pulse-subtle"
                  : "bg-surface/30 border-border/30 text-text-muted"
              }`}
            >
              <div>{s.label}</div>
              <div className="text-[7px] opacity-70 font-normal">{s.desc}</div>
            </div>
          ))}
        </div>

        {/* Command chaining diagram */}
        <div className="flex flex-wrap items-center justify-center gap-3 p-4 bg-surface/30 rounded-md border border-border/40 min-h-[80px]">
          {parsedPipeline.redirectIn && (
            <div className="flex items-center gap-1 shrink-0">
              <span className="px-2 py-1 bg-purple-bg border border-purple-border text-purple rounded text-[9px] font-mono font-bold">
                📄 {parsedPipeline.redirectIn}
              </span>
              <span className="text-text-muted text-[10px]">→ (stdin)</span>
            </div>
          )}

          {parsedPipeline.commands.map((cmd, i) => (
            <div key={i} className="flex items-center gap-2 shrink-0">
              {i > 0 && (
                <div className="flex flex-col items-center">
                  <span className="text-[8px] font-mono text-gold font-bold">pipe()</span>
                  <span className="text-text-muted text-[10px]">🡲</span>
                </div>
              )}
              <div className="p-2 bg-blue-bg/20 border border-blue-border/40 rounded text-center font-mono">
                <div className="text-[10px] text-blue font-bold">{cmd.binary}</div>
                {cmd.args && (
                  <div className="text-[8px] text-text-soft mt-0.5 truncate max-w-[120px]">
                    {cmd.args}
                  </div>
                )}
              </div>
            </div>
          ))}

          {parsedPipeline.redirectOut && (
            <div className="flex items-center gap-1 shrink-0">
              <span className="text-text-muted text-[10px]">(stdout) →</span>
              <span className="px-2 py-1 bg-purple-bg border border-purple-border text-purple rounded text-[9px] font-mono font-bold">
                📄 {parsedPipeline.redirectOut}
              </span>
            </div>
          )}

          {parsedPipeline.hasBg && (
            <div className="ml-2 px-1.5 py-0.5 bg-gold-bg border border-gold-border text-gold rounded text-[8px] font-mono font-bold animate-pulse">
              & BACKGROUND
            </div>
          )}
        </div>

        {/* Pipeline output console */}
        <div className="bg-bg border border-border/60 rounded p-2 text-[9px] font-mono text-green leading-normal h-[60px] overflow-y-auto mt-4">
          {step === 0 && <div className="text-text-muted">// Ready to simulate pipeline. Press RUN_PIPELINE.</div>}
          {step === 1 && <div>$ shlex::split(&quot;{inputCommand}&quot;) parsed {parsedPipeline.commands.length} processes successfully.</div>}
          {step === 2 && (
            <div>
              {parsedPipeline.redirectIn ? `* Redirecting stdin to open(&quot;${parsedPipeline.redirectIn}&quot;)` : "* Inheriting standard stdin"}
              <br />
              {parsedPipeline.redirectOut ? `* Redirecting stdout to open(&quot;${parsedPipeline.redirectOut}&quot;, Create|Write)` : "* Inheriting standard stdout"}
            </div>
          )}
          {step === 3 && (
            <div>
              {parsedPipeline.commands.length > 1
                ? `* Initializing ${parsedPipeline.commands.length - 1} OS pipes. Mapping write descriptor to next read descriptor via dup2().`
                : "* No pipeline sockets needed. Single command spawned."}
            </div>
          )}
          {step === 4 && (
            <div>
              {parsedPipeline.hasBg
                ? `[1] ${10000 + Math.floor(Math.random() * 40000)} spawned in background. Reaping via non-blocking try_wait().`
                : "* Executing commands. Waiting on pipeline termination... Exit code: 0 (Success)."}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ==========================================
// 2. BITCASK KV STORAGE ENGINE SIMULATOR
// ==========================================
function BitcaskSimulator() {
  const [db, setDb] = useState<Record<string, { val: string; offset: number; len: number; timestamp: number }>>({
    "port": { val: "8080", offset: 0, len: 32, timestamp: Date.now() - 10000 },
    "max_conn": { val: "5000", offset: 32, len: 36, timestamp: Date.now() - 8000 }
  });
  const [appendLog, setAppendLog] = useState<Array<{ key: string; val: string; isDead: boolean; offset: number; timestamp: number }>>([
    { key: "port", val: "8080", isDead: false, offset: 0, timestamp: Date.now() - 10000 },
    { key: "max_conn", val: "5000", isDead: false, offset: 32, timestamp: Date.now() - 8000 }
  ]);
  
  const [formKey, setFormKey] = useState("");
  const [formVal, setFormVal] = useState("");
  const [terminalMsg, setTerminalMsg] = useState("// Bitcask storage initialized. Write path Appends to log; Read path Index-looks.");
  const [isCompacting, setIsCompacting] = useState(false);

  const handlePut = (e: React.FormEvent) => {
    e.preventDefault();
    if (!formKey.trim() || !formVal.trim()) return;

    const lastOffset = appendLog.length > 0 ? appendLog[appendLog.length - 1].offset + 32 : 0;
    const timestamp = Date.now();

    // Mark previous instances of this key as DEAD in appendLog
    const updatedLog = appendLog.map(item => {
      if (item.key === formKey) return { ...item, isDead: true };
      return item;
    });

    const newLogItem = { key: formKey, val: formVal, isDead: false, offset: lastOffset, timestamp };
    const nextLog = [...updatedLog, newLogItem];

    setAppendLog(nextLog);
    setDb(prev => ({
      ...prev,
      [formKey]: { val: formVal, offset: lastOffset, len: 32, timestamp }
    }));
    setTerminalMsg(`PUT key="${formKey}" val="${formVal}" -> Appended to segment offset ${lastOffset}. KeyDir index updated.`);
    setFormKey("");
    setFormVal("");
  };

  const handleDelete = (key: string) => {
    const lastOffset = appendLog.length > 0 ? appendLog[appendLog.length - 1].offset + 32 : 0;
    const timestamp = Date.now();

    // Mark previous instances of this key as DEAD in appendLog
    const updatedLog = appendLog.map(item => {
      if (item.key === key) return { ...item, isDead: true };
      return item;
    });

    // Append Tombstone log entry (val = "" or special tombstone indicator)
    const tombstoneItem = { key, val: "[TOMBSTONE]", isDead: true, offset: lastOffset, timestamp };
    setAppendLog([...updatedLog, tombstoneItem]);

    const updatedDb = { ...db };
    delete updatedDb[key];
    setDb(updatedDb);

    setTerminalMsg(`DELETE key="${key}" -> Appended Tombstone log entry at offset ${lastOffset}. KeyDir index entry purged.`);
  };

  const handleMerge = () => {
    setIsCompacting(true);
    setTerminalMsg("BACKGROUND WORKER: Initiating Bitcask compaction merge... Scanning KeyDir...");
    
    setTimeout(() => {
      // Re-write only active keys in KeyDir to a new compacted log segment
      let currentOffset = 0;
      const compactedLog: typeof appendLog = [];
      const compactedDb: typeof db = {};

      Object.entries(db).forEach(([k, v]) => {
        compactedLog.push({
          key: k,
          val: v.val,
          isDead: false,
          offset: currentOffset,
          timestamp: v.timestamp
        });
        compactedDb[k] = {
          val: v.val,
          offset: currentOffset,
          len: 32,
          timestamp: v.timestamp
        };
        currentOffset += 32;
      });

      setAppendLog(compactedLog);
      setDb(compactedDb);
      setIsCompacting(false);
      setTerminalMsg(`BACKGROUND WORKER: Compaction merge finished. Reclaimed ${appendLog.length - compactedLog.length} stale segments. New Active File size: ${currentOffset}B.`);
    }, 1500);
  };

  return (
    <div className="space-y-4">
      {/* Forms & Compaction button */}
      <div className="flex flex-col md:flex-row gap-4 items-start justify-between">
        <form onSubmit={handlePut} className="flex gap-2 w-full md:max-w-md">
          <input
            type="text"
            value={formKey}
            onChange={(e) => setFormKey(e.target.value)}
            placeholder="Key"
            className="flex-1 bg-bg border border-border rounded px-2 py-1.5 text-xs font-mono text-text outline-none focus:border-blue/50"
          />
          <input
            type="text"
            value={formVal}
            onChange={(e) => setFormVal(e.target.value)}
            placeholder="Value"
            className="flex-1 bg-bg border border-border rounded px-2 py-1.5 text-xs font-mono text-text outline-none focus:border-blue/50"
          />
          <button
            type="submit"
            className="bg-blue-bg border border-blue-border text-blue text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-blue/10 transition-all shrink-0"
          >
            SET KEY
          </button>
        </form>

        <button
          onClick={handleMerge}
          disabled={isCompacting || appendLog.length === Object.keys(db).length}
          className="flex items-center gap-1.5 bg-gold-bg border border-gold-border/40 text-gold text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-gold/10 transition-all disabled:opacity-40"
        >
          <RefreshCw size={11} className={isCompacting ? "animate-spin" : ""} />
          MERGE_COMPACTION
        </button>
      </div>

      <div className="grid md:grid-cols-2 gap-4">
        {/* Left: Append-Only Active Log File (on Disk) */}
        <div className="cyber-panel p-3">
          <div className="text-[9px] font-mono font-bold text-text-soft tracking-wider mb-2 border-b border-border/50 pb-1 flex items-center justify-between">
            <span>ACTIVE_SEGMENT.DATA (ON_DISK)</span>
            <span className="text-[8px] text-text-muted font-bold">APPEND-ONLY</span>
          </div>

          <div className="space-y-1.5 max-h-[140px] overflow-y-auto pr-1">
            {appendLog.map((log, idx) => (
              <div
                key={idx}
                className={`p-2 rounded border text-[9px] font-mono flex items-center justify-between ${
                  log.isDead
                    ? "bg-red-bg/20 border-red-border/30 text-text-muted line-through"
                    : "bg-surface/50 border-border text-text"
                }`}
              >
                <div>
                  <span className="text-text-muted">Offset {log.offset}B:</span>{" "}
                  <span className="text-blue font-bold">{log.key}</span>
                  <span className="text-text-soft"> → {log.val}</span>
                </div>
                <div className="flex items-center gap-1.5">
                  <span className="text-[7px] text-text-muted">CRC32C</span>
                  {log.isDead ? (
                    <span className="text-red font-bold">[STALE]</span>
                  ) : (
                    <span className="text-green font-bold">[ACTIVE]</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Right: KeyDir Index Map (in Memory) */}
        <div className="cyber-panel p-3">
          <div className="text-[9px] font-mono font-bold text-text-soft tracking-wider mb-2 border-b border-border/50 pb-1 flex items-center justify-between">
            <span>KEYDIR_INDEX_MAP (RAM)</span>
            <span className="text-[8px] text-text-muted font-bold">O(1) READS</span>
          </div>

          <div className="space-y-1.5 max-h-[140px] overflow-y-auto pr-1">
            {Object.keys(db).length === 0 ? (
              <div className="text-center text-[10px] text-text-muted py-6 font-mono">
                KeyDir index empty
              </div>
            ) : (
              Object.entries(db).map(([k, v]) => (
                <div
                  key={k}
                  className="p-2 bg-blue-bg/10 border border-blue-border/20 rounded text-[9px] font-mono flex items-center justify-between"
                >
                  <div>
                    <span className="text-gold font-bold">Key: {k}</span>
                    <span className="text-text-soft ml-2">Offset: {v.offset}B</span>
                    <span className="text-text-muted ml-2">Len: {v.len}B</span>
                  </div>
                  <button
                    onClick={() => handleDelete(k)}
                    className="text-red hover:text-red/80 font-bold hover:underline"
                  >
                    DEL
                  </button>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Database terminal output */}
      <div className="bg-bg border border-border/60 rounded p-2 text-[9px] font-mono text-green h-[45px] overflow-y-auto">
        {terminalMsg}
      </div>
    </div>
  );
}

// ==========================================
// 3. ASYNC RUNTIME SIMULATOR
// ==========================================
function AsyncRuntimeSimulator() {
  const [taskQueue, setTaskQueue] = useState<Array<{ id: number; type: "network" | "cpu"; fd?: number }>>([
    { id: 101, type: "network", fd: 4 },
    { id: 102, type: "cpu" }
  ]);
  const [threads, setThreads] = useState<Array<{ id: number; task: string | null; state: "IDLE" | "WORKING" | "STEALING" }>>([
    { id: 0, task: null, state: "IDLE" },
    { id: 1, task: "Task #99 (HTTP parse)", state: "WORKING" },
    { id: 2, task: null, state: "IDLE" },
    { id: 3, task: null, state: "IDLE" }
  ]);
  const [reactorFds, setReactorFds] = useState<Array<{ fd: number; event: string }>>([
    { fd: 3, event: "TCP_ACCEPT" }
  ]);
  const [log, setLog] = useState<string[]>(["// Async executor & Epoll reactor initialized."]);

  const addLog = (msg: string) => {
    setLog(prev => [...prev.slice(-10), `[${new Date().toTimeString().split(" ")[0]}] ${msg}`]);
  };

  const handleSpawnNetwork = () => {
    const nextFd = reactorFds.length > 0 ? reactorFds[reactorFds.length - 1].fd + 1 : 4;
    const taskId = 200 + Math.floor(Math.random() * 100);

    setReactorFds(prev => [...prev, { fd: nextFd, event: "SOCKET_READ" }]);
    addLog(`REACTOR: epoll_ctl(ADD, fd=${nextFd}, EPOLLIN). Monitoring socket client connection.`);

    // Simulate socket event firing after delay
    setTimeout(() => {
      setReactorFds(prev => prev.filter(x => x.fd !== nextFd));
      setTaskQueue(prev => [...prev, { id: taskId, type: "network", fd: nextFd }]);
      addLog(`REACTOR: epoll_wait event fired on fd=${nextFd}. Dispatching Task #${taskId} to executor queue.`);
    }, 1500);
  };

  const handleSpawnCpu = () => {
    const taskId = 300 + Math.floor(Math.random() * 100);
    setTaskQueue(prev => [...prev, { id: taskId, type: "cpu" }]);
    addLog(`EXECUTOR: Enqueued Task #${taskId} (Heavy CPU hashing) straight to core queue.`);
  };

  const handleTriggerWork = () => {
    if (taskQueue.length === 0) {
      // Trigger a Work-Stealing simulation
      const busyIndex = threads.findIndex(t => t.state === "WORKING");
      const idleIndex = threads.findIndex(t => t.state === "IDLE");
      if (busyIndex !== -1 && idleIndex !== -1) {
        setThreads(prev => prev.map((t, idx) => {
          if (idx === idleIndex) return { ...t, state: "STEALING" };
          return t;
        }));
        addLog(`EXECUTOR: Thread #${idleIndex} queue empty. Executing crossbeam work-stealing from Thread #${busyIndex}...`);
        
        setTimeout(() => {
          setThreads(prev => prev.map((t, idx) => {
            if (idx === idleIndex) return { ...t, state: "WORKING", task: "Stolen task (hashing)" };
            return t;
          }));
          addLog(`EXECUTOR: Thread #${idleIndex} successfully stole and is working on task.`);
        }, 1000);
      }
      return;
    }

    const nextTask = taskQueue[0];
    setTaskQueue(prev => prev.slice(1));

    // Find first idle thread
    const idleIdx = threads.findIndex(t => t.state === "IDLE");
    if (idleIdx !== -1) {
      setThreads(prev => prev.map((t, idx) => {
        if (idx === idleIdx) return { 
          ...t, 
          state: "WORKING", 
          task: `Task #${nextTask.id} (${nextTask.type === "network" ? `TCP read fd=${nextTask.fd}` : "CPU hash"})` 
        };
        return t;
      }));
      addLog(`EXECUTOR: Thread #${idleIdx} popped Task #${nextTask.id} and is executing.`);
      
      // Complete task after delay
      setTimeout(() => {
        setThreads(prev => prev.map(t => {
          if (t.task?.includes(`#${nextTask.id}`)) return { ...t, state: "IDLE", task: null };
          return t;
        }));
        addLog(`EXECUTOR: Thread completed Task #${nextTask.id}. Returned to idle pool.`);
      }, 2000);
    } else {
      addLog("EXECUTOR: All threads currently busy. Task remaining in scheduling queue.");
    }
  };

  return (
    <div className="space-y-4">
      {/* Control Buttons */}
      <div className="flex flex-wrap gap-2">
        <button
          onClick={handleSpawnNetwork}
          className="bg-purple-bg border border-purple-border text-purple text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-purple/10 transition-all"
        >
          + NETWORK_READ_EVENT
        </button>
        <button
          onClick={handleSpawnCpu}
          className="bg-blue-bg border border-blue-border text-blue text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-blue/10 transition-all"
        >
          + HEAVY_CPU_TASK
        </button>
        <button
          onClick={handleTriggerWork}
          className="bg-green-bg border border-green-border text-green text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-green/10 transition-all ml-auto"
        >
          ⚡ TRIGGER_TICK_SCHEDULER
        </button>
      </div>

      <div className="grid md:grid-cols-3 gap-4">
        {/* Reactor Column */}
        <div className="cyber-panel p-3">
          <div className="text-[9px] font-mono font-bold text-text-soft tracking-wider mb-2 border-b border-border/50 pb-1 flex items-center justify-between">
            <span>EPOLL REACTOR (MIO)</span>
            <span className="text-[8px] text-purple font-bold">I/O WAITS</span>
          </div>
          <div className="space-y-1.5 min-h-[120px] max-h-[140px] overflow-y-auto">
            {reactorFds.map((rf, idx) => (
              <div key={idx} className="p-1.5 bg-purple-bg/10 border border-purple-border/30 rounded text-[9px] font-mono flex items-center justify-between">
                <span>Descriptor fd: {rf.fd}</span>
                <span className="text-purple font-bold px-1 rounded bg-purple-bg text-[7px]">{rf.event}</span>
              </div>
            ))}
            {reactorFds.length === 0 && (
              <div className="text-center text-[10px] text-text-muted py-6 font-mono">
                Reactor monitoring 0 fds
              </div>
            )}
          </div>
        </div>

        {/* Task Queue Column */}
        <div className="cyber-panel p-3">
          <div className="text-[9px] font-mono font-bold text-text-soft tracking-wider mb-2 border-b border-border/50 pb-1 flex items-center justify-between">
            <span>EXECUTOR_TASK_QUEUE</span>
            <span className="text-[8px] text-blue font-bold">FIFO</span>
          </div>
          <div className="space-y-1.5 min-h-[120px] max-h-[140px] overflow-y-auto">
            {taskQueue.map((t, idx) => (
              <div key={idx} className="p-1.5 bg-blue-bg/10 border border-blue-border/30 rounded text-[9px] font-mono flex items-center justify-between">
                <span>Task #{t.id}</span>
                <span className="text-blue font-bold px-1 rounded bg-blue-bg text-[7px]">{t.type.toUpperCase()}</span>
              </div>
            ))}
            {taskQueue.length === 0 && (
              <div className="text-center text-[10px] text-text-muted py-6 font-mono">
                Task queue empty
              </div>
            )}
          </div>
        </div>

        {/* Thread Pool Column */}
        <div className="cyber-panel p-3">
          <div className="text-[9px] font-mono font-bold text-text-soft tracking-wider mb-2 border-b border-border/50 pb-1 flex items-center justify-between">
            <span>EXECUTOR WORKER POOL</span>
            <span className="text-[8px] text-green font-bold">CORES</span>
          </div>
          <div className="space-y-1.5 min-h-[120px] max-h-[140px] overflow-y-auto">
            {threads.map((t, idx) => (
              <div
                key={idx}
                className={`p-1.5 rounded border text-[9px] font-mono flex items-center justify-between ${
                  t.state === "WORKING"
                    ? "bg-green-bg/20 border-green-border/30 text-green"
                    : t.state === "STEALING"
                    ? "bg-gold-bg/20 border-gold-border/30 text-gold"
                    : "bg-surface/50 border-border text-text-soft"
                }`}
              >
                <span>Core #{t.id}: {t.state}</span>
                {t.task && (
                  <span className="text-[7px] truncate max-w-[80px] bg-bg/50 px-1 py-0.5 rounded border border-border/30">
                    {t.task}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Telemetry log console */}
      <div className="bg-bg border border-border/60 rounded p-2 text-[9px] font-mono text-green leading-normal h-[75px] overflow-y-auto">
        {log.map((l, idx) => (
          <div key={idx}>{l}</div>
        ))}
      </div>
    </div>
  );
}

// ==========================================
// 4. RAFT DISTRIBUTED CONSENSUS SIMULATOR
// ==========================================
function RaftSimulator() {
  const [nodes, setNodes] = useState([
    { id: 1, role: "Leader", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
    { id: 2, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
    { id: 3, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
    { id: 4, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
    { id: 5, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
  ]);
  const [partition, setPartition] = useState<null | "split">(null);
  const [simLogs, setSimLogs] = useState<string[]>(["// Raft cluster running. Term 1 initialized. Node 1 is Leader."]);
  const [clientKey, setClientKey] = useState("");
  const [clientVal, setClientVal] = useState("");

  const addSimLog = (msg: string) => {
    setSimLogs((prev) => [...prev.slice(-12), `[${new Date().toTimeString().split(" ")[0]}] ${msg}`]);
  };

  const handleClientWrite = (e: React.FormEvent) => {
    e.preventDefault();
    if (!clientKey.trim() || !clientVal.trim()) return;

    const cmdStr = `Set ${clientKey}=${clientVal}`;
    const targetLeader = nodes.find(n => n.role === "Leader" && n.active);

    if (!targetLeader) {
      addSimLog("ERROR: Client write failed. No active leader currently online.");
      return;
    }

    addSimLog(`CLIENT: Submit write ${cmdStr} to Leader Node ${targetLeader.id}.`);

    setNodes(prevNodes => {
      // Create new logs
      const updatedNodes = prevNodes.map(node => {
        if (!node.active) return node;

        // Determine if target node is in the leader's partition network
        let inPartition = true;
        if (partition === "split") {
          // Partition groups: {1,2} and {3,4,5}
          const isLeaderInA = targetLeader.id <= 2;
          const isNodeInA = node.id <= 2;
          inPartition = isLeaderInA === isNodeInA;
        }

        if (inPartition) {
          const nextLogs = [...node.logs, { term: node.term, cmd: cmdStr }];
          return { ...node, logs: nextLogs };
        }
        return node;
      });

      // Calculate if quorum is achieved in the partition
      const activeCountInPartition = updatedNodes.filter(n => {
        if (!n.active) return false;
        if (partition === "split") {
          const isLeaderInA = targetLeader.id <= 2;
          const isNodeInA = n.id <= 2;
          return isLeaderInA === isNodeInA;
        }
        return true;
      }).length;

      // Majority of 5 nodes is 3 nodes
      const quorumMet = activeCountInPartition >= 3;

      if (quorumMet) {
        addSimLog(`RAFT: Quorum met (${activeCountInPartition}/5 nodes online in partition). Committing entry index ${targetLeader.logs.length + 1}.`);
        return updatedNodes.map(node => {
          if (!node.active) return node;
          
          let inPartition = true;
          if (partition === "split") {
            const isLeaderInA = targetLeader.id <= 2;
            const isNodeInA = node.id <= 2;
            inPartition = isLeaderInA === isNodeInA;
          }

          if (inPartition) {
            return { ...node, commitIndex: node.logs.length };
          }
          return node;
        });
      } else {
        addSimLog(`WARNING: Quorum NOT met (${activeCountInPartition}/5 nodes in partition). Log entry written but remains UNCOMMITTED.`);
        return updatedNodes;
      }
    });

    setClientKey("");
    setClientVal("");
  };

  const triggerPartition = () => {
    setPartition("split");
    addSimLog("CHAOS: Simulating Network Partition Split! Group A: [1, 2] | Group B: [3, 4, 5].");
    addSimLog("RAFT: Group A leader Node 1 loses quorum. Group B detects leader loss and elects Node 3 (Term 2).");
    
    setNodes(prev => prev.map(n => {
      if (n.id <= 2) {
        // Group A: Node 1 stays leader but cannot commit
        return { ...n, term: 1 };
      } else {
        // Group B: Node 3 becomes leader in Term 2
        return { 
          ...n, 
          term: 2, 
          role: n.id === 3 ? "Leader" as const : "Follower" as const 
        };
      }
    }));
  };

  const healPartition = () => {
    setPartition(null);
    addSimLog("CHAOS: Network partition healed! Peer pathways reunited.");
    addSimLog("RAFT: Node 1 steps down (sees higher Term 2). Group A nodes rollback uncommitted entries and synchronize logs from Leader Node 3.");

    setNodes(prev => {
      // Leader node 3 has the authoritative log
      const leaderNode = prev.find(n => n.id === 3)!;
      return prev.map(n => {
        return {
          ...n,
          role: n.id === 3 ? "Leader" as const : "Follower" as const,
          term: leaderNode.term,
          commitIndex: leaderNode.commitIndex,
          logs: [...leaderNode.logs] // Pull log entries
        };
      });
    });
  };

  const toggleNode = (id: number) => {
    setNodes(prev => {
      const targetNode = prev.find(n => n.id === id)!;
      const willBeActive = !targetNode.active;

      if (!willBeActive) {
        addSimLog(`CHAOS: Node ${id} crashed (SIGKILL offline).`);
      } else {
        addSimLog(`CHAOS: Node ${id} revived. Syncing log entries with current leader.`);
      }

      let updated = prev.map(n => {
        if (n.id === id) {
          return { ...n, active: willBeActive, role: willBeActive ? "Follower" as const : "Offline" as const };
        }
        return n;
      });

      // If the leader was crashed, followers trigger a new election
      if (!willBeActive && targetNode.role === "Leader") {
        addSimLog("RAFT: Leader lost! Initiating randomized election timeouts on active followers...");
        // Find first active follower to become new leader
        const nextLeader = updated.find(n => n.active && n.id !== id);
        if (nextLeader) {
          const nextTerm = targetNode.term + 1;
          addSimLog(`RAFT: Node ${nextLeader.id} won election for Term ${nextTerm}. Steps up as Leader.`);
          updated = updated.map(n => {
            if (n.id === nextLeader.id) {
              return { ...n, role: "Leader" as const, term: nextTerm };
            }
            if (n.active && n.role !== "Offline") {
              return { ...n, term: nextTerm };
            }
            return n;
          });
        }
      }

      // If node is revived, replicate log from leader
      if (willBeActive) {
        const currentLeader = updated.find(n => n.role === "Leader" && n.active);
        if (currentLeader) {
          updated = updated.map(n => {
            if (n.id === id) {
              return { ...n, logs: [...currentLeader.logs], term: currentLeader.term, commitIndex: currentLeader.commitIndex };
            }
            return n;
          });
        }
      }

      return updated;
    });
  };

  const resetSim = () => {
    setPartition(null);
    setNodes([
      { id: 1, role: "Leader", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
      { id: 2, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
      { id: 3, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
      { id: 4, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
      { id: 5, role: "Follower", term: 1, commitIndex: 1, logs: [{ term: 1, cmd: "init=true" }], active: true },
    ]);
    setSimLogs(["// Simulator reset. Term 1, node 1 leader."]);
  };

  return (
    <div className="space-y-4">
      {/* Forms & Toggles */}
      <div className="flex flex-col md:flex-row gap-4 items-center justify-between">
        <form onSubmit={handleClientWrite} className="flex gap-2 w-full md:max-w-md">
          <input
            type="text"
            value={clientKey}
            onChange={(e) => setClientKey(e.target.value)}
            placeholder="Key (e.g. name)"
            className="flex-1 bg-bg border border-border rounded px-2.5 py-1.5 text-xs font-mono text-text outline-none focus:border-blue/50"
          />
          <input
            type="text"
            value={clientVal}
            onChange={(e) => setClientVal(e.target.value)}
            placeholder="Value (e.g. keagan)"
            className="flex-1 bg-bg border border-border rounded px-2.5 py-1.5 text-xs font-mono text-text outline-none focus:border-blue/50"
          />
          <button
            type="submit"
            className="bg-blue-bg border border-blue-border text-blue text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-blue/10 transition-all shrink-0"
          >
            CLIENT WRITE
          </button>
        </form>

        <div className="flex items-center gap-2">
          {partition ? (
            <button
              onClick={healPartition}
              className="bg-green-bg border border-green-border text-green text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-green/10 transition-all"
            >
              HEAL_PARTITION
            </button>
          ) : (
            <button
              onClick={triggerPartition}
              className="bg-red-bg border border-red-border text-red text-[10px] font-mono font-bold px-3 py-1.5 rounded hover:bg-red/10 transition-all"
            >
              SPLIT_PARTITION
            </button>
          )}

          <button
            onClick={resetSim}
            className="border border-border text-text-soft hover:text-text text-[10px] font-mono px-3 py-1.5 rounded transition-all"
          >
            RESET
          </button>
        </div>
      </div>

      {/* Cluster Node Map Visualizer */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
        {nodes.map(node => {
          const isLeader = node.role === "Leader";
          const isCandidate = node.role === "Candidate";
          const isOffline = !node.active;

          return (
            <div
              key={node.id}
              onClick={() => toggleNode(node.id)}
              className={`cyber-panel p-3 cursor-pointer text-center relative border transition-all ${
                isOffline
                  ? "bg-red-bg/10 border-red-border/30 opacity-60"
                  : isLeader
                  ? "bg-gold-bg/10 border-gold/40"
                  : isCandidate
                  ? "bg-blue-bg/10 border-blue/40"
                  : "bg-surface/50 border-border"
              }`}
            >
              {/* Telemetry pulse */}
              <div className="absolute top-2 right-2 flex items-center gap-1">
                <span className={`w-1.5 h-1.5 rounded-full ${isOffline ? "bg-red" : "bg-green animate-pulse-subtle"}`} />
              </div>

              <div className="text-[12px] font-mono font-bold text-text-soft">
                NODE_0{node.id}
              </div>

              <div className="mt-2 mb-3">
                <span className={`text-[9px] font-mono font-bold px-2 py-0.5 rounded border uppercase ${
                  isOffline ? "bg-neutral-800 border-neutral-700 text-neutral-400" :
                  isLeader ? "bg-gold-bg border-gold text-gold" :
                  isCandidate ? "bg-blue-bg border-blue text-blue" :
                  "bg-surface border-border text-text-soft"
                }`}>
                  {node.role}
                </span>
              </div>

              <div className="text-[9px] font-mono text-text-muted space-y-0.5 text-left bg-bg/50 p-1.5 rounded border border-border/20">
                <div>Term: <span className="text-text font-bold">{node.term}</span></div>
                <div>CommitIdx: <span className="text-text font-bold">{node.commitIndex}</span></div>
              </div>

              {/* Local WAL preview */}
              <div className="mt-3">
                <div className="text-[7px] font-mono text-text-muted text-left uppercase tracking-wider mb-1">WRITE-AHEAD LOG</div>
                <div className="space-y-0.5 max-h-[80px] overflow-y-auto pr-0.5">
                  {node.logs.map((entry, idx) => {
                    const isCommitted = idx + 1 <= node.commitIndex;
                    return (
                      <div key={idx} className={`text-[8px] font-mono px-1 py-0.5 rounded border text-left flex justify-between ${
                        isCommitted ? "bg-green-bg/20 border-green-border/30 text-green" : "bg-gold-bg/20 border-gold-border/30 text-gold"
                      }`}>
                        <span className="truncate max-w-[65px]">{entry.cmd}</span>
                        <span className="text-[6px] opacity-75">{isCommitted ? "C" : "U"}</span>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Network splits overlays indicators */}
      {partition && (
        <div className="flex items-center justify-center gap-6 py-1 bg-red-bg/10 border border-red-border/30 rounded-md text-[9px] font-mono text-red font-bold">
          <span>GROUP A [Node 1, 2]</span>
          <span className="animate-pulse">◀ SPLIT-BRAIN ISOLATION NETWORK BARRIER ▶</span>
          <span>GROUP B [Node 3, 4, 5]</span>
        </div>
      )}

      {/* Telemetry simulator logs console */}
      <div className="bg-bg border border-border/60 rounded p-2 text-[9px] font-mono text-green leading-normal h-[90px] overflow-y-auto">
        {simLogs.map((l, idx) => (
          <div key={idx}>{l}</div>
        ))}
      </div>
    </div>
  );
}
