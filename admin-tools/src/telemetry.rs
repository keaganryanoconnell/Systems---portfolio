//! Telemetry Snapshot Parser
//!
//! Parses the JSON payload returned by the platform-nodes HTTP proxy
//! without any external JSON parsing library.

/// A point-in-time snapshot of platform-nodes telemetry metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetrySnapshot {
    /// Operational status string (e.g. "ACTIVE").
    pub status: String,
    /// Number of live peers tracked by the SWIM gossip consensus layer.
    pub swim_peers: u64,
    /// Number of SSTable files currently tracked by the LSM storage engine.
    pub lsm_sstables: u64,
}

/// Extracts the value of a JSON string field from a raw JSON string.
/// e.g. `extract_str(r#"{"status":"ACTIVE"}"#, "status")` → `Some("ACTIVE")`
fn extract_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\"", key);
    let key_pos = json.find(needle.as_str())?;
    let after_key = &json[key_pos + needle.len()..];
    let colon = after_key.find(':')? + 1;
    let after_colon = after_key[colon..].trim_start();

    if let Some(inner) = after_colon.strip_prefix('"') {
        let end = inner.find('"')?;
        Some(&inner[..end])
    } else {
        None
    }
}

/// Extracts the value of a JSON numeric field from a raw JSON string.
/// e.g. `extract_u64(r#"{"swim_peers":3}"#, "swim_peers")` → `Some(3)`
fn extract_u64(json: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{}\"", key);
    let key_pos = json.find(needle.as_str())?;
    let after_key = &json[key_pos + needle.len()..];
    let colon = after_key.find(':')? + 1;
    let value_str = after_key[colon..].trim_start();

    // Read digits until a non-digit character
    let digits: String = value_str
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

impl TelemetrySnapshot {
    /// Parses a raw JSON string into a `TelemetrySnapshot`.
    /// Returns `None` if required fields are missing or malformed.
    pub fn parse(raw: &str) -> Option<Self> {
        let status = extract_str(raw, "status")?.to_string();
        let swim_peers = extract_u64(raw, "swim_peers").unwrap_or(0);
        let lsm_sstables = extract_u64(raw, "lsm_sstables").unwrap_or(0);

        Some(Self {
            status,
            swim_peers,
            lsm_sstables,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_snapshot() {
        let raw = r#"
{
  "status": "ACTIVE",
  "swim_peers": 5,
  "lsm_sstables": 2
}"#;
        let snap = TelemetrySnapshot::parse(raw).unwrap();
        assert_eq!(snap.status, "ACTIVE");
        assert_eq!(snap.swim_peers, 5);
        assert_eq!(snap.lsm_sstables, 2);
    }

    #[test]
    fn test_parse_zero_values() {
        let raw = r#"{"status": "ACTIVE", "swim_peers": 0, "lsm_sstables": 0}"#;
        let snap = TelemetrySnapshot::parse(raw).unwrap();
        assert_eq!(snap.swim_peers, 0);
        assert_eq!(snap.lsm_sstables, 0);
    }

    #[test]
    fn test_parse_missing_status_returns_none() {
        let raw = r#"{"swim_peers": 1, "lsm_sstables": 0}"#;
        assert!(TelemetrySnapshot::parse(raw).is_none());
    }

    #[test]
    fn test_parse_missing_numeric_defaults_to_zero() {
        let raw = r#"{"status": "ACTIVE"}"#;
        let snap = TelemetrySnapshot::parse(raw).unwrap();
        assert_eq!(snap.swim_peers, 0);
        assert_eq!(snap.lsm_sstables, 0);
    }
}
