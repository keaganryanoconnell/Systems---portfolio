use nix::sched::CloneFlags;

/// NET namespace isolation. The actual namespace creation is done
/// via CLONE_NEWNET in isolate::mod.rs. This module handles the
/// network configuration AFTER the namespace is created.

/// Returns the CLONE_NEWNET flag if network isolation is requested.
pub fn clone_flag() -> CloneFlags {
    CloneFlags::CLONE_NEWNET
}
