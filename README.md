# Systems Engineering Portfolio — Infrastructure from First Principles

A deliberately cross-cutting systems engineering portfolio demonstrating infrastructure design at every layer of the stack: Linux kernel syscalls, distributed consensus protocols, lock-free data structures, binary network protocols, and cloud-native orchestration — all built from first principles with zero reliance on platform frameworks.

---

## Core Objective & Structural Constraints

**The problem:** Modern infrastructure is built on layers of abstraction that engineers rarely inspect. Docker wraps containerd wraps runc wraps libcontainer wraps clone(). Kubernetes wraps etcd wraps Raft. Kafka wraps segmented logs. Understanding the kernel primitives underneath the abstractions is what separates a principal engineer from a framework operator.

**This project builds each layer from scratch — no Docker, no runc, no Kafka, no Kubernetes:**

| Constraint | Enforcement |
|---|---|
| **No container runtimes** | Namespace isolation via `clone()` syscall, cgroups v2 via raw filesystem writes |
| **No message brokers** | Custom binary TCP protocol with lock-free ring buffers and segmented append-only logs |
| **No consensus-as-a-service** | Hand-rolled SWIM gossip protocol over UDP + Raft consensus with explicit leader election |
| **No serialization frameworks** | Custom 32-byte binary telemetry protocol with magic-byte validation and DataView zero-copy parsing |
| **No allocation in hot paths** | Pre-allocated ring buffers, const-generic SPSC queues, explicit `Vec::with_capacity` patterns |
| **No privileged containers** | 5 capabilities kept from ~40; ~120 syscalls whitelisted via seccomp-BPF; NO_NEW_PRIVS enforced |
| **Supply chain integrity** | `deny.toml` blocks GPL/AGPL, yanked crates, multiple versions, unknown registries; `cargo-audit` in CI |

---

## System Architecture & Data Flow

```
                        ┌─────────────────────────────────────┐
                        │        ui-control-center              │
                        │    Next.js 15 · React 19 · Canvas     │
                        │   Portfolio + Live Interactive Demos  │
                        └─────────────┬────────────────────────┘
                                      │ Binary Telemetry Protocol (32B/node, 5 nodes × 500ms)
          ┌───────────────────────────┼───────────────────────────┐
          │                           │                           │
          ▼                           ▼                           ▼
┌──────────────────┐    ┌─────────────────────┐    ┌────────────────────────────┐
│ compute-         │    │   log-broker         │    │    platform-nodes           │
│ orchestrator     │    │   ───────────────    │    │    ───────────────          │
│                  │    │   [Producer]─TCP→    │    │    [SWIM Gossip]──UDP──┐   │
│ Actor System     │    │     [Ring Buffer]    │    │    [LSM Storage]       │   │
│ (tokio tasks)    │    │       [Segment.log]  │    │    [Raft Consensus]    │   │
│ SWIM Membership  │    │     [Consumer]←TCP   │    │    [epoll HTTP Proxy]  │   │
│ Task Scheduler   │    │                      │    │                        │   │
│ OpenTelemetry    │    │  Binary TCP Protocol │    │  SWIM over UDP (7946)  │   │
└────────┬─────────┘    └──────────┬───────────┘    └───────────┬────────────┘
         │                         │                            │
         │    Actor Messaging      │    Segmented Log I/O       │    Gossip Dissemination
         │    (bincode over TCP)   │    (CRC32C per message)    │    (Ping/Ack/PingReq)
         │                         │                            │
         ▼                         ▼                            ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                              core-sys                                          │
│    Lock-Free SPSC Queue (const-generic, Acquire/Release + fence ordering)      │
│    Zero-Allocation Telemetry Logger (structured JSON with timestamp nanos)     │
└──────────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────┐    ┌──────────────────────────────┐
│    container-engine           │    │    raft-kv                    │
│    ────────────────           │    │    ──────                     │
│    clone(NEWPID|NEWNS|NEWUTS |    │    Raft Consensus (tokio)      │
│          NEWIPC|NEWNET)        │    │    Key-Value Store            │
│    pivot_root + OverlayFS      │    │    Network Simulation         │
│    cgroups v2 (mem/cpu/io/pid) │    │    Configurable Partitions    │
│    seccomp-BPF (~120 syscalls) │    │                              │
│    veth + cbr0 + iptables      │    │                              │
└──────────────────────────────┘    └──────────────────────────────┘
```

---

## Performance Profiles

*Benchmarks executed via Criterion. Hardware: AMD Ryzen 7 (8C/16T), Linux 6.1. Results are verification-grade — run `cargo bench` to reproduce.*

| Benchmark | Configuration | Result |
|---|---|---|
| **SPSC Push/Pop** | 1024 × u64 elements, single-threaded | Stable throughput; queue capacity: 2048 |
| **SPSC Cross-Thread** | Producer → Consumer, batch sizes 64/256/1024 | Latency measured at queue saturation; spin-loop backoff on contention |
| **SPSC Back-Pressure** | Fill 512 slots, drain completely | Measures bounded behavior under half-capacity load |
| **LSM MemTable Insert** | 64 / 256 / 1024 key-value pairs, sequential | Memory-only path; flush threshold 128MB prevents SSTable spill |
| **LSM MemTable Read** | 64 / 256 / 1024 reads, pre-populated | BTreeMap lookup latency under increasing dataset size |
| **LSM Mixed 80/20** | 800 reads + 200 writes, seeded with 200 keys | Simulates production read-heavy workload profile |

