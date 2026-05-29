# RFC-002: Network Topology & QUIC (WebTransport) Protocol Selection

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Date** | 2026-05-29 |
| **Author** | Keagan Ryan O'Connell |
| **Affects** | `api-gateway`, `common-protocol`, future `webtransport-server` crate |

---

## 1. Context & Problem Statement

The collaborative spatial analytics engine requires real-time, bi-directional state synchronization between multiple browser clients and a central coordination server. Users must be able to:

1. **View the same dataset simultaneously** — query results, viewport transforms, and filter states must be shared in real-time across peers
2. **Execute queries without blocking other users** — each client's filter scan runs independently in Wasm/Web Workers, but the filter parameters must be broadcast to other clients
3. **Recover from network interruptions** — a client that disconnects for 30 seconds and reconnects must converge to the same state as peers without a full reload

The traditional approach — WebSockets over HTTP/1.1 — has a fatal flaw: **head-of-line blocking**. Under packet loss, a single dropped TCP segment stalls all subsequent messages in the same stream, including urgent state updates. For a real-time collaborative engine processing sub-10ms state deltas, this is unacceptable.

## 2. Design Decision: WebTransport (HTTP/3 QUIC)

### 2.1 Protocol Comparison

| Property | WebSocket (HTTP/1.1) | WebTransport (HTTP/3 QUIC) |
|---|---|---|
| **Transport** | Single TCP stream | Multiple independent QUIC streams |
| **Head-of-line blocking** | Yes — one lost packet stalls all messages in the stream | No — each stream is independent; datagrams have no ordering guarantees |
| **Connection migration** | No — TCP connection drops on network change (e.g., WiFi to cellular) | Yes — QUIC connection ID survives IP address changes |
| **0-RTT handshake** | No — TCP + TLS requires 2-3 round trips | Yes — QUIC 0-RTT resumes sessions in 0ms (pre-shared key from prior connection) |
| **Datagram support** | No — all data is ordered and reliable | Yes — unreliable datagrams for real-time sensor streams, reliable streams for state messages |
| **Binary framing** | Text or binary frames, single stream | Native binary framing with stream-level flow control |

### 2.2 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        BROWSER CLIENT                            │
│                                                                  │
│  Main Thread                    Web Worker Pool                  │
│  ┌──────────────┐              ┌──────────────────────────────┐  │
│  │ React/Leptos │              │ W1: Wasm Engine + CRDT       │  │
│  │ UI Shell     │              │ W2: Wasm Engine + CRDT       │  │
│  └──────┬───────┘              │ W3: Wasm Engine + CRDT       │  │
│         │                      │ W4: Wasm Engine + CRDT       │  │
│         │                      └──────────────┬───────────────┘  │
│         │                                     │                   │
│  ┌──────▼─────────────────────────────────────▼───────────────┐  │
│  │              SharedArrayBuffer (128MB)                      │  │
│  │  ┌─────────────────┐  ┌────────────────────────────────┐   │  │
│  │  │ Control Ring    │  │ Data Buffer                     │   │  │
│  │  │ (Atomics)       │  │ (Raw binary chunks)             │   │  │
│  │  └─────────────────┘  └────────────────────────────────┘   │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │              WebTransport Client (browser API)                ││
│  │  ┌──────────────────┐  ┌──────────────────────────────────┐  ││
│  │  │ Unidirectional   │  │ Bidirectional Stream              │  ││
│  │  │ Streams (×4)     │  │ (State Sync + CRDT Deltas)       │  ││
│  │  │ Data ingestion   │  │ Reliable, ordered, low-latency   │  ││
│  │  │ feeds (out-of-   │  │                                   │  ││
│  │  │ order, datagram) │  │                                   │  ││
│  │  └──────────────────┘  └──────────────────────────────────┘  ││
│  └──────────────────────────────────────────────────────────────┘│
└──────────────────────────────┬───────────────────────────────────┘
                               │ HTTPS (QUIC on UDP 443)
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│                    BACKEND SERVER (Rust)                          │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              WebTransport Server (wtransport crate)           │ │
│  │  ┌──────────────────┐  ┌──────────────────────────────────┐  │ │
│  │  │ Session Manager  │  │ Room Coordinator                 │  │ │
│  │  │ (per-client      │  │ (maps clients → workspaces)      │  │ │
│  │  │  QUIC connection) │  │                                  │  │ │
│  │  └──────────────────┘  └──────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              CRDT Engine (Rust, shared library)               │ │
│  │  ┌──────────────────┐  ┌──────────────────────────────────┐  │ │
│  │  │ LWW-Element-Set  │  │ Delta Generator                  │  │ │
│  │  │ (state-based)    │  │ (byte-level diffs, compact)      │  │ │
│  │  └──────────────────┘  └──────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              Storage Layer                                     │ │
│  │  ┌──────────────────┐  ┌──────────────────────────────────┐  │ │
│  │  │ LSM Engine       │  │ Log Broker (audit trail)         │  │ │
│  │  │ (state snapshots)│  │ (every mutation logged)          │  │ │
│  │  └──────────────────┘  └──────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

### 2.3 Stream Allocation Strategy

