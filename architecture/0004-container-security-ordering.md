# ADR 0004: Container Security Operation Ordering

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-05-29 |
| **Deciders** | Keagan Ryan O'Connell |
| **Affects** | `container-engine/src/security/capabilities.rs`, `container-engine/src/security/no_new_privs.rs`, `container-engine/src/security/seccomp.rs` |

---

## Context

Container security is not a single operation — it's a sequence of progressively restrictive operations that collectively define the security boundary. Each operation reduces the process's capabilities. The order of these operations matters because:

1. Some operations are **irreversible** — once set, they cannot be unset by the process
2. Some operations **require privileges** that may be dropped by earlier operations
3. The **cumulative effect** is smaller than the sum of individual operations — an attacker who bypasses one layer still faces the others

Docker applies its security operations in a specific order, but that order is implicit in the Docker daemon's code and not documented as an explicit security property. For a principal engineer's container runtime, the ordering must be **explicit, documented, and verifiable**.

---

## Considered Alternatives

### 1. Arbitrary Ordering
Apply operations in whatever order the code happens to call them.

**Verdict:** This risks installing seccomp before dropping capabilities, which would allow a compromised init process to call privileged syscalls before the filter is in place. Every millisecond between operations is a window.

### 2. Single Combined Operation
Bundle all security operations into one atomic step.

**Verdict:** The kernel doesn't provide an atomic "secure this process" syscall. Each operation is a separate `prctl()` or `capset()` call. The best we can do is minimize the gap between them.

### 3. Irreversible Chain (Chosen)
Apply operations in order from least reversible to most restrictive, where each operation builds on the previous one.

**Verdict:** This matches Docker's approach but makes the ordering explicit and documents the rationale.

---

## Decision

Three security operations, applied in **irreversible order**:

### Step 1: `PR_SET_NO_NEW_PRIVS`

```rust
prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
```

**What it does:** Prevents the process from gaining new privileges. Once set, the process cannot:
- Execute setuid/setgid binaries (they run as the invoking user, not the file owner)
- Acquire new capabilities via `capset()`
- Transition to a new SELinux domain
- Enable `SECCOMP_MODE_FILTER` for the first time (but we install the filter before this flag — see below)

**Why first:** This is the foundation. It prevents the process from ever escalating its privileges, regardless of what vulnerabilities exist in the code. It's the simplest, strongest, and most irreversible operation.

**Irreversible:** Once set, `PR_SET_NO_NEW_PRIVS` cannot be unset by any process, including root. It persists across `execve()`.

### Step 2: Capability Bounding Set Drop

```rust
// Keep only these 5 capabilities from the original ~40:
KEPT_CAPS = [
    CAP_CHOWN,         // Change file ownership
    CAP_DAC_OVERRIDE,  // Bypass file read/write/execute permission checks
    CAP_FOWNER,        // Bypass permission checks on operations for file owner
    CAP_FSETID,        // Don't clear setuid/setgid bits on file modification
    CAP_KILL,          // Send signals to processes owned by other users
];
```

**What it does:** Reduces the bounding set from ~40 capabilities to exactly 5. All other capabilities — including `CAP_SYS_ADMIN` (essentially root), `CAP_NET_ADMIN` (network configuration), `CAP_SYS_PTRACE` (debug other processes), `CAP_SETUID`, `CAP_SETGID`, `CAP_NET_BIND_SERVICE` — are permanently dropped.

**Why second:** Capabilities are required for the seccomp installation in Step 3. By dropping them here, we ensure the process can no longer perform privileged kernel operations even if it later discovers a way to call additional syscalls.

**Irreversible:** Dropped capabilities cannot be re-acquired (NO_NEW_PRIVS from Step 1 ensures this).

### Step 3: Seccomp-BPF Filter Installation

```rust
prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog);
```

**What it does:** Installs a Berkeley Packet Filter program that the kernel evaluates before every syscall. The filter allows ~120 syscalls and kills the process on any other.

**Why third:** The seccomp filter is the narrowest and most application-specific constraint. It assumes that capabilities have already been dropped (Step 2) and that privilege escalation is blocked (Step 1). Installing it last ensures that the filter only needs to handle the syscalls that the now-capability-limited process can actually invoke.

**Irreversible:** Once installed, the filter cannot be removed, relaxed, or replaced. Even a compromised process with arbitrary code execution cannot disable the filter.

---

## The Ordering Chain

```
┌─────────────────────┐
│ PR_SET_NO_NEW_PRIVS │  ← Foundation: prevent any future privilege gain
├─────────────────────┤
│ Drop 35+ Capabilities│  ← Reduce attack surface: remove kernel-level power
├─────────────────────┤
│ Install Seccomp-BPF  │  ← Narrowest filter: restrict syscall interface
└─────────────────────┘
```

**Reasoning:** Each step constrains the attack surface for the next step. If the seccomp filter were installed first, the process could still exploit a vulnerability in a capability-gated syscall (e.g., `mount()` via `CAP_SYS_ADMIN`). By dropping capabilities first, we ensure that even if the seccomp filter has a bug, the process cannot call those syscalls because it lacks the required capabilities.

---

## Trade-offs

### Advantages
- **Defense in depth:** An attacker must bypass three independent security boundaries to execute arbitrary syscalls. Even if one layer has a bug, the others provide protection.
- **Explicit and auditable:** Each step is a single, well-defined kernel operation. The ordering is documented and can be verified by inspecting `/proc/<pid>/status` (Seccomp field) and `/proc/<pid>/status` (NoNewPrivs field).
- **Follows Docker's proven pattern:** This is the same ordering used by Docker, runc, and systemd-nspawn. It's battle-tested in production.

### Disadvantages
- **CAP_SETUID removal breaks `sudo`:** Processes inside the container cannot change their UID. This is intentional — `sudo` inside a container is a security anti-pattern — but it differs from Docker's default behavior, which retains `CAP_SETUID`.
- **Seccomp filter fragility:** Adding new functionality (e.g., io_uring, BPF-based networking) requires updating the syscall allowlist. A missing syscall causes the process to be killed with `SIGSYS`.
- **No container can run as root:** Even with `CAP_SYS_ADMIN` dropped, the root user inside the container still has access to all files with `CAP_DAC_OVERRIDE`. True non-root containers require user namespace mapping (UID 0 in container → UID 65534 on host), which is not yet implemented.

---

## Historical Note

During the May 2026 security audit, the initial capability set included `CAP_SETUID`, `CAP_SETGID`, and `CAP_NET_BIND_SERVICE`. These were removed from `KEPT_CAPS` as part of the audit remediation:
- `CAP_SETUID`/`CAP_SETGID` — prevents privilege escalation via suid binaries inside the container
- `CAP_NET_BIND_SERVICE` — no container workload needs to bind to ports <1024

The seccomp filter's `allowed_syscalls` array was also found to be empty during the audit — the filter architecture was correct, but the allowlist data was missing. This was fixed by populating the array with ~120 syscall numbers.

---

## Related Code

- `container-engine/src/security/no_new_privs.rs` — Safe wrapper for `prctl(PR_SET_NO_NEW_PRIVS)`
- `container-engine/src/security/capabilities.rs` — Capability bounding set management via `caps` crate
- `container-engine/src/security/seccomp.rs` — BPF filter construction and `prctl(PR_SET_SECCOMP)`
