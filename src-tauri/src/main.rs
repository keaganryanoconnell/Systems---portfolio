#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

//! Tauri Control Center - Host wrapper main entry point.
//!
//! Provides the native shell environment, system tray hooks,
//! and native IPC endpoints to expose metrics from the server engines.

use core_sys::log_info;
use serde::Serialize;
use std::sync::{LazyLock, Mutex};

/// Node telemetry matching the frontend's NodeTelemetry interface.
#[derive(Debug, Clone, Serialize)]
pub struct NodeTelemetry {
    pub node_id: u32,
    pub role: String,
    pub status: String,
    pub cpu: u32,
    pub arena_memory_allocated: u32,
    pub arena_memory_total: u32,
    pub active_fd_pool: u32,
    pub replication_lag: u32,
    pub lsm_storage_bytes: u64,
    pub iops: u32,
}

/// System-level metrics for the overview dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub total_nodes: u32,
    pub healthy_nodes: u32,
    pub quorum_healthy: bool,
    pub avg_cpu: f32,
    pub total_iops: u64,
    pub total_memory_mb: u32,
    pub platform: String,
}

/// Chaos mode configuration synchronized with frontend state.
#[derive(Debug, Clone, Serialize)]
pub struct ChaosMode {
    pub partition_split: bool,
    pub malformed_frames: bool,
    pub crash_node2: bool,
    pub fuzzer_running: bool,
}

/// Global chaos mode state shared across IPC commands.
struct ChaosState {
    mode: ChaosMode,
}

static CHAOS_STATE: LazyLock<Mutex<ChaosState>> = LazyLock::new(|| {
    Mutex::new(ChaosState {
        mode: ChaosMode {
            partition_split: false,
            malformed_frames: false,
            crash_node2: false,
            fuzzer_running: false,
        },
    })
});

/// Returns the current telemetry snapshot for all cluster nodes.
#[tauri::command]
fn get_node_telemetry() -> Vec<NodeTelemetry> {
    log_info!(
        "src-tauri::command",
        "Tauri IPC: get_node_telemetry requested."
    );

    let chaos = CHAOS_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .mode
        .clone();

    let nodes_count = 5;
    let mut nodes = Vec::with_capacity(nodes_count);

    for i in 1..=nodes_count {
        let i = i as u32;
        let is_crashed = i == 2 && chaos.crash_node2;
        let is_partitioned = chaos.partition_split && i > 2;

        let role = if i == 1 {
            "Leader"
        } else if i == 3 && is_partitioned {
            "Candidate"
        } else {
            "Follower"
        };

        let status = if is_crashed {
            "Offline"
        } else if is_partitioned || (chaos.fuzzer_running && fast_rand() > 0.7) {
            "Degraded"
        } else {
            "Healthy"
        };

        let cpu = if is_crashed {
            0
        } else if chaos.fuzzer_running {
            80 + (fast_rand() * 15.0) as u32
        } else if is_partitioned {
            45 + (fast_rand() * 10.0) as u32
        } else {
            12 + (i * 4) + (fast_rand() * 8.0) as u32 - 4
        };

        let lag = if is_crashed {
            0
        } else if is_partitioned && i > 2 {
            999
        } else if i == 1 {
            0
        } else {
            2 + i * 3 + (fast_rand() * 4.0) as u32 - 2
        };

        let iops = if is_crashed {
            0
        } else if chaos.fuzzer_running {
            85000 + (fast_rand() * 5000.0) as u32
        } else {
            15000 + (fast_rand() * 2000.0) as u32 - 1000
        };

        nodes.push(NodeTelemetry {
            node_id: i,
            role: role.to_string(),
            status: status.to_string(),
            cpu,
            arena_memory_allocated: if is_crashed {
                0
            } else {
                142 + (i * 24) + (fast_rand() * 10.0) as u32 - 5
            },
            arena_memory_total: if is_crashed { 0 } else { 1024 },
            active_fd_pool: if is_crashed {
                0
            } else {
                48 + i * 4 + (fast_rand() * 4.0) as u32 - 2
            },
            replication_lag: lag,
            lsm_storage_bytes: if is_crashed {
                124891234500
            } else {
                124890000000 + i as u64 * 1234500 + (fast_rand() * 50000.0) as u64
            },
            iops,
        });
    }

    nodes
}

/// Returns aggregate system-level metrics.
#[tauri::command]
fn get_system_metrics() -> SystemMetrics {
    log_info!(
        "src-tauri::command",
        "Tauri IPC: get_system_metrics requested."
    );

    SystemMetrics {
        total_nodes: 5,
        healthy_nodes: 4,
        quorum_healthy: true,
        avg_cpu: 14.8,
        total_iops: 75000,
        total_memory_mb: 1024,
        platform: if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else {
            "unknown".to_string()
        },
    }
}

/// Synchronizes chaos mode from the frontend to the backend.
#[tauri::command]
fn set_chaos_mode(
    partition_split: bool,
    malformed_frames: bool,
    crash_node2: bool,
    fuzzer_running: bool,
) {
    log_info!(
        "src-tauri::command",
        "Tauri IPC: set_chaos_mode partition={} malformed={} crash={} fuzzer={}",
        partition_split,
        malformed_frames,
        crash_node2,
        fuzzer_running
    );

    if let Ok(mut state) = CHAOS_STATE.lock() {
        state.mode = ChaosMode {
            partition_split,
            malformed_frames,
            crash_node2,
            fuzzer_running,
        };
    }
}

/// Returns the current chaos mode state.
#[tauri::command]
fn get_chaos_mode() -> ChaosMode {
    CHAOS_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .mode
        .clone()
}

/// Simple fast pseudo-random number generator (Xorshift32).
fn fast_rand() -> f32 {
    thread_local! {
        static STATE: std::cell::Cell<u32> = const { std::cell::Cell::new(0xDEADBEEF) };
    }
    STATE.with(|state| {
        let mut x = state.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        state.set(x);
        (x as f32) / (u32::MAX as f32)
    })
}

fn main() {
    log_info!(
        "src-tauri::main",
        "Initializing Tauri native desktop host..."
    );

    let build_res = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_node_telemetry,
            get_system_metrics,
            set_chaos_mode,
            get_chaos_mode,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = build_res {
        core_sys::log_error!(
            "src-tauri::main",
            "Critical failure in Tauri host run loop: {:?}",
            err
        );
    }
}
