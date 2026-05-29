# ADR 0002: Binary Telemetry Protocol vs JSON

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-05-29 |
| **Deciders** | Keagan Ryan O'Connell |
| **Affects** | `ui-control-center/src/app/utils/tauri.ts`, Tauri IPC layer |

---

## Context

The control center dashboard polls cluster node telemetry at 500ms intervals. Each node has 10 fields: node ID, role, status, CPU %, arena memory allocated, arena memory total, active file descriptors, replication lag (ms), LSM storage bytes, and IOPS. With 5 nodes in the cluster, this is 50 data points every 500ms.

The frontend needs to consume this data with minimal overhead — the dashboard includes real-time canvas charts, animated cluster maps, and virtualized log streams, all running at 60 FPS. Every millisecond spent parsing JSON is a millisecond not spent rendering frames.

---

## Considered Alternatives

### 1. JSON
```json
[
  {"nodeId": 1, "role": "Leader", "status": "Healthy", "cpu": 42, "arenaMemoryAllocated": 512,
   "arenaMemoryTotal": 1024, "activeFdPool": 64, "replicationLag": 3,
   "lsmStorageBytes": 123456789012, "iops": 15000},
  ...
]
```
- **Size:** ~160 bytes per node with field names, ~800 bytes for 5 nodes
- **Parsing overhead:** `JSON.parse()` creates JavaScript objects with string-keyed properties, allocating heap memory and triggering GC
- **Validation:** Requires runtime checking that all fields exist and have correct types

### 2. Protocol Buffers / FlatBuffers
- **Size:** ~60-70 bytes per node (no field names, varint encoding)
- **Parsing overhead:** Requires a generated schema and protobuf decoder library
- **Dependency:** Adds a compile-time dependency to both the Rust backend and the TypeScript frontend

### 3. Custom 32-Byte Binary (Chosen)
A fixed-width binary format with exactly 32 bytes per node entry:

```
Bytes 0-3:   Magic (0xAABBCCDD) — misalignment detection
Byte 4:      Node ID (uint8)
Byte 5:      Role (uint8: 0=Leader, 1=Follower, 2=Candidate)
Byte 6:      Status (uint8: 0=Healthy, 1=Degraded, 2=Offline)
Byte 7:      CPU % (uint8, 0-100)
Bytes 8-11:  Arena Memory Allocated (uint32)
Bytes 12-15: Arena Memory Total (uint32)
Bytes 16-17: Active File Descriptors (uint16)
Bytes 18-19: Replication Lag, ms (uint16)
Bytes 20-27: LSM Storage Bytes (uint64, big-endian)
Bytes 28-31: IOPS (uint32)
```

- **Size:** Exactly 32 bytes per node, 160 bytes for 5 nodes. **80% smaller than JSON.**
- **Parsing overhead:** `DataView.getUint8(offset)`, `DataView.getUint32(offset)` — zero-copy reads from the underlying `ArrayBuffer`. No string allocation, no GC pressure.
- **Validation:** Magic byte check at byte 0 catches misaligned or corrupted buffers in O(1). Invalid data is skipped, not parsed into malformed objects.

---

## Decision

Use the custom 32-byte binary protocol for all telemetry transport. Implement matching encode/decode functions:

- **`encodeNodeTelemetry(nodes: NodeTelemetry[]): Uint8Array`** — Rust backend serializes nodes into 32-byte entries. Used in mock mode for protocol verification.
- **`decodeNodeTelemetry(buffer: Uint8Array): NodeTelemetry[]`** — Frontend parses `Uint8Array` from Tauri IPC or WebSocket into typed `NodeTelemetry` objects. Validates magic bytes; skips misaligned entries.

### Data Flow

```
Rust Backend                 Tauri IPC               JavaScript Frontend
┌──────────────┐    binary    ┌──────────┐    Uint8Array    ┌──────────────┐
│ NodeTelemetry│ ──────────▶  │ safeInvoke│ ─────────────▶ │decodeTelemetry│
│   (5 nodes)  │  160 bytes   │          │   ArrayBuffer  │  → state      │
└──────────────┘              └──────────┘                └──────────────┘
```

The roundtrip encode → decode is verified in Vitest unit tests (`tauri.test.ts`, 13 tests) to ensure exact fidelity for all fields, including BigInt handling for `lsmStorageBytes`.

---

## Trade-offs

### Advantages
- **5x bandwidth reduction:** 160 bytes vs 800 bytes per poll. At 2Hz (500ms intervals), this saves ~1.3 KB/s — negligible for a single client, but meaningful at scale.
- **Zero-copy parsing:** `DataView` methods read integers directly from the `ArrayBuffer` without intermediate string allocation. This is the same approach used by WebGL and WebAssembly for binary data transfer.
- **Built-in corruption detection:** Magic bytes (0xAABBCCDD) at the start of each 32-byte entry immediately detect misaligned buffers, partial writes, or bit-flip errors. Corrupted entries are skipped rather than producing garbage data.
- **No schema dependency:** The protocol is self-documenting (the byte layout IS the schema). There's no `.proto` file to keep in sync between Rust and TypeScript.

### Disadvantages
- **Not human-readable:** A raw binary buffer cannot be inspected with `console.log()` or `curl`. Debugging requires a hex viewer or the `decodeNodeTelemetry` function.
- **Fragile to field changes:** Adding a new field requires updating both `encode` and `decode` at matching byte offsets. A mismatch produces silently incorrect data. This is mitigated by the Vitest roundtrip tests, which would catch any offset misalignment.
- **Endianness coupling:** The protocol uses big-endian (`setUint32`, `setBigUint64`) for consistency. Little-endian machines (x86_64) require byte-swapping on both encode and decode. `DataView` handles this transparently since it always operates in the specified endianness, but it adds CPU cycles compared to native-endian writes.

---

## Rejected Alternatives

- **"Just use JSON, it's simpler"** — JSON is simpler for the developer, but not for the machine. The 5x size difference and parser overhead are unacceptable for a real-time dashboard rendering at 60 FPS. Every millisecond counts when you're also rendering canvas charts, animated Bezier curves, and virtualized log streams.
- **"Use MessagePack"** — MessagePack is a binary JSON alternative that's more compact than JSON but still requires a decoder library. At ~40-50 bytes per node, it's good, but the custom 32-byte format is still better and requires zero dependencies.
- **"Use Cap'n Proto"** — Excellent for zero-copy in C++/Rust, but the JavaScript implementation is less mature and adds significant bundle weight. Overkill for a 5-node telemetry feed.

---

## Related Code

- `ui-control-center/src/app/utils/tauri.ts` — `encodeNodeTelemetry()`, `decodeNodeTelemetry()`, `generateMockNodesTelemetry()`
- `ui-control-center/src/app/utils/tauri.test.ts` — 13 Vitest tests covering roundtrip fidelity, magic byte validation, CPU clamping, and BigInt handling
- `src-tauri/src/main.rs` — Tauri IPC command `get_node_telemetry()` returns `Vec<NodeTelemetry>` matching the binary protocol
