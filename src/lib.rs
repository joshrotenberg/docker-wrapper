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
#[cfg(feature = "compose")]
pub mod compose;
pub mod error;
pub mod prerequisites;

pub use command::{
    attach::{AttachCommand, AttachResult},
    bake::BakeCommand,
    build::{BuildCommand, BuildOutput},
    commit::{CommitCommand, CommitResult},
    cp::{CpCommand, CpResult},
    create::{CreateCommand, CreateResult},
    diff::{DiffCommand, DiffResult, FilesystemChange, FilesystemChangeType},
    events::{DockerEvent, EventActor, EventsCommand, EventsResult},
    exec::{ExecCommand, ExecOutput},
    export::{ExportCommand, ExportResult},
    history::{HistoryCommand, HistoryResult, ImageLayer},
    images::{ImageInfo, ImagesCommand, ImagesOutput},
    import::{ImportCommand, ImportResult},
    info::{DockerInfo as SystemDockerInfo, InfoCommand, InfoOutput, SystemInfo},
    inspect::{InspectCommand, InspectOutput},
    kill::{KillCommand, KillResult},
    load::{LoadCommand, LoadResult},
    login::{LoginCommand, LoginOutput},
    logout::{LogoutCommand, LogoutOutput},
    logs::LogsCommand,
    pause::{PauseCommand, PauseResult},
    port::{PortCommand, PortMapping as PortInfo, PortResult},
    ps::{ContainerInfo, PsCommand, PsFormat, PsOutput},
    pull::PullCommand,
    push::PushCommand,
    rename::{RenameCommand, RenameResult},
    restart::{RestartCommand, RestartResult},
    rm::{RmCommand, RmResult},
    rmi::{RmiCommand, RmiResult},
    run::{ContainerId, MountType, RunCommand, VolumeMount},
    save::{SaveCommand, SaveResult},
    search::{RepositoryInfo, SearchCommand, SearchOutput},
    start::{StartCommand, StartResult},
    stats::{ContainerStats, StatsCommand, StatsResult},
    stop::{StopCommand, StopResult},
    tag::{TagCommand, TagResult},
    top::{ContainerProcess, TopCommand, TopResult},
    unpause::{UnpauseCommand, UnpauseResult},
    update::{UpdateCommand, UpdateResult},
    version::{ClientVersion, ServerVersion, VersionCommand, VersionInfo, VersionOutput},
    wait::{WaitCommand, WaitResult},
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
