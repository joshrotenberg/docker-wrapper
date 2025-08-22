//! Docker network management commands.
//!
//! This module provides commands for managing Docker networks:
//! - `network create` - Create a network
//! - `network ls` - List networks
//! - `network rm` - Remove networks
//! - `network inspect` - Display detailed network information
//! - `network connect` - Connect a container to a network
//! - `network disconnect` - Disconnect a container from a network
//! - `network prune` - Remove unused networks

pub mod connect;
pub mod create;
pub mod disconnect;
pub mod inspect;
pub mod ls;
pub mod prune;
pub mod rm;

pub use connect::{NetworkConnectCommand, NetworkConnectResult};
pub use create::{NetworkCreateCommand, NetworkCreateResult};
pub use disconnect::{NetworkDisconnectCommand, NetworkDisconnectResult};
pub use inspect::{NetworkInspectCommand, NetworkInspectOutput};
pub use ls::{NetworkInfo, NetworkLsCommand, NetworkLsOutput};
pub use prune::{NetworkPruneCommand, NetworkPruneResult};
pub use rm::{NetworkRmCommand, NetworkRmResult};
