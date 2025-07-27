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

pub use command::{
    bake::BakeCommand,
    build::{BuildCommand, BuildOutput},
    exec::{ExecCommand, ExecOutput},
    images::{ImageInfo, ImagesCommand, ImagesOutput},
    info::{DockerInfo as SystemDockerInfo, InfoCommand, InfoOutput, SystemInfo},
    login::{LoginCommand, LoginOutput},
    logout::{LogoutCommand, LogoutOutput},
    ps::{ContainerInfo, PsCommand, PsFormat, PsOutput},
    pull::PullCommand,
    push::PushCommand,
    run::{ContainerId, MountType, RunCommand, VolumeMount},
    search::{RepositoryInfo, SearchCommand, SearchOutput},
    version::{ClientVersion, ServerVersion, VersionCommand, VersionInfo, VersionOutput},
    CommandExecutor, CommandOutput, DockerCommand, EnvironmentBuilder, PortBuilder, PortMapping,
    Protocol,
};
pub use error::{Error, Result};
pub use prerequisites::{ensure_docker, DockerInfo, DockerPrerequisites};

/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // Verify version follows semver format (major.minor.patch)
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(parts.len() >= 3, "Version should have at least 3 parts");

        // Verify each part is numeric
        for part in &parts[0..3] {
            assert!(
                part.chars().all(|c| c.is_ascii_digit()),
                "Version parts should be numeric"
            );
        }
    }
}
