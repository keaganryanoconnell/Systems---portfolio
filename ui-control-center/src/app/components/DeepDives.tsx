"use client";

import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

interface DiveSection {
  heading: string;
  lines: string[];
}

interface Dive {
  title: string;
  summary: string;
  diagram?: string;
  sections: DiveSection[];
}

const DIVES: Dive[] = [
  {
    title: "Building a Container Runtime from Scratch (Without Docker)",
    summary: "How I implemented namespace isolation, cgroups v2 resource control, OverlayFS, and seccomp-BPF in pure Rust — no Docker, no runc, no libcontainer.",
    diagram: `┌─────────────────────────────────────────────────────────────────────────┐
│                            HOST KERNEL                                    │
│  ┌───────────────────────────────────────────────────────────────────────┐│
│  │                 clone(CLONE_NEWPID|NEWNS|NEWUTS|NEWIPC|NEWNET)        ││
│  │                              │                                         ││
│  │  ┌───────────────────────────▼─────────────────────────────────────┐  ││
│  │  │                   CONTAINER NAMESPACE BOUNDARY                   │  ││
│  │  │                                                                  │  ││
│  │  │   PID 1 (init process)           Child Process                    │  ││
│  │  │   ┌─────────────────┐           ┌─────────────────┐              │  ││
│  │  │   │ • Reaper loop   │           │ • exec(command) │              │  ││
│  │  │   │ • Signal fwd    │──fork──▶  │ • User workload │              │  ││
│  │  │   │ • Zombie reap   │           │ • mount namespace│             │  ││
│  │  │   └─────────────────┘           └─────────────────┘              │  ││
│  │  │                                                                  │  ││
│  │  │   SECURITY:  NO_NEW_PRIVS → drop 35+ caps → seccomp-BPF (~120)   │  ││
│  │  │   FS:        OverlayFS (lowerdir RO + upperdir RW) + pivot_root   │  ││
│  │  │   NET:       veth pair → cbr0 bridge (10.88.0.0/16) → IPTables   │  ││
│  │  │   CGROUPS:   /sys/fs/cgroup/container-<id>/ (mem, cpu, io, pid)   │  ││
│  │  │   MOUNTS:    /proc /sys /dev /dev/pts /dev/mqueue /run /tmp       │  ││
│  │  └──────────────────────────────────────────────────────────────────┘  ││
│  └───────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────┘`,
    sections: [
      {
        heading: "Namespace Isolation",
        lines: [
          "clone() syscall with 5 namespace flags creates the container boundary — a new PID namespace, mount namespace, UTS namespace (hostname), IPC namespace, and optionally a network namespace",
          "2MB pre-allocated child stack prevents stack overflow within the cloned process",
          "MS_PRIVATE mount propagation ensures mounts inside the container do not leak to the host",
          "sethostname() called inside CLONE_NEWUTS to set the container's hostname independently",
        ],
      },
      {
        heading: "Security Ordering (Irreversible Chain)",
        lines: [
          "PR_SET_NO_NEW_PRIVS — prevents setuid binaries and capability gains. Once set, cannot be unset by any process including root. Persists across execve().",
          "Capability bounding set drop — reduces from ~40 capabilities to exactly 5 (CAP_CHOWN, CAP_DAC_OVERRIDE, CAP_FOWNER, CAP_FSETID, CAP_KILL). CAP_SETUID, CAP_SETGID, CAP_SYS_ADMIN explicitly removed.",
          "Seccomp-BPF filter installation — ~120 syscalls in explicit allowlist. Architecture validated (x86_64). Unknown syscall → KILL_PROCESS. Filter cannot be removed once installed.",
          "Each step constrains the attack surface for the next. An attacker must bypass three independent boundaries to execute arbitrary syscalls.",
        ],
      },
      {
        heading: "Filesystem Setup",
        lines: [
          "OverlayFS with read-only lowerdir (the container image) and writable upperdir (container-specific changes). Combined via a single merged mount point.",
          "pivot_root() swaps the container's root filesystem to the OverlayFS merge directory. Old root is unmounted with MNT_DETACH.",
          "11 kernel paths masked (e.g., /proc/sysrq-trigger, /proc/kcore) and 4 set to readonly (e.g., /sys/firmware) to prevent information leaks and system manipulation.",
          "Virtual filesystems mounted: /proc, /sys, /dev, /dev/pts (PTY), /dev/mqueue (POSIX MQ), /run (tmpfs), /tmp (tmpfs).",
        ],
      },
      {
        heading: "Networking",
        lines: [
          "veth pair: one end in the container namespace (eth0), the other in the host namespace. Traffic flows through the virtual Ethernet tunnel.",
          "cbr0 bridge connects all container veth endpoints on the host side. 10.88.0.0/16 subnet with per-container IP allocation from a persistent pool lease file.",
          "iptables MASQUERADE enables outbound NAT so container traffic appears to originate from the host IP. DNAT rules map host ports to container ports.",
          "/etc/resolv.conf, /etc/hosts, and /etc/hostname are generated inside the container for DNS resolution and host identification.",
        ],
      },
      {
        heading: "Process Lifecycle",
        lines: [
          "Two-stage fork: the first fork creates PID 1 (init) inside the new namespace tree. The init process runs a reaper loop, forwards signals to child, and reaps zombie processes.",
          "The second fork (from init) execs the user's command. If the user command exits, init reaps it and exits itself, cleaning up the container.",
          "CLI provides 10 subcommands: run, exec, kill, ps, stats, inspect, logs, pause, resume, rm. Container lifecycle follows a validated state machine (Created → Running → Paused → Stopped/Dead).",
        ],
      },
    ],
  },
  {
    title: "The Binary Protocol: Why 32 Bytes Instead of JSON",
    summary: "Design decisions behind the zero-copy telemetry protocol — magic bytes, DataView parsing, and why JSON was the wrong choice for real-time metrics.",
    diagram: `Byte Layout Visualization (32 bytes per node entry):

┌────────┬──────┬──────┬──────┬──────┬────────────┬────────────┬───────────┐
│  0-3   │  4   │  5   │  6   │  7   │    8-11    │   12-15    │   16-17   │
│ MAGIC  │ NODE │ ROLE │STATUS│ CPU% │ MEM_ALLOC  │ MEM_TOTAL  │  FD_POOL  │
│0xAABBC │  ID  │      │      │      │  (uint32)  │  (uint32)  │ (uint16)  │
│  CDD   │(u8)  │ (u8) │ (u8) │ (u8) │            │            │           │
├────────┼──────┴──────┴──────┴──────┴────────────┴────────────┴───────────┤
│  18-19 │          20-27                  │         28-31                  │
│REPLAG  │      LSM STORAGE BYTES         │           IOPS                  │
│ (u16)  │     (uint64 big-endian)        │         (uint32)               │
└────────┴────────────────────────────────┴────────────────────────────────┘

5 nodes × 32 bytes = 160 bytes. JSON equivalent: ~800 bytes. 80% bandwidth savings.
DataView.getUint8(offset) — zero-copy read from raw ArrayBuffer. No GC pressure.`,
    sections: [
      {
        heading: "Byte Layout & Encoding",
        lines: [
          "Magic bytes (0xAABBCCDD) at offset 0 provide immediate misalignment detection — corrupted or partial buffers are caught in O(1) before any data is read",
          "Node ID, role (Leader/Follower/Candidate), and status (Healthy/Degraded/Offline) packed into single uint8 fields with 3-value enums",
          "Memory and IOPS use uint32 (4 bytes each) — enough for up to 4GB memory and 4 billion IOPS",
          "LSM storage bytes use uint64 (8 bytes) big-endian for compatibility across architectures. JavaScript's DataView handles byte order transparently",
        ],
      },
      {
        heading: "Binary vs JSON: Real Numbers",
        lines: [
          "JSON: 5 nodes at ~160 bytes each = ~800 bytes per poll. Field names like 'arenaMemoryAllocated' add ~50% overhead",
          "Binary: exactly 32 bytes per node × 5 nodes = 160 bytes total. 80% reduction in bandwidth",
          "JSON.parse() creates JavaScript objects with string-keyed properties — heap allocation + GC churn at 2Hz polling",
          "DataView reads integers directly from the underlying ArrayBuffer — the same mechanism used by WebGL and WebAssembly for binary data transfer. No allocation, no parsing",
        ],
      },
      {
        heading: "DataView Zero-Copy Parsing",
        lines: [
          "DataView.getUint8(offset) reads a single byte at the specified offset — no intermediate string or parsing step",
          "DataView.getUint32(offset) reads 4 bytes as a 32-bit unsigned integer in big-endian order",
          "DataView.getBigUint64(offset) reads 8 bytes as a 64-bit unsigned integer — JavaScript BigInt support with a u32 fallback for older runtimes",
          "The encode/decode roundtrip is verified in 13 Vitest unit tests covering exact fidelity for all fields including BigInt handling",
        ],
      },
      {
        heading: "Protocol Verification & Safety",
        lines: [
          "encodeNodeTelemetry → decodeNodeTelemetry forms a perfect roundtrip, tested for all field types and edge values (0, MAX_INT, empty buffers)",
          "Magic byte validation at the start of each 32-byte block catches bit-flip errors, partial writes, and protocol version mismatches",
          "CPU values clamped to 0-100 range on encode — a clean defensive measure against corrupt data from the backend",
          "The binary protocol was a deliberate architectural choice: it demonstrates understanding of wire formats, memory layout, and the trade-off between human readability and machine efficiency",
        ],
      },
    ],
  },
  {
    title: "Lock-Free Ring Buffers: Atomics, Fences, and Cache Lines",
    summary: "How the SPSC queue in core-sys achieves zero-allocation message passing using atomic operations, compiler fences, and careful memory ordering.",
    diagram: `SPSC Ring Buffer — Memory Ordering Protocol

  Producer Thread                          Consumer Thread
┌────────────────────┐                  ┌────────────────────┐
│ 1. head.load(Relax)│                  │ 1. tail.load(Relax)│
│ 2. tail.load(Acq)  │                  │ 2. head.load(Acq)  │
│        │           │                  │        │           │
│        ▼           │                  │        ▼           │
│ ┌──────────────┐   │                  │ ┌──────────────┐   │
│ │ WRITE DATA   │   │                  │ │ READ DATA    │   │
│ │ to buffer[]  │   │                  │ │ from buffer[]│   │
│ │ [head:head+n]│   │                  │ │ [tail:tail+n]│   │
│ └──────┬───────┘   │                  │ └──────┬───────┘   │
│        │           │                  │        │           │
│        ▼           │                  │        ▼           │
│ 3. fence(SeqCst)   │  ◄── shared ──▶ │ 3. fence(SeqCst)   │
│        │           │                  │        │           │
│        ▼           │                  │        ▼           │
│ 4. head.store(Rel) │                  │ 4. tail.store(Rel) │
└────────────────────┘                  └────────────────────┘
   head: AtomicUsize                      tail: AtomicUsize
   └───────────────── shared ────────────────────┘

head indicates next write slot. tail indicates next read slot.
Available = head - tail (bounded by capacity).`,
    sections: [
      {
        heading: "Buffer Architecture",
        lines: [
          "Pre-allocated Vec<u8> with power-of-2 capacity (1MB default). No allocations during push/pop — the buffer is sized once and reused",
          "AtomicUsize head (producer writes, consumer reads) and tail (consumer writes, producer reads) provide lock-free coordination between threads",
          "UnsafeCell<Vec<u8>> signals interior mutability to the compiler — the buffer is mutated through shared references, which Rust normally forbids",
          "SPSC contract: exactly one producer and one consumer. Concurrent access is safe because head and tail are on separate cache lines, avoiding false sharing",
        ],
      },
      {
        heading: "Memory Ordering Protocol (Per-Operation Rationale)",
        lines: [
          "head.load(Relaxed) on producer — no ordering required, we just need the current value",
          "tail.load(Acquire) on producer — must see all consumer writes before we write, preventing overwrite of unread data",
          "head.store(Release) on producer — ensures buffer writes are globally visible before the consumer sees the updated head index",
          "tail.store(Release) on consumer — ensures buffer reads are complete before the producer sees the updated tail (preventing overwrite of in-flight reads)",
          "fence(SeqCst) on both sides — hardware memory barrier (DMB SY on ARM, MFENCE on x86_64) prevents CPU-level reordering of buffer access relative to index access",
        ],
      },
      {
        heading: "x86_64 vs ARM: Why fence Matters",
        lines: [
          "x86_64 (TSO — Total Store Order): Acquire/Release ops are free (they map to MOV instructions). fence(SeqCst) emits MFENCE (~33 cycles), but executes once per batch, not per element",
          "ARM (weakly-ordered): Acquire/Release ops still need explicit barriers. fence(SeqCst) emits DMB SY (~20-30 cycles). Without it, writes to buffer[] may be reordered before the index update",
          "compiler_fence (the bug): initially used compiler_fence instead of fence. This only prevents compiler reordering, not CPU reordering. On ARM, a reader could observe partially-written data — a latent data race. Fixed during May 2026 security audit.",
        ],
      },
      {
        heading: "Cache Line & Performance Characteristics",
        lines: [
          "head and tail are separate AtomicUsize fields — they live on different cache lines, preventing false sharing between producer and consumer cores",
          "Buffer data is a contiguous Vec<u8> — all data shares cache lines, but the SPSC protocol ensures producer and consumer never access the same index simultaneously",
          "Criterion benchmarks: ~50ns per push/pop pair on x86_64. Throughput limited by memory bandwidth (~50-100 GB/s), not synchronization overhead",
          "Back-pressure handling: try_write returns BufferFull when head-tail >= capacity. Consumer must drain before producer can continue. No spinning, no blocking.",
        ],
      },
    ],
  },
  {
    title: "Seccomp-BPF: Filtering Syscalls at the Kernel Level",
    summary: "How the container engine uses Berkeley Packet Filter programs to restrict which Linux syscalls a container process can invoke — before they reach the kernel.",
    diagram: `Seccomp-BPF Filter Evaluation Flow

User Process calls syscall (e.g., read(fd, buf, len))
                         │
                         ▼
              ┌──────────────────────────────┐
              │   LINUX KERNEL: seccomp-BPF  │
              │                              │
              │  ┌────────────────────────┐  │
              │  │ Step 1: LD arch        │  │  Load seccomp_data.arch
              │  │   JEQ AUDIT_ARCH_X86_64│──┼──▶ If not x86_64 → SECCOMP_RET_KILL_PROCESS
              │  │                        │  │
              │  │ Step 2: LD syscall_nr  │  │  Load seccomp_data.nr (the syscall number)
              │  │                        │  │
              │  │ Step 3: Linear scan    │  │  For each allowed syscall in the whitelist:
              │  │   JEQ allowed_nr ──────┼──▶ If A == allowed_nr → SECCOMP_RET_ALLOW
              │  │   ...continue scan...  │  │
              │  │                        │  │
              │  │ Step 4: No match       │──┼──▶ SECCOMP_RET_KILL_PROCESS (SIGSYS signal)
              │  └────────────────────────┘  │
              └──────────────────────────────┘
                         │
              ┌──────────▼──────────────┐
              │     ALLOWED: proceed    │
              │     to actual syscall   │
              │     (kernel executes    │
              │      read/write/etc.)   │
              └─────────────────────────┘

Filter is a SockFprog { len, filter: *const SockFilter } installed via:
prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog)
Once installed, the filter CANNOT be removed or relaxed. Irreversible.`,
    sections: [
      {
        heading: "BPF Filter Construction",
        lines: [
          "The filter is built as a Vec<SockFilter> — each entry is a BPF instruction (opcode + jump target + comparison value). 4 instruction types used: ld (load), jeq (jump if equal), ret (return), and sequential fallthrough",
          "Architecture validation is the first check — seccomp_data.arch must match AUDIT_ARCH_X86_64. A 32-bit process on a 64-bit kernel would use a different audit_arch value and be immediately killed",
          "The syscall number is loaded and linear-scanned against the ~120 allowed syscalls. In production, this would be binary search over a sorted list, but the linear scan is correct, auditable, and simple",
          "Any syscall not in the allowlist reaches the final RET KILL_PROCESS instruction — the process receives SIGSYS and terminates immediately",
        ],
      },
      {
        heading: "Allowed Syscall Categories (~120 total)",
        lines: [
          "I/O (~15): read, write, open, close, stat, poll, lseek, pread64, pwrite64, readv, writev",
          "Memory (~8): mmap, mprotect, munmap, brk, mremap, msync, mincore, madvise",
          "Network (~15): socket, connect, accept, sendto, recvfrom, bind, listen, getsockname, getpeername, setsockopt, getsockopt",
          "Process (~12): clone, fork, vfork, execve, exit, wait4, kill, getpid, getppid, setsid, setpgid",
          "File System (~20): mkdir, rmdir, rename, link, unlink, symlink, readlink, chmod, fchmod, truncate, ftruncate, getdents, getcwd, chdir, access, pipe",
          "Futex/Epoll/Signals (~10): futex, epoll_create, epoll_ctl, epoll_wait, rt_sigaction, rt_sigprocmask",
        ],
      },
      {
        heading: "Explicitly Blocked Syscalls",
        lines: [
          "mount, umount2, pivot_root — blocked after initial container setup to prevent mount namespace escape",
          "ptrace — blocked to prevent one container process from inspecting or manipulating another",
          "kexec_load, reboot — blocked to prevent container processes from rebooting the physical host",
          "init_module, finit_module — blocked to prevent kernel module loading from within a container",
        ],
      },
      {
        heading: "Why BPF Instead of LD_PRELOAD Interposition",
        lines: [
          "LD_PRELOAD intercepts libc function calls in userspace. A statically-linked binary bypasses it entirely. A raw int 0x80 or syscall instruction also bypasses it",
          "BPF runs in kernel context before the syscall executes. It intercepts the syscall instruction itself, not the libc wrapper. There is no userspace bypass",
          "The filter is installed via prctl() and is per-process, not per-thread. Once installed, it applies to all future syscalls from this process and any children",
          "This is the same mechanism used by Docker, systemd, Chrome's sandbox, and Flatpak. It's the industry standard for syscall filtering on Linux",
        ],
      },
    ],
  },
  {
    title: "Integrating 12 Crates into a Unified Distributed Platform",
    summary: "How the entire workspace was unified from isolated crates into a single distributed SQL database and compute platform with a common IPC protocol, API gateway, and Docker Compose deployment.",
    diagram: `Unified Distributed Platform — Runtime Data Flow

                       ┌──────────────────────┐
                       │    CLIENT REQUEST     │
                       │   (HTTPS / TLS 1.3)   │
                       └──────────┬───────────┘
                                  │
                       ┌──────────▼───────────┐
                       │     API GATEWAY       │
                       │  axum · tokio · TLS   │
                       │  8 REST endpoints     │
                       └──┬───────┬───────┬───┘
                          │       │       │
              ┌───────────▼─┐ ┌──▼────┐ ┌▼──────────┐
              │ SQL ENGINE   │ │COMPUTE│ │ CLUSTER    │
              │ Parser       │ │ORCHES-│ │ HEALTH     │
              │ Planner      │ │TRATOR │ │ /v1/nodes  │
              │ Executor     │ │Actor  │ │ /v1/health │
              └──────┬───────┘ │Model  │ └───────────┘
                     │         └──┬────┘
         ┌───────────▼────┐  ┌───▼──────────┐
  WRITES │  RAFT KV       │  │  CONTAINER    │
  ──────▶│  Consensus     │  │  ENGINE       │
         │  3-node cluster│  │  Sandbox      │
         └───────┬────────┘  └───────────────┘
                 │
    ┌────────────▼───────────┐    ┌──────────────┐
    │    LSM ENGINE          │    │  LOG BROKER  │
    │  MemTable → SSTable    │    │  Audit Trail │
    │  Compaction Pipeline   │    │  Pub-Sub Log │
    └────────────────────────┘    └──────────────┘

┌───────────────────────────────────────────────────────────┐
│               COMMON PROTOCOL LAYER                        │
│  30-byte frames · 20 message types · bincode payloads      │
│  trace_id propagation · 16MB max frame · magic validation  │
│  All 12 crates depend on this layer for interop             │
└───────────────────────────────────────────────────────────┘`,
    sections: [
      {
        heading: "Before Integration (Isolated Crates)",
        lines: [
          "8 crates with only 1 shared dependency (core-sys). The remaining 7 had zero cross-crate imports and zero runtime communication paths.",
          "5 different incompatible message formats: log-broker used custom Frame protocol, compute-orchestrator used bincode MessageEnvelope, raft-kv used serde RPC enums, platform-nodes used hand-written JSON, container-engine had no network API.",
          "The only runtime communication was admin-tools polling platform-nodes via raw HTTP on a single /telemetry endpoint — a hand-formatted JSON response with no versioning or schema.",
          "Each crate defined its own telemetry types, storage types, and network types independently. No shared schema. No versioned API contracts.",
        ],
      },
      {
        heading: "After Integration (Unified Platform)",
        lines: [
          "12 crates, ALL depending on common-protocol. Every crate gets a shared binary frame format, message type enum, and typed payload structs.",
          "One binary frame protocol: [4B magic=0xCAFEBEEF][4B len][2B ver][4B type][16B trace_id][bincode payload]. Validated by FrameDecoder with overflow protection and magic byte detection.",
          "20 MessageType variants covering every cross-crate operation: SqlQuery(10), RaftAppend(20), StoragePut(30), ComputeTask(40), BrokerProduce(50), ContainerRun(60), HealthCheck(70), TelemetryQuery(80).",
          "API Gateway exposes 8 REST endpoints (TLS 1.3) routing to sql-engine, compute-orchestrator, and cluster health. trace_id propagated via x-trace-id HTTP header for distributed tracing.",
        ],
      },
      {
        heading: "Runtime Data Flow (End-to-End Trace)",
        lines: [
          "1. Client → TLS 1.3 → API Gateway (axum on port 443). Gateway extracts or generates trace_id, injects into x-trace-id response header.",
          "2. API Gateway → common-protocol → SQL Engine. SQL text parsed, planned, executed. Writes routed to Raft KV for consensus replication.",
          "3. SQL Engine writes → Raft KV cluster (3 nodes). AppendEntries replicated with leader election. Quorum confirms commit before returning to client.",
          "4. SQL Engine reads → LSM Engine (MemTable lookup). In-memory BTreeMap for recent writes. SSTable files on disk for older data, with binary search index.",
          "5. API Gateway → Compute Orchestrator (MacroTask dispatch). Scheduler splits into MicroTasks. Actor mailboxes receive work. Results collected via ProcessId addressing.",
          "6. Compute Orchestrator → Container Engine (sandboxed execution). Actor system spawns a container with namespace isolation and memory limits for each untrusted task.",
          "7. Raft KV → Log Broker (transaction audit trail). Each committed AppendEntries produces to the log broker's segmented append-only log files with CRC32C checks.",
        ],
      },
      {
        heading: "Deployment & Operations",
        lines: [
          "Single command deployment: docker compose up -d starts all 10 services (api-gateway, sql-engine, 3×raft-kv, lsm-engine, log-broker, compute-orchestrator, container-engine, admin-tools)",
          "Health checks on every service: api-gateway exposes /health (liveness) and /ready (readiness). Docker HEALTHCHECK runs every 30s on compute-orchestrator using the status subcommand",
          "GitHub Actions CI pipeline auto-runs cargo-audit, trufflehog secret scanning, cargo fmt --check, cargo clippy -D warnings, and cargo test across all crates on every push",
          "Terraform IaC provisions the full cluster: VPC, subnet, security groups (SSH restricted, actor ports VPC-only), EC2 instances with encrypted EBS, auto-scaling group",
        ],
      },
    ],
  },
];

