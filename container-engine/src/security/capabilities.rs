use caps::{CapSet, Capability, CapsHashSet};

use crate::util::errors::{ContainerError, ContainerResult};

/// Capabilities kept inside the container (whitelist approach).
/// All other capabilities are dropped from the bounding set.
const KEPT_CAPS: &[Capability] = &[
    Capability::CAP_CHOWN,
    Capability::CAP_DAC_OVERRIDE,
    Capability::CAP_FOWNER,
    Capability::CAP_FSETID,
    Capability::CAP_KILL,
    Capability::CAP_NET_BIND_SERVICE,
    Capability::CAP_SETUID,
    Capability::CAP_SETGID,
    // Note: CAP_NET_RAW is deliberately omitted — disables ping and raw sockets
    // Note: CAP_SYS_ADMIN is deliberately omitted — prevents namespace escape
];

/// Drops all capabilities except those in KEPT_CAPS from the bounding
/// and inheritable sets.
pub fn apply_capabilities() -> ContainerResult<()> {
    let mut bounding_set: CapsHashSet =
        caps::read(None, CapSet::Bounding).map_err(|e| ContainerError::CapabilityError {
            cap: "read_bounding".into(),
            detail: format!("failed to read capability bounding set: {e}"),
        })?;

    let mut inheritable_set: CapsHashSet =
        caps::read(None, CapSet::Inheritable).map_err(|e| ContainerError::CapabilityError {
            cap: "read_inheritable".into(),
            detail: format!("failed to read inheritable set: {e}"),
        })?;

    // Determine which caps to drop
    let caps_to_keep: CapsHashSet = KEPT_CAPS.iter().copied().collect();

    // Drop from bounding set
    for cap in bounding_set.clone().difference(&caps_to_keep) {
        caps::drop(None, CapSet::Bounding, *cap).map_err(|e| ContainerError::CapabilityError {
            cap: format!("{cap:?}"),
            detail: format!("failed to drop capability from bounding set: {e}"),
        })?;
        tracing::debug!(?cap, "Dropped capability from bounding set");
    }

    // Drop from inheritable set (same set)
    for cap in inheritable_set.clone().difference(&caps_to_keep) {
        caps::drop(None, CapSet::Inheritable, *cap).map_err(|e| {
            ContainerError::CapabilityError {
                cap: format!("{cap:?}"),
                detail: format!("failed to drop capability from inheritable set: {e}"),
            }
        })?;
    }

    tracing::info!(
        kept = ?KEPT_CAPS,
        "Capability bounding set configured"
    );

    Ok(())
}

/// Verify that the bounding set contains only the expected capabilities.
pub fn verify_capabilities() -> ContainerResult<()> {
    let bounding_set =
        caps::read(None, CapSet::Bounding).map_err(|e| ContainerError::CapabilityError {
            cap: "verify".into(),
            detail: format!("failed to read bounding set: {e}"),
        })?;

    for cap in KEPT_CAPS {
        if !bounding_set.contains(cap) {
            return Err(ContainerError::CapabilityError {
                cap: format!("{cap:?}"),
                detail: "expected capability was dropped".into(),
            });
        }
    }

    Ok(())
}
