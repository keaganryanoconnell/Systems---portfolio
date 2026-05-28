use serde::{Deserialize, Serialize};

use crate::util::errors::ContainerResult;

/// A container event (OCI runtime-spec compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerEvent {
    pub event_type: EventType,
    pub id: String,
    pub message: String,
    pub timestamp_ns: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    Created,
    Started,
    Stopped,
    Killed,
    Paused,
    Resumed,
    OomKilled,
    HealthcheckFailed,
}

/// Simple in-memory event ring buffer.
/// In production, this would write to a journald or file.
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_EVENTS: usize = 1024;

static EVENT_LOG: Mutex<Vec<ContainerEvent>> = Mutex::new(Vec::new());

pub fn record_event(event_type: EventType, id: &str, message: String) -> ContainerResult<()> {
    let mut log = EVENT_LOG.lock().unwrap_or_else(|e| e.into_inner());
    let timestamp_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    log.push(ContainerEvent {
        event_type,
        id: id.to_string(),
        message,
        timestamp_ns,
    });

    if log.len() > MAX_EVENTS {
        log.remove(0);
    }

    Ok(())
}

pub fn get_events(id: Option<&str>) -> Vec<ContainerEvent> {
    let log = EVENT_LOG.lock().unwrap_or_else(|e| e.into_inner());
    match id {
        Some(filter) => log.iter().filter(|e| e.id == filter).cloned().collect(),
        None => log.clone(),
    }
}