export default function DeepDives() {
  const [open, setOpen] = useState<number | null>(null);

  return (
    <section id="deepdives" className="section">
      <div className="section-heading">Technical Deep Dives</div>
      <h2 className="section-title">How It Works Under the Hood</h2>
      <p className="text-text-soft text-base max-w-2xl mb-8">
        Detailed explanations of the engineering decisions, architecture
        patterns, and kernel-level techniques used across these projects.
        Each dive includes technical diagrams and structured breakdowns.
      </p>

      <div className="space-y-3">
        {DIVES.map((dive, i) => (
          <div key={i} className="cyber-panel overflow-hidden">
            <button
              onClick={() => setOpen(open === i ? null : i)}
              className="w-full flex items-center justify-between p-5 text-left hover:bg-surface/50 transition-colors"
            >
              <div>
                <h4 className="text-sm font-bold text-text mb-1">{dive.title}</h4>
                <p className="text-xs text-text-soft">{dive.summary}</p>
              </div>
              <motion.div
                animate={{ rotate: open === i ? 180 : 0 }}
                transition={{ duration: 0.2 }}
              >
                <ChevronDown size={16} className="text-text-muted" />
              </motion.div>
            </button>
            <AnimatePresence>
              {open === i && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: "auto", opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.3, ease: "easeInOut" }}
                  className="overflow-hidden"
                >
                  <div className="px-5 pb-5 border-t border-border">
                    {/* ASCII Architecture Diagram */}
                    {dive.diagram && (
                      <div className="mt-4 mb-5 border border-border rounded-lg bg-bg/50 overflow-x-auto">
                        <div className="text-[8px] font-mono text-gold font-bold uppercase tracking-wider px-3 pt-3 pb-1 border-b border-border/50">
                          ARCHITECTURE DIAGRAM
                        </div>
                        <pre className="text-[10px] font-mono text-text-soft leading-tight whitespace-pre p-3">
                          {dive.diagram}
                        </pre>
                      </div>
                    )}

                    {/* Structured sections */}
                    <div className="space-y-5">
                      {dive.sections.map((s, si) => (
                        <div key={si}>
                          <h5 className="text-[11px] font-mono font-bold text-gold mb-2.5 tracking-wide uppercase">
                            {s.heading}
                          </h5>
                          <ul className="space-y-2">
                            {s.lines.map((l, li) => (
                              <li key={li} className="flex items-start gap-2 text-[13px] text-text-soft leading-relaxed">
                                <span className="text-gold mt-[3px] shrink-0">▸</span>
                                <span>{l}</span>
                              </li>
                            ))}
                          </ul>
                        </div>
                      ))}
                    </div>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        ))}
      </div>
    </section>
  );
}
