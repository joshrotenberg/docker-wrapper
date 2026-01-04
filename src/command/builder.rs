//! Docker builder and buildx commands
//!
//! This module provides support for Docker builder/buildx commands that manage
//! the build subsystem, build cache, and builder instances.
//!
//! ## Builder Management
//!
//! - [`BuildxCreateCommand`] - Create a new builder instance
//! - [`BuildxUseCommand`] - Set the current builder instance
//! - [`BuildxInspectCommand`] - Inspect a builder instance
//! - [`BuildxLsCommand`] - List builder instances
//! - [`BuildxRmCommand`] - Remove builder instances
//! - [`BuildxStopCommand`] - Stop a builder instance
//!
//! ## Build Operations
//!
//! - [`BuilderBuildCommand`] - Build with extended features
//! - [`BuilderPruneCommand`] - Clean up build cache

pub mod build;
pub mod create;
pub mod inspect;
pub mod ls;
pub mod prune;
pub mod rm;
pub mod stop;
pub mod use_cmd;

pub use build::BuilderBuildCommand;
pub use create::{BuildxCreateCommand, BuildxCreateResult};
pub use inspect::{BuildxInspectCommand, BuildxInspectResult};
pub use ls::{BuilderInfo, BuildxLsCommand, BuildxLsResult};
pub use prune::{BuilderPruneCommand, BuilderPruneResult};
pub use rm::{BuildxRmCommand, BuildxRmResult};
pub use stop::{BuildxStopCommand, BuildxStopResult};
pub use use_cmd::{BuildxUseCommand, BuildxUseResult};
