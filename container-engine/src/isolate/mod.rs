pub mod ipc;
pub mod mntns;
pub mod netns;
pub mod pid;
pub mod uts;

use nix::sched::{clone, CloneFlags};
use nix::unistd::Pid;
use std::ptr::null_mut;

use crate::container::config::ContainerConfig;
use crate::container::state::ContainerState;
use crate::util::errors::{ContainerError, ContainerResult};

const STACK_SIZE: usize = 2 * 1024 * 1024; // 2 MB child stack

/// Holds the PIDs resulting from the two-stage fork.
pub struct IsolatedProcess {
    /// PID 1 init process in the new namespace tree.
    pub init_pid: Pid,
    /// The user's command PID (child of init).
    pub child_pid: Pid,
}

/// Entry point: create all requested namespaces via clone(), then run
/// the two-stage init in the child.
pub fn isolate(config: &ContainerConfig) -> ContainerResult<IsolatedProcess> {
    // Build the namespace flags
    let mut flags = CloneFlags::empty();
    flags.set(CloneFlags::CLONE_NEWPID, true);
    flags.set(CloneFlags::CLONE_NEWNS, true);
    flags.set(CloneFlags::CLONE_NEWUTS, true);
    flags.set(CloneFlags::CLONE_NEWIPC, true);
    // NEWNET is set only when network_mode is not Host
    let netns = matches!(
        config.network_mode,
        crate::container::config::NetworkMode::Bridge
    );
    flags.set(CloneFlags::CLONE_NEWNET, netns);

    // Allocate a stack for the child
    let mut stack = vec![0u8; STACK_SIZE];

    // Clone config onto the heap so the pointer survives the clone() boundary.
    // clone() returns the child PID in the parent immediately; the parent's stack
    // frame could be reused before the child reads the pointer. Boxing puts the
    // config on the heap with a stable address.
    let config_box = Box::new(config.clone());
    let cfg_ptr = Box::into_raw(config_box) as *const ContainerConfig;

    let child_pid = clone(
        Box::new(move || child_entry(cfg_ptr)),
        &mut stack,
        flags,
        None,
    )
    .map_err(|e| ContainerError::NamespaceError {
        source: e,
        context: "clone(CLONE_NEW*) failed".into(),
    })?;

    tracing::info!(pid = %child_pid, "Container init process created");

    Ok(IsolatedProcess {
        init_pid: child_pid,
        child_pid: child_pid,
    })
}

/// The child entry point runs inside the new namespace but BEFORE
/// filesystem isolation, cgroups, and security setup (which happen
/// in the second stage inside PID 1 init).
fn child_entry(config_ptr: *const ContainerConfig) -> isize {
    // Safety: config_ptr was heap-allocated via Box::into_raw in isolate().
    // The pointer is valid for the lifetime of the container process.
    // We reconstruct the Box to ensure proper deallocation when it goes out of scope.
    let config = unsafe { Box::from_raw(config_ptr as *mut ContainerConfig) };

    // --- Stage 1: Inside new namespaces ---
    // Now we are PID 1 in the new PID namespace.
    // From here, we orchestrate the full container startup.

    // 1. Set hostname if requested (CLONE_NEWUTS is active)
    if let Some(ref hostname) = config.hostname {
        if let Err(e) = nix::unistd::sethostname(hostname) {
            tracing::error!(error = %e, "sethostname failed");
            return -1;
        }
    }

    // 2. Mount filesystem isolation
    if let Err(e) = crate::filesystem::isolate_root(config) {
        tracing::error!(error = %e, "Filesystem isolation failed");
        return -1;
    }

    // 3. Mount virtual filesystems inside the new root
    if let Err(e) = crate::filesystem::mount_virtual_fs(config) {
        tracing::error!(error = %e, "Virtual filesystem mount failed");
        return -1;
    }

    // 4. Apply security primitives: no_new_privs, capabilities, seccomp
    if let Err(e) = crate::security::apply_security(config) {
        tracing::error!(error = %e, "Security setup failed");
        return -1;
    }

    // 5. Write /etc/hosts, /etc/hostname, /etc/resolv.conf
    if let Err(e) = crate::network::write_etc_files(config) {
        tracing::error!(error = %e, "Failed to write /etc files");
        return -1;
    }

    // 6. Set up loopback interface inside container netns
    if let Err(e) = crate::network::setup_loopback() {
        tracing::warn!(error = %e, "Loopback setup failed (non-fatal)");
    }

    // 7. Start the PID 1 init loop or exec directly
    let cmd = if config.command.is_empty() {
        vec!["/bin/sh".to_string()]
    } else {
        config.command.clone()
    };

    if config.init_process {
        // Two-stage: PID 1 init will fork+exec the user command
        crate::process::init::run_init_loop(&cmd, &config.env, config.working_dir.as_deref())
    } else {
        // Direct exec — no PID 1 reaping (simpler but less correct)
        let cmd_str: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        match nix::unistd::execvp(&cmd_str[0], &cmd_str) {
            Ok(_) => 0,
            Err(e) => {
                tracing::error!(error = %e, command = %cmd_str[0], "execvp failed");
                -1
            }
        }
    }
}
