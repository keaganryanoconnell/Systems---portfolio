# ADR 0005: Actor Model Concurrency vs OS Threads

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-05-29 |
| **Deciders** | Keagan Ryan O'Connell |
| **Affects** | `compute-orchestrator/src/actor/system.rs`, `compute-orchestrator/src/actor/message.rs` |

---

## Context

The compute-orchestrator manages distributed task execution across a cluster of worker nodes. Each task is an independent unit of computation — a micro-task split from a larger macro workload. Tasks arrive from the scheduler, execute on worker nodes, and produce results. Each task needs:

- Its own lifecycle (spawn → receive work → execute → reply → stop)
- Isolation from other tasks (a failing task should not crash the scheduler)
- Supervision (failed tasks should be retried or reported)
- Message delivery (results must be returned to the scheduler)

The naive approach — one OS thread per task — works for 10 tasks but fails catastrophically at 1,000 tasks. Linux limits processes to ~32,000 threads per PID namespace, and each thread consumes 2-8MB of stack space and kernel scheduling overhead.

---

## Considered Alternatives

### 1. OS Threads (`std::thread::spawn`)

Each actor runs as a dedicated OS thread with its own stack and kernel scheduling.

**Pros:**
- Strong isolation: a thread crash kills only that thread (with `catch_unwind`)
- Preemptive scheduling: the kernel ensures fairness
- Trivial to implement: `thread::spawn(move || loop { ... })`

**Cons:**
- **Memory overhead:** 2-8MB of stack space per thread × 1,000 tasks = 2-8 GB of memory
- **Scheduling overhead:** Context switches between 1,000 threads consume 5-10% of CPU
- **Thread limit:** Linux caps threads per process (~32,000); approaching this limit causes `EAGAIN` errors
- **Startup latency:** Creating a new OS thread takes ~100μs

**Verdict:** Does not scale beyond ~100 concurrent actors. Rejected for our use case of hundreds of tasks.

### 2. Async Tasks with Shared State (`Arc<Mutex<State>>`)

All tasks share a single-threaded or thread-pooled async runtime with a shared mutable state.

**Pros:**
- Memory efficient: one stack for the runtime, not per-task
- Fast task spawning: `tokio::spawn` is ~1μs

**Cons:**
- **Shared state is a bug factory:** Every task holds `Arc<Mutex<HashMap<...>>>` — deadlocks, lock contention, and ordering bugs are inevitable at scale
- **No isolation:** A panicking task brings down the entire runtime unless every task is wrapped in `catch_unwind`
- **No supervision:** There's no built-in way to restart a failed task

**Verdict:** Too fragile for a distributed system. Rejected in favor of message-passing isolation.

### 3. Actor Model with `tokio::spawn` + `mpsc` (Chosen)

Each actor is a `tokio::spawn` task with its own `mpsc::channel` mailbox. Communication is exclusively via `ActorMessage` envelopes. The `ActorSystem` manages lifecycle and supervision.

**Pros:**
- **Memory efficient:** ~2KB per actor (tokio task + mpsc channel), not 2MB
- **Spawns 1,000 actors in ~3ms** (vs ~100ms for 1,000 OS threads)
- **Message-passing isolation:** No shared state between actors. Actors communicate by sending messages to mailboxes. A failing actor cannot corrupt another actor's state.
- **Supervision built-in:** The `ActorSystem` detects panics (via the `tokio::select!` in the actor loop), counts restarts, and applies supervision strategies (OneForOne, AllForOne).

**Cons:**
- **No memory isolation:** All actors share the same process address space. A memory corruption bug in one actor can affect others. This is inherent to userspace concurrency.
- **No preemptive fairness:** `tokio` tasks are cooperatively scheduled. A CPU-bound actor that never `.await`s will starve other actors. This is mitigated by using `tokio::task::spawn_blocking` for CPU-intensive work.
- **Message delivery is async:** There's no guarantee that a message sent via `system.send(msg)` has been received and processed. The caller must implement its own acknowledgment protocol if needed.

---

## Decision

Implement an actor system with the following architecture:

### Actor Lifecycle

```
Created → Starting → Running → Stopping → Stopped
                              ↘ Failed (after max_restarts exceeded)
```

### Message Envelope

