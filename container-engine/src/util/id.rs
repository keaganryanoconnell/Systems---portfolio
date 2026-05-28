use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

const ID_PREFIX: &str = "ctr";
const ID_CHARS: &[u8] = b"abcdef0123456789";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(String);

impl ContainerId {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let suffix: String = (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..ID_CHARS.len());
                ID_CHARS[idx] as char
            })
            .collect();
        ContainerId(format!("{ID_PREFIX}-{suffix}"))
    }

    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() == 13 && s.starts_with("ctr-") && s[4..].chars().all(|c| c.is_ascii_hexdigit()) {
            Some(ContainerId(s.to_string()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short_id(&self) -> &str {
        &self.0[4..]
    }
}

impl fmt::Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ContainerId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ContainerId::from_str(s).ok_or_else(|| format!("invalid container ID: {s}"))
    }
}