To produce concrete numbers:
```bash
cargo bench --workspace --all-targets --exclude platform-nodes --exclude container-engine
```

---

## Security Posture

### Container Boundary
- **NO_NEW_PRIVS** enforced before any other security operation (irreversible)
- **Capability bounding set**: 5 kept from ~40 (`CAP_CHOWN`, `CAP_DAC_OVERRIDE`, `CAP_FOWNER`, `CAP_FSETID`, `CAP_KILL`). `CAP_SETUID`, `CAP_SETGID`, `CAP_NET_BIND_SERVICE`, `CAP_SYS_ADMIN`, `CAP_NET_RAW` explicitly dropped
- **Seccomp-BPF**: ~120 syscalls whitelisted. Architecture validation (x86_64 check → KILL on mismatch). Unknown syscalls → KILL_PROCESS. Filter installed after NO_NEW_PRIVS for kernel enforcement
- **Seccomp defense-in-depth**: BPF runs at the kernel boundary before syscall execution — cannot be bypassed by statically-linked binaries or raw `int 0x80`/`syscall` instructions

### Supply Chain
- **`deny.toml`**: CVEs denied, GPL/AGPL/LGPL all variants denied, unlicensed packages denied, yanked crates denied, unknown registries denied, multiple versions denied
- **CI pipeline**: `cargo-deny check` runs on every PR; `cargo-audit` scans for RustSec advisories; `trufflehog` scans for accidentally committed secrets
- **Static analysis**: `deny-unwraps.py` bans `.unwrap()`, `.expect()`, `panic!()` in production code

### Web Layer
- **CSP**: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'`
- **Headers**: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`
- **Tauri CSP**: `default-src 'self'; script-src 'self'` — explicit allowlist, not wildcard

### CI Prevention
- **`cargo fmt --check`** — zero formatting deviations
- **`cargo clippy -- -D warnings`** — zero warnings enforced
- **`cargo test --workspace`** — 37 Rust tests, 58 frontend tests
- **`cargo bench --no-run`** — benchmark compilation gate

---

## Project Index

| Crate | What It Is | Why It Matters |
|---|---|---|
| **container-engine** | Linux container runtime from syscalls | Proves kernel-level engineering: namespaces, cgroups, seccomp, OverlayFS |
| **log-broker** | Distributed pub-sub log broker | Proves systems design: segmented logs, lock-free buffers, binary protocols |
| **compute-orchestrator** | Actor-driven compute layer | Proves distributed systems: SWIM gossip, actor model, OpenTelemetry, IaC |
| **platform-nodes** | Cluster consensus daemon | Proves storage + consensus: LSM engine, SWIM gossip, epoll proxy |
| **raft-kv** | Raft consensus KV store | Proves consensus protocols: leader election, log replication, state machines |
| **core-sys** | Lock-free data structures | Proves concurrent programming: SPSC queue, atomic ordering, zero-alloc logging |
| **admin-tools** | Terminal TUI dashboard | Proves systems tooling: zero-dep HTTP, hand-rolled JSON, ANSI rendering |
| **src-tauri** | Desktop application shell | Proves platform engineering: native IPC, system tray, cross-platform builds |
| **ui-control-center** | Portfolio + interactive demos | Proves full-stack: Next.js 15, Canvas API, binary protocol decode, Web Workers |

---

## Quick Start

```bash
# Build all cross-platform crates (excludes Linux-only container-engine + platform-nodes)
cargo build --workspace --all-features --exclude platform-nodes --exclude container-engine

# Run test suites
cargo test --workspace --all-features --exclude platform-nodes --exclude container-engine

# Run benchmarks (compile-only on non-Linux)
cargo bench --workspace --all-targets --no-run

# Build frontend
cd ui-control-center && npm ci && npm run build && npm test
```

---

## DevOps Pipeline

- **CI**: 3-stage GitHub Actions DAG (security-audit → lint-check → build-matrix × 3 OS → safety-and-bench)
- **CD**: Docker multi-stage build (rust:alpine → scratch, ~8MB) → GHCR → Terraform IaC (VPC, EC2, SG)
- **Safety**: `deny-unwraps.py` bans unhandled panics in production; `cargo-audit` blocks known CVEs
- **Secrets**: `trufflehog` scans every push; `GITHUB_TOKEN` auto-rotated; OIDC for cloud auth

---

## Architectural Decision Records

See [`/architecture/`](architecture/) for detailed RFCs documenting every major engineering trade-off:

- [ADR 0001: Seccomp-BPF Syscall Filtering](architecture/0001-seccomp-bpf-filtering.md)
- [ADR 0002: Binary Telemetry Protocol vs JSON](architecture/0002-binary-telemetry-protocol.md)
- [ADR 0003: SPSC Ring Buffer Memory Ordering](architecture/0003-spcs-ring-buffer-ordering.md)
- [ADR 0004: Container Security Operation Ordering](architecture/0004-container-security-ordering.md)
- [ADR 0005: Actor Model Concurrency vs OS Threads](architecture/0005-actor-model-concurrency.md)

---

## License

MIT