```rust
pub struct ActorMessage {
    pub sender: ProcessId,      // {node_id}:{actor_id} — globally unique address
    pub recipient: ProcessId,   // destination actor
    pub msg_type: MessageType,  // TaskDispatch | TaskResult | ActorSpawn | ActorStop | Heartbeat
    pub payload: Vec<u8>,       // bincode-serialized message body (max 1MB)
}
```

### Supervision

| Strategy | Behavior |
|---|---|
| **OneForOne** | Restart only the failed actor. Other siblings are unaffected. |
| **AllForOne** | Restart all sibling actors when one fails. Used when sibling actors share state or have dependencies. |

### Resource Limits

| Limit | Value | Rationale |
|---|---|---|
| Mailbox size | 1..65536 messages | Bounded to prevent unbounded memory growth from unprocessed messages |
| Payload size | 1 MB | Prevents OOM via oversized bincode payloads |
| Max restarts | 3 | Prevents infinite restart loops (a failing actor is eventually stopped, not retried forever) |
| Actor ID counter | `AtomicU64` with `SeqCst` | Monotonic ID generation prevents collisions under concurrent `spawn()` calls |

---

## Trade-offs

### Advantages
- **Scale:** 1,000 actors can run in a single process with ~2 MB of overhead (vs 2-8 GB for threads). Spawning is ~1,000x faster than OS thread creation.
- **Isolation through messaging:** Actors communicate via typed `ActorMessage` envelopes, not shared state. This eliminates entire classes of concurrency bugs (deadlocks, race conditions on shared data, lock-ordering issues).
- **Supervision:** The `ActorSystem` automatically detects panics, counts restarts, and applies configurable supervision strategies. This means individual actor failures don't bring down the entire system.
- **Observability:** Every actor has a `ProcessId` (`{node_id}:{actor_id}`). The `ActorSystem` exposes `list_actors()` and `get_state()` for introspection. Combined with OpenTelemetry tracing, this gives full visibility into actor lifecycles.

### Disadvantages
- **No preemptive scheduling:** A misbehaving actor that never yields (e.g., an infinite loop without `.await`) will block the tokio runtime. This is a fundamental limitation of cooperative multitasking. Mitigation: CPU-bound work should use `tokio::task::spawn_blocking`.
- **No hardware isolation:** All actors share the same process. A segmentation fault in one actor (via `unsafe` code in a native dependency) will crash the entire process, killing all actors. For hardware-level isolation, actors should be deployed as separate processes or containers.
- **Message delivery is fire-and-forget:** The `system.send(msg)` API does not wait for a response. If the recipient actor is stopped or its mailbox is full, the message is silently dropped. Callers that need guaranteed delivery must implement their own request-response protocol on top of the actor system.

---

## Historical Note

This actor system was designed as part of Project 10 (the compute-orchestrator crate). Prior to this, the workspace had **no actor model code** — all concurrency was managed via `std::thread::spawn` and `Arc<Mutex<T>>` patterns. The actor system introduces a structured concurrency model that the project can build on for future distributed workloads.

During the May 2026 security audit, the following actor system improvements were made:
- **Mailbox size validation:** Added a check that `mailbox_size` is in range 1..65536 (previously unbounded)
- **Payload size validation:** Added `MAX_PAYLOAD_SIZE` (1MB) check in `ActorMessage::new()`
- **Lock ordering fix:** The shutdown path was locking `states` before `actors` (inconsistent with `spawn()`). Now both paths lock `actors` first.
- **MessageType validation:** `from_u32()` now returns `Option<Self>` instead of silently defaulting to `TaskDispatch` on unknown values.

---

## Related Code

- `compute-orchestrator/src/actor/system.rs` — `ActorSystem` with `spawn()`, `send()`, `get_state()`, `list_actors()`
- `compute-orchestrator/src/actor/message.rs` — `ActorMessage` envelope, `ActorState` enum
- `compute-orchestrator/src/actor/pid.rs` — `ProcessId` (`{node_id}:{actor_id}`)
- `compute-orchestrator/src/network/serializer.rs` — `MessageType` enum with `from_u32()` validation
- `compute-orchestrator/src/network/transport.rs` — TLS-enabled `send_message()` / `recv_message()`
