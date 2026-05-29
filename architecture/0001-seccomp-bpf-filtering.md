# ADR 0001: Seccomp-BPF Syscall Filtering

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-05-29 |
| **Deciders** | Keagan Ryan O'Connell |
| **Affects** | `container-engine/src/security/seccomp.rs`, `container-engine/src/security/no_new_privs.rs` |

---

## Context

Container security requires restricting which Linux syscalls a containerized process can invoke. A process that can call `mount()`, `pivot_root()`, `kexec_load()`, or `reboot()` has effectively escaped its namespace boundary. Docker's default seccomp profile blocks approximately 44 syscalls out of over 300 — but the profile is opaque, version-dependent, and varies between container runtimes.

For a principal-level systems engineer, container security must be **explicit, auditable, and bounded**. Every syscall in the allowlist must have a documented justification. Every syscall not in the allowlist must be killed at the kernel boundary.

---

## Considered Alternatives

### 1. Docker's Default Seccomp Profile
The Docker daemon applies a JSON-based seccomp profile at container creation time. It's well-tested and broadly compatible. However:
- The profile blocks only 44 syscalls — the remaining ~260 are implicitly allowed
- The profile differs between Docker CE and Enterprise editions
- It's opaque: you cannot inspect the active filter from inside the container
- It's removed if the container runs with `--privileged`

**Verdict:** Too permissive, too opaque, not portable.

### 2. AppArmor / SELinux
Mandatory Access Control (MAC) frameworks provide filesystem-level isolation and labeling. They complement seccomp but do not replace it:
- They control resource access, not system call invocation
- Policies are distribution-specific (AppArmor on Ubuntu, SELinux on RHEL)
- Policy languages are complex and error-prone

**Verdict:** Useful as a second layer, but insufficient as the primary syscall boundary.

### 3. Custom Seccomp-BPF (Chosen)
Write a Berkeley Packet Filter program that the kernel evaluates before executing each syscall. The filter is installed via `prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog)` and, once installed, cannot be removed or relaxed by the process.

**Verdict:** Kernel-enforced, programmatic, exact. This is the right level of control.

---

## Decision

We install a seccomp-BPF filter that:

1. **Validates the architecture** — loads `seccomp_data.arch`, compares to `AUDIT_ARCH_X86_64`. If the architecture doesn't match (e.g., a 32-bit process trying to use the filter), the process is immediately killed. This prevents architecture-based filter bypass.

2. **Loads the syscall number** — reads `seccomp_data.nr` (the syscall number).

3. **Linear-scans an allowlist of ~120 syscalls** — for each allowed syscall number, generates a BPF instruction: "if A == allowed_nr, ALLOW; else continue." In production, this would be a binary search over a sorted list, but the linear scan is correct and auditable.

4. **Kills on any unknown syscall** — if no match is found after scanning the allowlist, the instruction sequence terminates with `SECCOMP_RET_KILL_PROCESS`.

### Syscall Allowlist Categories

| Category | Count | Examples |
|---|---|---|
| I/O | ~15 | read, write, open, close, stat, lseek, poll, pread64, pwrite64, readv, writev |
| Memory | ~8 | mmap, mprotect, munmap, brk, mremap, msync, mincore, madvise |
| Signals | ~3 | rt_sigaction, rt_sigprocmask, rt_sigreturn |
| IPC | ~6 | shmget, shmat, shmctl, shmdt, semget, semop, semctl, msgget, msgsnd, msgrcv, msgctl |
| File system | ~20 | mkdir, rmdir, rename, link, unlink, symlink, readlink, chmod, fchmod, truncate, ftruncate, getdents, getcwd, chdir, fchdir, access, pipe |
| Process | ~12 | clone, fork, vfork, execve, exit, wait4, kill, getpid, getppid, setsid, setpgid |
| Network | ~15 | socket, connect, accept, sendto, recvfrom, sendmsg, recvmsg, bind, listen, shutdown, getsockname, getpeername, setsockopt, getsockopt |
| Time | ~4 | gettimeofday, nanosleep, clock_gettime, times |
| User/Group | ~15 | getuid, getgid, geteuid, getegid, setuid, setgid, setreuid, setregid, getgroups, setresuid, getresuid, setresgid, getresgid, getpgid, setfsuid, setfsgid, getsid |
| Capabilities | ~2 | capget, capset |
| Futex/Epoll | ~5 | futex, epoll_create, epoll_ctl, epoll_wait, epoll_create1, eventfd2 |
| Misc | ~8 | ioctl, dup, dup2, fcntl, flock, fsync, fdatasync, prctl, arch_prctl, set_tid_address, set_robust_list |

**Total: ~120 syscalls allowed. Everything else: killed.**

---

## Trade-offs

### Advantages
- **Kernel-enforced:** The BPF filter runs in kernel context before the syscall executes. It cannot be bypassed by statically-linked binaries, raw `int 0x80` instructions, or `LD_PRELOAD` interposition.
- **Irreversible:** Once installed via `PR_SET_SECCOMP`, the filter cannot be removed or relaxed. Even a compromised process that gains arbitrary code execution cannot disable the filter.
- **Explicit:** Every allowed syscall has a documented justification. Any syscall not in the allowlist is killed. There is no implicit "allow everything else" fallback.

### Disadvantages
- **Maintenance burden:** Adding new functionality (e.g., io_uring support) requires updating the allowlist with new syscall numbers. The list must be kept in sync with the Linux kernel's syscall table.
- **Testing complexity:** Verifying that all required syscalls are in the allowlist requires running the workload and checking for `SIGSYS` (the signal delivered when seccomp kills a process). This is an integration test, not a unit test.
- **Architecture coupling:** The filter includes an x86_64 architecture check. Running on aarch64 would require a separate filter with different audit arch values and potentially different syscall numbers.

---

## Rejected Alternatives

- **"Allow all syscalls"** — defeats the purpose of a container security boundary. The container process should not be able to call `mount()`, `kexec_load()`, or `reboot()`.
- **"Block only known-dangerous syscalls"** — this is Docker's approach. It's a deny-list, not an allow-list. Any new dangerous syscall added to the kernel (e.g., io_uring when it was new) would be implicitly allowed until someone updates the deny-list. An allow-list is safer by default.
- **"No seccomp at all, rely on namespaces"** — namespaces isolate the view of system resources. They do not prevent a process from calling dangerous syscalls. A process in a PID namespace can still call `reboot()` and reboot the physical host if it has `CAP_SYS_BOOT`.

---

## Security Audit Note

During the 2026 security audit, the `allowed_syscalls` array was found to be **empty** — the filter killed every syscall, making the container non-functional. This was a configuration bug, not a design flaw. The fix populated the array with the ~120 syscalls documented above. The filter architecture was correct; the allowlist data was missing.