| Stream Type | Count | Direction | Purpose |
|---|---|---|---|
| **Bidirectional (Stream 0)** | 1 per client | Client ↔ Server | CRDT state deltas, query parameter broadcasts, heartbeat/keepalive |
| **Unidirectional Ingestion** | 1 per data source | Server → Client | Raw columnar binary chunks (out-of-order delivery OK — chunk IDs handle ordering) |
| **Unidirectional Telemetry** | 1 per client | Client → Server | Client metrics (FPS, heap usage, query latency) for the server-side dashboard |
| **Datagrams** | N (dynamic) | Client ↔ Server | Low-latency sensor data packets (unreliable, fire-and-forget) |

## 3. CRDT Engine Design

### 3.1 LWW-Element-Set (Last-Write-Wins)

The collaborative state consists of:

- **Active filter set**: each peer's current filter parameters (column, operator, value)
- **Viewport state**: each peer's current zoom/pan coordinates
- **Query result set**: pointers to matching row indices (derived — not stored in CRDT)

The LWW-Element-Set uses a vector clock per peer:

```rust
struct LwwSet<V: Clone + Eq> {
    add_set: HashMap<V, u64>,     // Value → timestamp of last ADD
    remove_set: HashMap<V, u64>,  // Value → timestamp of last REMOVE
    peer_clock: u64,               // Monotonic clock for this peer
}

impl<V: Clone + Eq> LwwSet<V> {
    fn add(&mut self, value: V) {
        self.peer_clock += 1;
        self.add_set.insert(value, self.peer_clock);
    }

    fn remove(&mut self, value: V) {
        self.peer_clock += 1;
        self.remove_set.insert(value, self.peer_clock);
    }

    fn contains(&self, value: &V) -> bool {
        let add_ts = self.add_set.get(value).copied().unwrap_or(0);
        let remove_ts = self.remove_set.get(value).copied().unwrap_or(0);
        add_ts > remove_ts
    }

    fn merge(&mut self, other: &Self) {
        for (v, ts) in &other.add_set {
            self.add_set.entry(v.clone()).and_modify(|t| *t = (*t).max(*ts)).or_insert(*ts);
        }
        for (v, ts) in &other.remove_set {
            self.remove_set.entry(v.clone()).and_modify(|t| *t = (*t).max(*ts)).or_insert(*ts);
        }
        self.peer_clock = self.peer_clock.max(other.peer_clock);
    }
}
```

### 3.2 Delta Optimization

Instead of transmitting the entire `add_set` + `remove_set` on every merge (which could be hundreds of entries), the engine computes a **delta**: only entries whose timestamps changed since the last transmission.

```rust
fn compute_delta(&self, last_transmitted_clock: u64) -> LwwSet<V> {
    let mut delta = LwwSet::new();
    for (v, ts) in &self.add_set {
        if *ts > last_transmitted_clock {
            delta.add_set.insert(v.clone(), *ts);
        }
    }
    delta
}
```

At 42 bytes per delta (typical), 1,247 merges per session = ~52KB of total sync traffic.

## 4. Why WebTransport Over WebSockets (Quantified)

| Scenario | WebSocket | WebTransport |
|---|---|---|
| **Normal operation** | 5ms RTT, ordered delivery | 5ms RTT, parallel streams |
| **1% packet loss** | TCP retransmits block ALL messages in the stream for ~40ms (retransmission timeout) | Only the affected stream is delayed; other streams proceed independently |
| **Network switch (WiFi → 5G)** | Connection drops — full TCP + TLS re-handshake (~200ms) | QUIC connection migration — 0ms interruption (connection ID preserved) |
| **High-frequency sensor data (1KHz IMU)** | TCP backpressure throttles the entire connection | Unreliable datagrams fire-and-forget — no backpressure on critical state sync |

## 5. Implementation Roadmap

| Phase | Component | Status |
|---|---|---|
| **Phase 1** | Columnar engine + LRU pool + vectorized queries (`columnar-engine`) | DONE — 17 tests |
| **Phase 2** | `common-protocol` + `api-gateway` + `sql-engine` | DONE — compiled |
| **Phase 3** | WebTransport server (`wtransport` crate, QUIC transport) | Not yet built |
| **Phase 4** | CRDT engine (LWW-Element-Set, delta sync) | Not yet built |
| **Phase 5** | WebGPU + WGSL compute shaders | Not yet built |

## 6. Trade-offs & Defenses

| Decision | Pro | Con |
|---|---|---|
| **WebTransport over WebSocket** | No head-of-line blocking, connection migration, 0-RTT, datagram support | Browser support is newer (Chrome 97+, Edge 97+, Firefox not yet). Acceptable: Chrome + Edge = 85% market share. |
| **LWW-Element-Set over CRDT alternatives** | Simple, converges deterministically, small delta size | Last-write-wins means concurrent edits to the same filter are resolved by timestamp, not operation merging. Acceptable: filter parameters are small discrete values; conflicts are rare. |
| **State-based CRDT over operation-based** | State-based CRDTs are idempotent — receiving the same delta twice is safe. Operation-based CRDTs require exactly-once delivery. | State-based deltas are larger than operation-based (full entry vs. "add key=val"). At 42B/delta, this is negligible. |
| **`wtransport` crate for QUIC** | Pure Rust, no external C library dependency. Async-native with tokio. | Younger ecosystem than `quinn`. Trade-off: `wtransport` adds WebTransport semantics (sessions, streams) on top of `quinn`'s QUIC transport. |
