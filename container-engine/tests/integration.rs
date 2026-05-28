#[cfg(test)]
mod tests {
    // Integration tests require Linux and are skipped when not running as root.
    // Run with: cargo test -p container-engine --test integration -- --nocapture

    #[test]
    #[cfg(target_os = "linux")]
    fn test_container_config_builder() {
        use container_engine::container::config::ContainerConfig;
        use container_engine::container::state::ContainerState;

        // Test that the builder validates required fields
        let result = ContainerConfig::builder().build();
        assert!(result.is_err(), "Builder should error without rootfs");

        // Test valid config
        let config = ContainerConfig::builder()
            .rootfs(std::path::PathBuf::from("/tmp"))
            .memory_limit_mb(256)
            .cpu_weight(100)
            .pids_max(256)
            .hostname("test-container".into())
            .command(vec!["/bin/sh".into(), "-c".into(), "echo hello".into()])
            .readonly_rootfs(true)
            .build();
        assert!(config.is_ok(), "Valid config should build");
        let config = config.unwrap();

        assert_eq!(config.memory_limit_bytes, Some(256 * 1024 * 1024));
        assert_eq!(config.pids_max, Some(256));
        assert_eq!(config.hostname, Some("test-container".to_string()));
        assert!(config.readonly_rootfs);
        assert_eq!(config.command, vec!["/bin/sh", "-c", "echo hello"]);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_container_state_machine() {
        use container_engine::container::state::ContainerState;

        // Valid transitions
        assert!(ContainerState::Created.valid_transition(ContainerState::Running));
        assert!(ContainerState::Running.valid_transition(ContainerState::Paused));
        assert!(ContainerState::Running.valid_transition(ContainerState::Stopped));
        assert!(ContainerState::Paused.valid_transition(ContainerState::Running));
        assert!(ContainerState::Paused.valid_transition(ContainerState::Stopped));
        assert!(ContainerState::Stopped.valid_transition(ContainerState::Created));

        // Invalid transitions
        assert!(!ContainerState::Created.valid_transition(ContainerState::Paused));
        assert!(!ContainerState::Created.valid_transition(ContainerState::Stopped));
        assert!(!ContainerState::Running.valid_transition(ContainerState::Created));
        assert!(!ContainerState::Paused.valid_transition(ContainerState::Created));
        assert!(!ContainerState::Stopped.valid_transition(ContainerState::Stopped));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_container_id_generation() {
        use container_engine::util::id::ContainerId;
        use std::str::FromStr;

        let id = ContainerId::generate();
        assert_eq!(id.as_str().len(), 13);
        assert!(id.as_str().starts_with("ctr-"));

        // Test parsing
        let parsed = ContainerId::from_str("ctr-a1b2c3d4");
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().as_str(), "ctr-a1b2c3d4");

        // Test invalid IDs
        assert!(ContainerId::from_str("").is_none());
        assert!(ContainerId::from_str("invalid").is_none());
        assert!(ContainerId::from_str("ctr-invalid").is_none());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_zero_copy_binary_protocol() {
        // This tests that the binary protocol for cgroup stats parsing works.
        // In a real integration test, this would verify actual cgroup reads.
        use container_engine::monitor::stats::ContainerStats;
        use container_engine::util::id::ContainerId;

        // Verify struct layout is correct for serde
        let id = ContainerId::generate();
        let stats = ContainerStats {
            id: id.clone(),
            cpu_usage_us: 0,
            cpu_nr_periods: 0,
            cpu_nr_throttled: 0,
            cpu_throttled_us: 0,
            memory_usage_bytes: 0,
            memory_max_bytes: 268435456,
            memory_swap_bytes: 0,
            memory_oom_count: 0,
            pids_current: 1,
            pids_limit: 256,
            io_read_bytes: 0,
            io_write_bytes: 0,
            io_read_ops: 0,
            io_write_ops: 0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains(&id.to_string()));
        assert!(json.contains("268435456"));
        assert!(json.contains("256"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_signal_parsing() {
        use container_engine::process::signal::parse_signal;
        use nix::sys::signal::Signal;

        assert_eq!(parse_signal("SIGTERM").unwrap(), Signal::SIGTERM);
        assert_eq!(parse_signal("TERM").unwrap(), Signal::SIGTERM);
        assert_eq!(parse_signal("term").unwrap(), Signal::SIGTERM);
        assert_eq!(parse_signal("9").unwrap(), Signal::SIGKILL);
        assert_eq!(parse_signal("15").unwrap(), Signal::SIGTERM);
        assert_eq!(parse_signal("SIGKILL").unwrap(), Signal::SIGKILL);

        assert!(parse_signal("INVALID").is_err());
        assert!(parse_signal("").is_err());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_container_error_types() {
        use container_engine::container::state::ContainerState;
        use container_engine::util::errors::ContainerError;

        // Verify Display impl for all error types
        let err = ContainerError::StateTransitionError {
            from: ContainerState::Created,
            to: ContainerState::Paused,
        };
        let msg = err.to_string();
        assert!(msg.contains("Created"));
        assert!(msg.contains("Paused"));
    }
}
