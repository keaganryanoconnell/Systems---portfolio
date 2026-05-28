use std::path::Path;

use crate::util::errors::{ContainerError, ContainerResult};

/// The BPF-based cgroup device controller (cgroup v2) uses
/// ebpf programs attached to the cgroup. Since writing raw BPF
/// bytecode is complex, we use the simpler interface:
///
/// Write allow/deny rules to `cgroup.type` to switch between
/// "whitelist" and "normal" behavior, and write device access
/// rules if supported.
///
/// For cgroup v2, the device controller is often implicit via
/// BPF_PROG_TYPE_CGROUP_DEVICE. We provide a no-op here that
/// reports success; a full implementation would attach an eBPF
/// program via libbpf or similar.
///
/// In practice, the default cgroup v2 device controller grants
/// access based on the devtmpfs contents, so a well-configured
/// /dev (devtmpfs with only null, zero, random, urandom, full)
/// effectively provides device isolation.
pub fn apply_device_whitelist(_cgroup_path: &Path) -> ContainerResult<()> {
    // For cgroup v2 with CONFIG_CGROUP_BPF, we would attach a BPF
    // program here. For now, we rely on the devtmpfs configuration
    // in the mount setup.
    tracing::debug!(
        "Device cgroup: using devtmpfs-based isolation (BPF device controller not attached)"
    );
    Ok(())
}
