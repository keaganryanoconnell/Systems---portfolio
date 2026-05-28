//! Consensus Module - Cluster Membership Discovery
//!
//! Provides gossip membership discovery services using the SWIM protocol
//! to track live nodes in the cluster.

pub mod swim;

pub use swim::{PeerState, SwimNode};
