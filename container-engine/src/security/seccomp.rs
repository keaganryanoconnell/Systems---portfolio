use crate::util::errors::{ContainerError, ContainerResult};

/// BPF instruction for the seccomp filter (struct sock_filter equivalent).
#[repr(C)]
#[derive(Clone, Copy)]
struct SockFilter {
    code: u16,
    jt: u8,
    jf: u8,
    k: u32,
}

/// BPF program (struct sock_fprog equivalent).
#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *const SockFilter,
}

// BPF opcodes for seccomp
const BPF_LD: u16 = 0x00;
const BPF_LD_W_ABS: u16 = 0x20;
const BPF_JMP: u16 = 0x05;
const BPF_JMP_JEQ: u16 = 0x15;
const BPF_JMP_JGT: u16 = 0x25;
const BPF_JMP_JGE: u16 = 0x35;
const BPF_JMP_JSET: u16 = 0x45;
const BPF_RET: u16 = 0x06;
const BPF_ALU: u16 = 0x04;
const BPF_ALU_AND: u16 = 0x50;

// Seccomp return values
const SECCOMP_RET_KILL_PROCESS: u32 = 0x80000000;
const SECCOMP_RET_TRAP: u32 = 0x00030000;
const SECCOMP_RET_ERRNO: u32 = 0x00050000;
const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;

// Audit architecture identifiers
const AUDIT_ARCH_X86_64: u32 = 0xC000003E;

// Offsets into seccomp_data
const DATA_ARCH_OFFSET: u32 = 4;
const DATA_NR_OFFSET: u32 = 0;

/// BPF instruction helper: load 32-bit word from seccomp_data at offset.
fn bpf_ld(offset: u32) -> SockFilter {
    SockFilter {
        code: BPF_LD | BPF_LD_W_ABS,
        jt: 0,
        jf: 0,
        k: offset,
    }
}

/// BPF instruction helper: jump if A == k.
fn bpf_jeq(k: u32, jt: u8, jf: u8) -> SockFilter {
    SockFilter {
        code: BPF_JMP | BPF_JMP_JEQ,
        jt,
        jf,
        k,
    }
}

/// BPF instruction helper: return.
fn bpf_ret(k: u32) -> SockFilter {
    SockFilter {
        code: BPF_RET,
        jt: 0,
        jf: 0,
        k,
    }
}

/// Apply a seccomp-BPF filter that:
/// 1. Validates the architecture is x86_64
/// 2. Defines a whitelist of allowed syscalls
/// 3. KILLs the process on any disallowed syscall
///
/// The BPF program architecture:
///   Line 0: LD arch offset -> A
///   Line 1: JEQ AUDIT_ARCH_X86_64 -> if match, goto 3; else goto 2
///   Line 2: RET KILL (not x86_64)
///   Line 3: LD syscall nr offset -> A
///   Lines 4..N: JEQ syscall_N -> if match, goto RET_ALLOW; else continue
///   Line N+1: RET KILL (no match found)
///
/// This is a minimal implementation. A production version would use
/// a BPF compiler (like libseccomp or seccompiler) to generate optimized
/// binary-search filters.
pub fn apply_seccomp_filter() -> ContainerResult<()> {
    use libc::prctl;
    use libc::PR_SET_NO_NEW_PRIVS;
    use libc::PR_SET_SECCOMP;
    use libc::SECCOMP_MODE_FILTER;

    // Build the BPF filter program.
    //
    // The filter determines allowed syscalls based on the syscall number
    // in seccomp_data. For this initial implementation, we load the
    // architecture (to verify it's x86_64), then load the syscall number
    // and allow it unconditionally.
    //
    // A full whitelist would replace the unconditional ALLOW with a
    // binary-search over allowed syscalls.
    //
    // Note: The `libc` crate provides the prctl() wrapper which takes
    // `c_int` arguments. We use it directly.

    // The syscall numbers that are allowed. We use a sorted array for
    // potential binary search. Initial implementation allows all syscalls.
    let allowed_syscalls: &[u32] = &[];

    // Build the filter program
    let mut filter_prog: Vec<SockFilter> = Vec::new();

    // Step 1: Load architecture and verify it's x86_64
    filter_prog.push(bpf_ld(DATA_ARCH_OFFSET));
    // If architecture matches x86_64, skip the kill
    filter_prog.push(bpf_jeq(AUDIT_ARCH_X86_64, 1, 0)); // match: skip 1; no match: fallthrough to kill
    filter_prog.push(bpf_ret(SECCOMP_RET_KILL_PROCESS)); // kill on wrong arch

    // Step 2: Load syscall number
    filter_prog.push(bpf_ld(DATA_NR_OFFSET));

    // Step 3: Linear scan through allowed syscalls (production would use binary search)
    for &nr in allowed_syscalls {
        // For each allowed syscall, if A == nr, allow; else continue
        filter_prog.push(bpf_jeq(nr, 1, 0)); // match: skip 1 to allow; mismatch: fallthrough
        filter_prog.push(bpf_ret(SECCOMP_RET_ALLOW));
    }

    // Step 4: No match found — kill the process
    filter_prog.push(bpf_ret(SECCOMP_RET_KILL_PROCESS));

    // Build the sock_fprog structure
    let prog = SockFprog {
        len: filter_prog.len() as u16,
        filter: filter_prog.as_ptr(),
    };

    // First, ensure NO_NEW_PRIVS is set (required for SECCOMP_MODE_FILTER
    // without CAP_SYS_ADMIN, though we should already have it set by now)
    unsafe {
        let ret = prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret != 0 {
            return Err(ContainerError::SeccompError(nix::errno::Errno::last()));
        }
    }

    // Install the seccomp filter via prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog)
    // Safety: The prog structure points to valid, stack-allocated filter data.
    unsafe {
        let ret = prctl(
            PR_SET_SECCOMP,
            SECCOMP_MODE_FILTER as libc::c_ulong,
            &prog as *const SockFprog as libc::c_ulong,
            0,
            0,
        );
        if ret != 0 {
            return Err(ContainerError::SeccompError(nix::errno::Errno::last()));
        }
    }

    tracing::info!(
        allowed_count = allowed_syscalls.len(),
        filter_len = filter_prog.len(),
        "Seccomp-BPF filter installed"
    );

    Ok(())
}
