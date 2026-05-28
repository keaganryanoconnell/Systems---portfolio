# Control Center — Systems Engineering Portfolio

A full-stack systems engineering project demonstrating infrastructure design at every level of the stack: Linux kernel primitives, distributed consensus protocols, lock-free data structures, and cloud-native orchestration.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ui-control-center                         │
│           Next.js 15 · React 19 · Tailwind v4               │
│       Portfolio site with live interactive demos            │
└──────────────────────────┬──────────────────────────────────┘
                           │ Tauri IPC / HTTP
    ┌──────────────────────┼──────────────────────┐
    │                      │                      │
    ▼                      ▼                      ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐
│  src-tauri    │  │ admin-tools   │  │ compute-orchestrator   │
│  Tauri 1.5    │  │ TUI Dashboard │  │ Actor-driven compute   │
│  Desktop Host │  │ Zero-dep HTTP │  │ SWIM gossip · Tokio    │
└───────────────┘  └───────────────┘  │ OpenTelemetry · Docker │
                                      │ Terraform · CI/CD      │
┌───────────────────────────────────┐ └───────────────────────┘
│          platform-nodes           │
│  LSM Storage Engine               │
│  SWIM Gossip Protocol             │
│  epoll HTTP Proxy                 │
└──────────────┬────────────────────┘
               │
┌──────────────┴────────────────────┐
│         core-sys                  │
│  Lock-free SPSC Queue             │
│  Zero-allocation Telemetry Logger │
└───────────────────────────────────┘

┌────────────────────┐  ┌──────────────────────┐
│  container-engine  │  │     log-broker        │
│  Linux namespaces  │  │  Segmented append log │
│  cgroups v2        │  │  Lock-free ring buf   │
│  seccomp-BPF       │  │  Binary TCP protocol  │
│  OverlayFS · veth  │  │  Consumer cursors     │
└────────────────────┘  └──────────────────────┘

┌────────────────────┐
│     raft-kv        │
│  Raft Consensus    │
│  KV Store          │
│  Network Sim       │
└────────────────────┘
```

## Crates

| Crate | Language | Description | Key Technologies |
|---|---|---|---|
| `container-engine` | Rust | Production Linux container runtime | Namespaces, cgroups v2, seccomp-BPF, OverlayFS, veth, iptables |
| `log-broker` | Rust | Distributed pub-sub log broker | Segmented logs, lock-free SPSC, binary TCP framing, mio |
| `platform-nodes` | Rust | Cluster consensus daemon | LSM storage, SWIM gossip, epoll HTTP proxy |
| `compute-orchestrator` | Rust | Cloud-native compute layer | Actor model, SWIM gossip, OpenTelemetry, Docker, Terraform |
| `raft-kv` | Rust | Raft consensus KV store | Raft protocol, network simulation |
| `core-sys` | Rust | Core systems library | Lock-free SPSC queue, zero-alloc telemetry logger |
| `admin-tools` | Rust | Terminal dashboard | Zero-dep HTTP client, hand-rolled JSON parser, ANSI TUI |
| `src-tauri` | Rust | Electron-alternative desktop shell | Tauri 1.5, native IPC commands |
| `ui-control-center` | TypeScript | Portfolio + interactive demos | Next.js 15, React 19, Tailwind v4, Canvas API |

## Quick Start

```bash
# Build all cross-platform crates
cargo build --workspace --all-features --exclude platform-nodes --exclude container-engine

# Run tests
cargo test --workspace --all-features --exclude platform-nodes --exclude container-engine

# Build frontend
cd ui-control-center && npm ci && npm run build && npm test
```

## DevOps

- **CI**: 3-stage GitHub Actions pipeline (fmt+clippy → build+test matrix → safety+bench)
- **CD**: Docker multi-stage build → GitHub Container Registry → Terraform IaC deployment
- **Safety**: `deny-unwraps.py` bans `.unwrap()`, `.expect()`, `panic!()` in production code
- **Supply chain**: `cargo-deny` validates licenses (MIT/Apache only), CVEs, and crate provenance

## License

MIT
