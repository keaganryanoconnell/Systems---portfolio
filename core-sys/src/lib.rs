//! Core Systems Utility Library
//!
//! Provides zero-allocation telemetry structures and lock-free logging primitives.
//! Optimized for real-time telemetry ingestion in the systems engineering dashboard.

pub mod logger;
pub mod spsc;

pub use logger::{init_telemetry_daemon, log_raw, stop_telemetry_daemon};

/// Helper macro for logging info level events without allocations.
#[macro_export]
macro_rules! log_info {
    ($target:expr, $($arg:tt)+) => {
        $crate::log_raw("INFO", $target, format_args!($($arg)+));
    };
}

/// Helper macro for logging error level events without allocations.
#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)+) => {
        $crate::log_raw("ERROR", $target, format_args!($($arg)+));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_free_logging() {
        // Initialize background thread logger daemon
        let inited = init_telemetry_daemon();
        assert!(inited, "Daemon should initialize successfully.");

        // Log formatted messages without heap allocations
        log_info!("core-sys::tests", "Starting SPSC queue integration tests");
        log_info!("core-sys::tests", "Telemetry code check: {}", 0xABC);
        log_error!("core-sys::tests", "Zero-allocation check: failed to fail");

        // Allow background thread to process and print to stdout
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Stop daemon cleanly
        stop_telemetry_daemon();
    }
}
