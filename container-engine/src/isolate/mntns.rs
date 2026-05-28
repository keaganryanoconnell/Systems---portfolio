use nix::mount::MsFlags;

/// MNT namespace isolation flag.
pub const MNT_FLAGS: MsFlags = MsFlags::MS_SLAVE;

/// After creating the mount namespace, make the root mount propagation
/// private to prevent mount events from leaking to/from the host.
pub fn make_root_private() -> nix::Result<()> {
    nix::mount::mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
}
