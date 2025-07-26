//! # docker-wrapper
//!
//! A simple, focused Docker CLI wrapper for Rust.
//!
//! This crate provides a clean interface to Docker's common commands,
//! focusing on reliability and ease of use.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod command;
pub mod error;
pub mod prerequisites;
pub mod run;

pub use command::{
    CommandExecutor, CommandOutput, DockerCommand, EnvironmentBuilder, PortBuilder, PortMapping,
    Protocol,
};
pub use error::{Error, Result};
pub use prerequisites::{ensure_docker, DockerInfo, DockerPrerequisites};
pub use run::{ContainerId, MountType, RunCommand, VolumeMount};

/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
