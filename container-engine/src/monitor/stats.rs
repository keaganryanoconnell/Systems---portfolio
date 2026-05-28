use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::util::errors::{ContainerError, ContainerResult};
use crate::util::id::ContainerId;

/// Container resource usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub id: ContainerId,
    pub cpu_usage_us: u64,
    pub cpu_nr_periods: u64,
    pub cpu_nr_throttled: u64,
    pub cpu_throttled_us: u64,
    pub memory_usage_bytes: u64,
    pub memory_max_bytes: u64,
    pub memory_swap_bytes: u64,
    pub memory_oom_count: u64,
    pub pids_current: u32,
    pub pids_limit: u32,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub io_read_ops: u64,
    pub io_write_ops: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
}

/// Read container stats from cgroup filesystem.
pub fn read_cgroup_stats(id: &ContainerId) -> ContainerResult<ContainerStats> {
    let cgroup_path = Path::new("/sys/fs/cgroup").join(format!("container-{}", id.short_id()));

    let cpu_stat = read_key_value(&cgroup_path.join("cpu.stat"))?;
    let memory_current = read_file(&cgroup_path.join("memory.current"))?
        .trim()
        .parse()
        .unwrap_or(0);
    let memory_max = read_file(&cgroup_path.join("memory.max"))?
        .trim()
        .parse()
        .unwrap_or(0);
    let memory_swap = read_file(&cgroup_path.join("memory.swap.current"))?
        .trim()
        .parse()
        .unwrap_or(0);
    let memory_oom = read_file(&cgroup_path.join("memory.events"))?;
    let pids_current = read_file(&cgroup_path.join("pids.current"))?
        .trim()
        .parse()
        .unwrap_or(0);
    let pids_max = read_file(&cgroup_path.join("pids.max"))?
        .trim()
        .parse()
        .unwrap_or(0);

    let io_stat = read_key_value(&cgroup_path.join("io.stat"))?;

    Ok(ContainerStats {
        id: id.clone(),
        cpu_usage_us: cpu_stat.get("usage_usec").copied().unwrap_or(0),
        cpu_nr_periods: cpu_stat.get("nr_periods").copied().unwrap_or(0),
        cpu_nr_throttled: cpu_stat.get("nr_throttled").copied().unwrap_or(0),
        cpu_throttled_us: cpu_stat.get("throttled_usec").copied().unwrap_or(0),
        memory_usage_bytes: memory_current,
        memory_max_bytes: memory_max,
        memory_swap_bytes: memory_swap,
        memory_oom_count: memory_oom.lines().filter(|l| l.starts_with("oom")).count() as u64,
        pids_current: pids_current as u32,
        pids_limit: pids_max as u32,
        io_read_bytes: io_stat.get("rbytes").copied().unwrap_or(0),
        io_write_bytes: io_stat.get("wbytes").copied().unwrap_or(0),
        io_read_ops: io_stat.get("rios").copied().unwrap_or(0),
        io_write_ops: io_stat.get("wios").copied().unwrap_or(0),
        net_rx_bytes: 0, // Requires net class cgroup or /sys/class/net
        net_tx_bytes: 0,
    })
}

fn read_file(path: &Path) -> ContainerResult<String> {
    std::fs::read_to_string(path).map_err(|e| ContainerError::Io(e))
}

fn read_key_value(path: &Path) -> ContainerResult<HashMap<String, u64>> {
    let content = read_file(path)?;
    let mut map = HashMap::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(val) = parts[1].parse::<u64>() {
                map.insert(parts[0].to_string(), val);
            }
        }
    }
    Ok(map)
}
