/// UTS namespace isolation. Hostname and domain name are set here.
/// The actual namespace creation is done via CLONE_NEWUTS.
use crate::util::errors::{ContainerError, ContainerResult};

pub fn set_hostname(hostname: &str) -> ContainerResult<()> {
    nix::unistd::sethostname(hostname).map_err(|e| ContainerError::NamespaceError {
        source: e,
        context: format!("sethostname({hostname}) failed"),
    })
}
