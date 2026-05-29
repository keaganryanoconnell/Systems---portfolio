# AGENTS.md — Systems Engineering Portfolio

## Quick-Start for AI Agents

This is a 17-crate Rust monorepo + Next.js 15 frontend. Read this before writing any code.

### Build Commands
```bash
# Rust check (all crates except Linux-only)
cargo check --workspace --all-features --exclude platform-nodes --exclude container-engine

# Rust tests
cargo test --workspace --all-features --exclude platform-nodes --exclude container-engine

# Rust formatting + linting
cargo fmt --all -- --check
cargo clippy --workspace --all-features --all-targets --exclude platform-nodes --exclude container-engine -- -D warnings

# Frontend build
cd ui-control-center && npm install && npm run build

# Frontend tests
cd ui-control-center && npm test

# Static analysis
python scripts/deny-unwraps.py
cargo deny check
cargo audit
```

### Live URLs
- **Portfolio:** https://systems-portfolio-five.vercel.app
- **Capstone console:** https://systems-portfolio-five.vercel.app/capstone
- **GitHub:** https://github.com/keaganryanoconnell/Systems---portfolio

## Architecture (17 Crates)

### Storage & Data Layer (Tier 1)
| Crate | Purpose | Key Files |
|---|---|---|
| `lsm-engine` | LSM storage: MemTable, SSTable, Compaction | `src/engine.rs`, `src/sstable.rs` |
| `sql-engine` | Recursive descent SQL parser + executor | `src/parser/mod.rs`, `src/executor/mod.rs` |
| `columnar-engine` | WASM columnar OLAP, zero-copy ingest, LRU pool | `src/chunk.rs`, `src/pool.rs`, `src/ingest.rs` |
| `lob-engine` | Limit order book: price-time matching engine | `src/book.rs`, `src/pool.rs` |
| `telemetry-aggregator` | Edge telemetry: Gorilla compression, packet ring | `src/compressor.rs`, `src/ring.rs` |
| `container-engine` | Linux container runtime (Linux only) | `src/isolate/mod.rs`, `src/security/` |
| `core-sys` | Lock-free SPSC queue, zero-alloc logger | `src/spsc.rs`, `src/logger.rs` |

### Distributed & Consensus Layer (Tier 2)
| Crate | Purpose | Key Files |
|---|---|---|
| `raft-kv` | Raft consensus: leader election, log replication | `src/raft.rs`, `src/rpc.rs` |
| `log-broker` | Pub-sub log: segmented logs, lock-free SPSC | `src/buffer.rs`, `src/network/protocol.rs` |
| `compute-orchestrator` | Actor-driven compute: SWIM gossip, OpenTelemetry | `src/actor/system.rs`, `src/gossip/mod.rs` |
| `api-gateway` | HTTP/TLS entry point: axum, 8 REST routes | `src/router.rs`, `src/handlers/` |
| `common-protocol` | Unified IPC: 30B frames, 20 message types | `src/frame.rs`, `src/message.rs` |
| `platform-nodes` | Cluster daemon: SWIM gossip, epoll proxy (Linux only) | `src/consensus/swim.rs` |
| `sensor-fusion-buffer` | MPMC CAS ring buffer: LiDAR/Camera/IMU fusion | `src/buffer.rs`, `src/affinity.rs` |
| `src-tauri` | Tauri desktop shell | `src/main.rs` |
| `admin-tools` | Terminal TUI dashboard | `src/tui.rs` |

### Sync & Consensus Layer (Tier 3 — Capstone)
| Crate | Purpose | Key Files |
|---|---|---|
| `crdt-engine` | LWW-Element-Set: delta sync, peer merge | `src/lww.rs`, `src/sync.rs` |

## Frontend Architecture (ui-control-center/)

### Key Files
| File | Purpose |
|---|---|
| `src/app/page.tsx` | Main scrollable portfolio |
| `src/app/capstone/page.tsx` | FAANG-grade capstone console |
| `src/app/components/ProjectWorkspace.tsx` | All 17 project widgets (1482 lines) |
| `src/app/components/CapstonePanels.tsx` | Capstone 3-column layout |
| `src/app/components/NavBar.tsx` | Sticky nav with scroll-spy |
| `src/app/components/DeepDives.tsx` | 5 technical deep dives with ASCII diagrams |
| `src/app/globals.css` | Tailwind v4 theme tokens |
| `src/shaders/spatial_transform.wgsl` | WGSL compute shader (NOT wired yet) |
| `src/workers/engine_pool.ts` | Worker pool with SharedArrayBuffer (NOT integrated yet) |
| `src/workers/engine_worker.ts` | Worker Atomics.wait() loop (NOT integrated) |

### How to Add a Portfolio Widget
1. Add project entry to `projects` array in `ProjectWorkspace.tsx`
2. Add conditional renderer: `{activeProjectId === "id" && (<NewSimulator />)}`
3. Add simulator function at end of file

## Conventions

### Rust
- **Edition:** 2021 for all crates
- **License:** MIT (required — deny.toml blocks GPL/AGPL)
- **No unwrap/expect/panic in src/** (enforced by `scripts/deny-unwraps.py`)
- **Zero clippy warnings** (enforced by CI: `-D warnings`)
- **Zero dependencies** where possible — core crates (lob-engine, core-sys) have zero deps
- **New crate pattern:** `Cargo.toml` with `edition = "2021"`, `license = "MIT"`, `description`
- **Platform gating:** Use `[target.'cfg(target_os = "linux")'.dependencies]` for Linux-only

### TypeScript/React
- **Next.js 15** with `output: 'export'` (static export)
- **All components use `"use client"` directive**
- **Tailwind v4** with custom `@theme` tokens in `globals.css`
- **lucide-react** for icons, **framer-motion** for animations
- **Simulated data pattern:** `useState` + `setInterval` for live metrics

### NEVER
- Add GPL/AGPL/LGPL licensed crates (blocked by `deny.toml`)
- Add unwrap/expect/panic in production Rust code
- Build platform-nodes or container-engine on Windows/macOS
- Delete or rename existing crates without updating workspace `members`
