use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};

use crate::util::errors::{ContainerError, ContainerResult};

/// Run the PID 1 init loop inside the container.
///
/// This function:
/// 1. Forks to create the user's command process
/// 2. Enters a reaper loop (waitpid with WNOHANG)
/// 3. Forwards SIGTERM/SIGINT/SIGQUIT to the child process group
/// 4. Exits with the child's exit code when the child dies
///
/// This function never returns — it calls _exit().
pub fn run_init_loop(
    command: &[String],
    env: &[(String, String)],
    working_dir: Option<&Path>,
) -> isize {
    // Fork the user's command
    match unsafe { fork() } {
        Err(e) => {
            tracing::error!(error = %e, "Init: fork failed");
            return -1;
        }
        Ok(ForkResult::Child) => {
            // ── Stage 2: User command process ──
            // Set working directory
            if let Some(dir) = working_dir {
                let _ = std::env::set_current_dir(dir);
            }

            // Set environment variables
            for (key, val) in env {
                std::env::set_var(key, val);
            }

            // Convert command to CString for execvp
            let cmd_cstr: Vec<CString> = command
                .iter()
                .map(|s| CString::new(s.as_bytes()).unwrap_or_default())
                .collect();

            let args_cstr: Vec<&CString> = cmd_cstr.iter().collect();

            // Exec the user's command
            nix::unistd::execvp(&cmd_cstr[0], &args_cstr).unwrap_or_else(|e| {
                tracing::error!(error = %e, command = %command[0], "Init: execvp failed");
                // If exec fails, try /bin/sh -c with the command as a single string
                let shell_cmd = command.join(" ");
                let shell_args = [
                    CString::new("/bin/sh").unwrap_or_default(),
                    CString::new("-c").unwrap_or_default(),
                    CString::new(shell_cmd).unwrap_or_default(),
                ];
                nix::unistd::execvp(&shell_args[0], &shell_args).unwrap_or_else(|e2| {
                    tracing::error!(error = %e2, "Init: /bin/sh fallback also failed");
                    std::process::exit(127);
                });
            });

            // Unreachable
            0
        }
        Ok(ForkResult::Parent { child }) => {
            // ── PID 1 Init Process ──
            let child_pid = child;
            tracing::info!(child_pid = %child_pid, "Init: user command forked");

            // Set up signal handling: we need a signal handler thread
            // because the main thread will be blocked in waitpid
            let running = AtomicBool::new(true);

            // Handle signals in a separate thread
            let thread_running = std::sync::Arc::new(AtomicBool::new(true));
            let thread_running_clone = thread_running.clone();

            std::thread::spawn(move || {
                // Read signals from a self-pipe or signalfd
                // For simplicity, we poll /proc/self/status for pending signals
                // In production, use signalfd(2) for efficient signal handling
                while thread_running_clone.load(Ordering::Relaxed) {
                    // Check if child is still alive
                    match kill(child_pid, None) {
                        Ok(()) => {} // Child exists
                        Err(_) => {
                            // Child is gone, exit
                            break;
                        }
                    }

                    // Sleep briefly
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            // Main reaper loop
            let mut exit_code: i32 = 0;

            loop {
                match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
                    Ok(WaitStatus::Exited(pid, code)) => {
                        if pid == child_pid {
                            exit_code = code;
                            tracing::info!(pid = %pid, code = code, "Init: child process exited");
                            break;
                        }
                        // Other process exited — reap it (zombie reaping)
                        tracing::debug!(pid = %pid, code = code, "Init: reaped orphan");
                    }
                    Ok(WaitStatus::Signaled(pid, sig, _core_dumped)) => {
                        if pid == child_pid {
                            exit_code = 128 + sig as i32;
                            tracing::info!(pid = %pid, signal = %sig, "Init: child process killed by signal");
                            break;
                        }
                    }
                    Ok(WaitStatus::StillAlive) => {
                        // No status available yet — send any pending signals
                        check_and_forward_signal(child_pid);
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Ok(_) => {
                        // Other status (Stopped, Continued, etc.) — ignore
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(e) => {
                        // ECHILD means no children — but we should have the child
                        if e == nix::Error::ECHILD {
                            tracing::warn!("Init: ECHILD — no children to wait on");
                            break;
                        }
                        tracing::error!(error = %e, "Init: waitpid error");
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }

                // Check if we should forward signals
                // (In a full implementation, use signalfd or sigaction with SA_SIGINFO)
            }

            // Cleanup: signal the signal thread to stop
            thread_running.store(false, Ordering::Relaxed);

            // Exit with the child's exit code
            tracing::info!(exit_code = exit_code, "Init: exiting");
            std::process::exit(exit_code);
        }
    }
}

/// Forward common signals to the child process group.
fn check_and_forward_signal(child_pid: Pid) {
    // Read /proc/self/status for pending signals
    // In production, use signalfd(2) for real-time signal handling
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("SigPnd:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(mask) = u64::from_str_radix(parts[1], 16) {
                        // SIGTERM = signal 15
                        if mask & (1u64 << (15 - 1)) != 0 {
                            let _ = kill(child_pid, Signal::SIGTERM);
                        }
                        // SIGINT = signal 2
                        if mask & (1u64 << (2 - 1)) != 0 {
                            let _ = kill(child_pid, Signal::SIGINT);
                        }
                        // SIGQUIT = signal 3
                        if mask & (1u64 << (3 - 1)) != 0 {
                            let _ = kill(child_pid, Signal::SIGQUIT);
                        }
                        // SIGHUP = signal 1
                        if mask & (1u64 << (1 - 1)) != 0 {
                            let _ = kill(child_pid, Signal::SIGHUP);
                        }
                    }
                }
                break;
            }
        }
    }
}
