# ADR 0003: SPSC Ring Buffer Memory Ordering

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-05-29 |
| **Deciders** | Keagan Ryan O'Connell |
| **Affects** | `core-sys/src/spsc.rs`, `log-broker/src/buffer.rs` |

---

## Context

Both the core-sys library and the log-broker use ring buffers for lock-free message passing between threads. The SPSC (Single Producer, Single Consumer) queue in `core-sys` transfers telemetry events to a background logging daemon. The ring buffer in `log-broker` accumulates produce payloads before sequential disk writes.

Both require **correct concurrent access without locks**. The correctness depends entirely on atomic operations and memory ordering — get these wrong, and the data races produce undefined behavior visible only under load.

---

## Considered Alternatives

### 1. Mutex\<VecDeque\>
A `Mutex<VecDeque<T>>` with `push_back()` and `pop_front()`.

**Pros:** Trivially correct. No reasoning about atomics required.

**Cons:** Mutex acquisition introduces kernel scheduling overhead (~500ns uncontended, microseconds when contended). Under high throughput, the mutex becomes the bottleneck — threads spend more time waiting for the lock than doing actual work. This defeats the purpose of a ring buffer, which is to decouple producer and consumer.

### 2. SeqCst Everywhere
Use `Ordering::SeqCst` for every atomic load and store.

**Pros:** Strongest guarantee. All threads see a single total order of operations. Impossible to get wrong.

**Cons:** On x86_64, `SeqCst` loads are free (MOV), but `SeqCst` stores require `XCHG` or `MFENCE` — each costs ~20-40 cycles. On ARM/PowerPC (weak memory models), the penalty is even higher. For a ring buffer doing millions of pushes per second, those cycles add up to milliseconds of latency.

### 3. Acquire/Release with `compiler_fence` (Initially Chosen)

The initial implementation used:
```rust
std::sync::atomic::compiler_fence(Ordering::SeqCst);
```

**Pros:** `compiler_fence` prevents the compiler from reordering instructions across the fence. It generates no CPU instructions — it's purely a compiler hint.

**Cons:** `compiler_fence` does **not** prevent the CPU from reordering memory operations. On ARM and PowerPC (weakly-ordered architectures), the CPU may reorder writes to the buffer data before the write to the head index, allowing a concurrent reader to see partially-written data. This was a **latent data race** on non-x86 architectures.

### 4. Acquire/Release with `fence(SeqCst)` (Chosen)

The corrected implementation:
```rust
std::sync::atomic::fence(Ordering::SeqCst);
```

`fence(SeqCst)` is a **hardware memory barrier** — it emits a `DMB SY` instruction on ARM, which prevents the CPU from reordering memory operations across the fence. On x86_64, `fence(SeqCst)` is equivalent to `MFENCE`.

---

## Decision

Use the following memory ordering protocol for all ring buffer operations:

### Producer (write path)
```
1. head.load(Relaxed)       — read producer position (no ordering needed)
2. tail.load(Acquire)       — read consumer position (must see the latest write)
3. Write data to buffer     — memcpy to the ring region
4. fence(SeqCst)           — hardware barrier: buffer writes visible before head update
5. head.store(Release)     — publish new head position (all prior writes visible)
```

### Consumer (read path)
```
1. tail.load(Relaxed)       — read consumer position
2. head.load(Acquire)       — read producer position (must see latest buffer writes)
3. Read data from buffer    — memcpy from the ring region
4. fence(SeqCst)           — hardware barrier: buffer reads complete before tail update
5. tail.store(Release)     — publish new tail position (consumer has consumed the data)
```

### Rationale for each ordering

| Operation | Ordering | Why |
|---|---|---|
| head.load (producer) | Relaxed | We only need the current value; no ordering with other operations |
| tail.load (producer) | Acquire | Must see all consumer writes before we write — prevents overwriting unread data |
| head.store (producer) | Release | Must ensure buffer writes are visible before the consumer sees the updated head |
| tail.load (consumer) | Relaxed | We only need the current value |
| head.load (consumer) | Acquire | Must see all producer writes before we read — prevents reading stale data |
| tail.store (consumer) | Release | Must ensure buffer reads are complete before the producer sees the updated tail |
| fence (both) | SeqCst | Hardware barrier: prevents CPU-level reordering of buffer access vs. index access |

---

## Trade-offs

### Advantages
- **Correct on all architectures:** The `fence(SeqCst)` emits appropriate hardware barriers on ARM (`DMB SY`), PowerPC (`sync`), and x86_64 (`MFENCE`). The code is correct everywhere.
- **Zero overhead on x86_64:** x86_64's Total Store Order (TSO) model already provides the guarantees we need. The `fence(SeqCst)` is an `MFENCE` instruction that costs ~33 cycles, but it only executes once per batch (not once per element). In practice, the overhead is negligible.
- **Lock-free:** No mutex acquisition. Producer and consumer never block each other. The only contention is on cache lines for the head/tail indices — and since producer writes head and consumer writes tail, these are on different cache lines. No false sharing.

### Disadvantages
- **ARM penalty:** On ARM, `fence(SeqCst)` emits `DMB SY`, which costs ~20-30 cycles. For low-throughput workloads, this is negligible. For millions of operations per second, it adds measurable latency.
- **UnsafeCell required:** The ring buffer uses `UnsafeCell<Vec<u8>>` to signal interior mutability to the compiler. Without it, casting `&self.buffer` to `*mut u8` violates Rust's aliasing rules. The `UnsafeCell` wrapper makes the intent explicit and satisfies Miri (Rust's undefined behavior detector).
- **SPSC contract:** The correctness proofs for this design assume exactly one producer and one consumer. If two threads try to `push()` concurrently, the `head` counter can race and produce overlapping writes. This is documented in the `Sync` impl safety comments but not enforced at runtime.

---

## Historical Note

During the May 2026 security audit, the `compiler_fence` was identified as a **HIGH** severity finding. The fix replaced it with `std::sync::atomic::fence(Ordering::SeqCst)` across all three ring buffer methods (`try_write`, `try_read`, `drain_into`). The `Sync` impl on `SpscQueue` was also restored with additional safety documentation after initially removing it broke the `OnceLock` static in the telemetry logger.

---

## Related Code

- `core-sys/src/spsc.rs` — `SpscQueue<T, N>` with `Send` + `Sync` impls and `push()`/`pop()` methods
- `log-broker/src/buffer.rs` — `RingBuffer` with `UnsafeCell<Vec<u8>>`, `try_write()`/`try_read()`/`drain_into()`
