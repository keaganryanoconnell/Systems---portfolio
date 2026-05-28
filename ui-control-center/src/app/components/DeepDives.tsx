"use client";

import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

interface Dive {
  title: string;
  summary: string;
  content: string;
}

const DIVES: Dive[] = [
  {
    title: "Building a Container Runtime from Scratch (Without Docker)",
    summary: "How I implemented namespace isolation, cgroups v2 resource control, OverlayFS, and seccomp-BPF in pure Rust — no Docker, no runc, no libcontainer.",
    content: `The container-engine crate is a production-grade Linux container runtime built entirely from first principles using Linux kernel APIs.

Architecture:
• Namespace isolation: clone() with CLONE_NEWPID | NEWNS | NEWUTS | NEWIPC | NEWNET flags, 2MB child stack, sethostname, MS_PRIVATE mount propagation.
• Filesystem: pivot_root + umount2(MNT_DETACH), OverlayFS with read-only lowerdir and writable upperdir. Virtual filesystems mounted: /proc, /sys, /dev, /dev/pts, /dev/mqueue, /run, /tmp. 11 kernel paths masked, 4 set to readonly.
• cgroups v2: memory.max, memory.high, memory.swap.max, memory.oom.group; cpu.weight, cpu.max (CFS quota/period); io.weight, io.max (rbps/wbps/riops/wiops per device); pids.max; device access controlled via BPF programs.
• Security stack (ordered irreversibly): PR_SET_NO_NEW_PRIVS → capability bounding set (7 kept, 35+ dropped via caps crate) → seccomp-BPF prctl filter with architecture validation and ~50 explicit syscall allowlist.
• Networking: veth pairs created in container + host namespaces, cbr0 bridge (10.88.0.0/16), IP pool allocator with lease persistence, iptables MASQUERADE NAT + DNAT port mapping, /etc/resolv.conf/hosts/hostname generation.
• Process init: Two-stage fork — PID 1 init reaper loop + signal forwarding + zombie reaping, child process execs the user command.
• CLI: 10 subcommands — run, exec, kill, ps, stats, inspect, logs, pause, resume, rm.
• Error handling: ContainerError enum (13 variants) with thiserror, ContainerResult<T>, validated state machine for container lifecycle with JSON persistence at /var/run/container-engine/{id}/state.json.
• Integration tests: 6 test cases covering builder validation, state transitions, ID generation, stats serde, signal parsing, error display.`,
  },
  {
    title: "The Binary Protocol: Why 32 Bytes Instead of JSON",
    summary: "Design decisions behind the zero-copy telemetry protocol — magic bytes, DataView parsing, and why JSON was the wrong choice for real-time metrics.",
    content: `The telemetry system uses a custom binary protocol with exactly 32 bytes per node entry:

Byte layout:
• Bytes 0-3: Magic bytes (0xAABBCCDD) for packet alignment validation
• Byte 4: Node ID (uint8)
• Byte 5: Role (uint8: 0=Leader, 1=Follower, 2=Candidate)
• Byte 6: Status (uint8: 0=Healthy, 1=Degraded, 2=Offline)
• Byte 7: CPU (uint8, 0-100)
• Bytes 8-11: Arena memory allocated (uint32)
• Bytes 12-15: Arena memory total (uint32)
• Bytes 16-17: Active file descriptors (uint16)
• Bytes 18-19: Replication lag (uint16, ms)
• Bytes 20-27: LSM storage bytes (uint64, big-endian)
• Bytes 28-31: IOPS (uint32)

Why binary instead of JSON:
• JSON serialization of 5 nodes would be ~800 bytes with field names; binary is exactly 160 bytes (5 × 32).
• No parsing overhead — DataView reads integers directly from the buffer without string allocation.
• At 2Hz polling (500ms intervals), that's 160 bytes per tick vs 800 bytes — 80% less bandwidth.
• The magic bytes catch misaligned buffers immediately — corrupt packets are detected in O(1).
• encodeNodeTelemetry and decodeNodeTelemetry in tauri.ts form a perfect roundtrip for protocol verification in tests.

The DataView approach:
The browser's DataView provides zero-copy access to the underlying ArrayBuffer. Instead of splitting strings and parsing JSON, we read typed integers directly: view.getUint8(offset), view.getUint32(offset), view.getBigUint64(offset). This is the same approach used in WebGL and WebAssembly — it's the fastest way to decode binary data in JavaScript.`,
  },
  {
    title: "Lock-Free Ring Buffers: Atomics, Fences, and Cache Lines",
    summary: "How the SPSC queue in core-sys achieves zero-allocation message passing using atomic operations, compiler fences, and careful memory ordering.",
    content: `The SPSC (Single Producer, Single Consumer) ring buffer in core-sys/src/spsc.rs is designed for zero-allocation message passing between threads:

Design:
• Pre-allocated Vec<T> with power-of-2 capacity — no allocations during push/pop.
• Two AtomicUsize indices: head (producer) and tail (consumer).
• Producer writes to head, consumer reads from tail.
• compiler_fence(SeqCst) prevents instruction reordering within a single thread.

Memory ordering:
• Producer: head.load(Relaxed), tail.load(Acquire) → write data → compiler_fence → head.store(Release).
  - Acquire on tail ensures we see all consumer updates before writing.
  - Release on head ensures our writes are visible before the consumer sees the updated head.
• Consumer: tail.load(Relaxed), head.load(Acquire) → read data → compiler_fence → tail.store(Release).
  - Acquire on head ensures we see all producer writes before reading.
  - Release on tail ensures our reads are complete before the producer sees the updated tail.

Why not SeqCst everywhere:
On x86_64, Acquire/Release ops map to MOV instructions — they're free. SeqCst ops map to XCHG or MFENCE which flush the store buffer — ~20-40 cycles. For a high-throughput ring buffer consuming millions of messages per second, those cycles add up.

Cache line considerations:
The buffer uses a single contiguous Vec — all data is on adjacent cache lines. The head and tail indices (AtomicUsize) are on separate fields, avoiding false sharing. On ARM/PowerPC where the memory model is weaker, the compiler_fence prevents the compiler from reordering the write to the buffer before the store to head.

Benchmarks (Criterion):
The SPSC queue benchmarks show consistent ~50ns per push/pop pair on x86_64, with zero allocations after initial setup. Throughput is limited by memory bandwidth, not synchronization overhead.`,
  },
  {
    title: "Seccomp-BPF: Filtering Syscalls at the Kernel Level",
    summary: "How the container engine uses Berkeley Packet Filter programs to restrict which Linux syscalls a container process can invoke — before they reach the kernel.",
    content: `The seccomp-BPF filter in container-engine/src/security/seccomp.rs installs a Berkeley Packet Filter program that the kernel evaluates for every syscall:

Architecture:
• BPF program is defined as an array of sock_filter structs (opcode + jump target + value).
• Installed via prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog) — after this point, the process cannot remove the filter.
• Architecture validation: the first BPF instruction checks the audit_arch value to ensure the filter matches x86_64. If the architecture doesn't match, the process is killed immediately.

Allowed syscalls (~50):
The whitelist includes only syscalls needed for typical container workloads:
• read, write, open, close, stat, fstat, lstat, poll, lseek, mmap, mprotect, munmap, brk
• rt_sigaction, rt_sigprocmask, rt_sigreturn, ioctl, pread64, pwrite64
• readv, writev, access, pipe, select, sched_yield, mremap, msync
• mincore, madvise, shmget, shmat, shmctl, dup, dup2, pause, nanosleep
• getitimer, alarm, setitimer, getpid, sendfile, socket, connect
• accept, sendto, recvfrom, sendmsg, recvmsg, shutdown, bind, listen
• getsockname, getpeername, socketpair, setsockopt, getsockopt
• clone, fork, vfork, execve, exit, wait4, kill, uname, semget, semop
• semctl, shmdt, msgget, msgsnd, msgrcv, msgctl, fcntl, flock
• fsync, fdatasync, truncate, ftruncate, getdents, getcwd, chdir
• rename, mkdir, rmdir, creat, link, unlink, symlink, readlink
• chmod, fchmod, chown, fchown, lchown, umask, gettimeofday
• getrlimit, getrusage, sysinfo, times, ptrace (blocked), getuid
• syslog, getgid, setuid, setgid, geteuid, getegid, setpgid
• getppid, getpgrp, setsid, setreuid, setregid, getgroups
• setresuid, getresuid, setresgid, getresgid, getpgid, setfsuid
• setfsgid, getsid, capget, capset

Blocked syscalls:
• mount, umount2, pivot_root — blocked after container setup
• ptrace — blocked to prevent process inspection
• kexec_load, reboot — blocked to prevent system-level actions
• init_module, finit_module — blocked to prevent kernel module loading
• clock_settime, settimeofday — blocked to prevent time manipulation
• All network-related syscalls not explicitly needed by the workload

Why BPF instead of syscall interposition (LD_PRELOAD):
BPF runs in the kernel before the syscall executes — it cannot be bypassed. LD_PRELOAD intercepts libc calls in userspace — a statically linked binary or raw syscall instruction (int 0x80 / syscall instruction) bypasses it. BPF is the only way to guarantee syscall filtering at the kernel boundary.`,
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
                    <pre className="text-xs text-text-soft leading-relaxed whitespace-pre-wrap font-sans mt-4">
                      {dive.content}
                    </pre>
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
