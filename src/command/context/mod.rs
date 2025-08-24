//! Docker context management commands
//!
//! This module provides commands for managing Docker contexts, which allow
//! you to quickly switch between different Docker daemons.

pub mod create;
pub mod inspect;
pub mod ls;
pub mod rm;
pub mod update;
pub mod use_context;

pub use create::ContextCreateCommand;
pub use inspect::ContextInspectCommand;
pub use ls::ContextLsCommand;
pub use rm::ContextRmCommand;
pub use update::ContextUpdateCommand;
pub use use_context::ContextUseCommand;
