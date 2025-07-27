//! # docker-wrapper
//!
//! A simple, focused Docker CLI wrapper for Rust.
//!
//! This crate provides a clean interface to Docker's common commands,
//! focusing on reliability and ease of use.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod bake;
pub mod build;
pub mod command;
pub mod error;
pub mod exec;
pub mod images;
pub mod login;
pub mod logout;
pub mod prerequisites;
pub mod ps;
pub mod pull;
pub mod push;
pub mod run;
pub mod search;

pub use bake::BakeCommand;
pub use build::{BuildCommand, BuildOutput};
pub use command::{
    CommandExecutor, CommandOutput, DockerCommand, EnvironmentBuilder, PortBuilder, PortMapping,
    Protocol,
};
pub use error::{Error, Result};
pub use exec::{ExecCommand, ExecOutput};
pub use images::{ImageInfo, ImagesCommand, ImagesOutput};
pub use login::{LoginCommand, LoginOutput};
pub use logout::{LogoutCommand, LogoutOutput};
pub use prerequisites::{ensure_docker, DockerInfo, DockerPrerequisites};
pub use ps::{ContainerInfo, PsCommand, PsFormat, PsOutput};
pub use pull::PullCommand;
pub use push::PushCommand;
pub use run::{ContainerId, MountType, RunCommand, VolumeMount};
pub use search::{RepositoryInfo, SearchCommand, SearchOutput};

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
