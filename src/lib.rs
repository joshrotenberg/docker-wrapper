//! # docker-wrapper
//!
//! A comprehensive, type-safe Docker CLI wrapper for Rust applications.
//!
//! `docker-wrapper` provides a clean, idiomatic Rust interface to Docker's command-line interface,
//! supporting all major Docker commands with strong typing, async/await support, and a consistent
//! builder pattern API.
//!
//! ## Features
//!
//! - **Complete Docker CLI coverage**: Implements all 35 essential Docker commands
//! - **Type-safe builder pattern**: Compile-time validation of command construction
//! - **Async/await support**: Built on Tokio for efficient async operations
//! - **Streaming support**: Real-time output streaming for long-running commands
//! - **Docker Compose support**: Optional feature for multi-container orchestration
//! - **Zero dependencies on Docker SDK**: Works directly with the Docker CLI
//! - **Comprehensive error handling**: Detailed error messages and types
//! - **Well-tested**: Extensive unit and integration test coverage
//!
//! ## Quick Start
//!
//! Add `docker-wrapper` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! docker-wrapper = "0.2"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! For Docker Compose support, enable the `compose` feature:
//!
//! ```toml
//! [dependencies]
//! docker-wrapper = { version = "0.2", features = ["compose"] }
//! ```
//!
//! ## Basic Usage
//!
//! ### Running a Container
//!
//! ```rust,no_run
//! use docker_wrapper::{DockerCommand, RunCommand};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Run a simple container
//! let output = RunCommand::new("nginx:latest")
//!     .name("my-web-server")
//!     .port(8080, 80)
//!     .detach()
//!     .execute()
//!     .await?;
//!
//! println!("Container started: {}", output.0);
//! # Ok(())
//! # }
//! ```
//!
//! ### Building an Image
//!
//! ```rust,no_run
//! use docker_wrapper::{DockerCommand, BuildCommand};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let output = BuildCommand::new(".")
//!     .tag("my-app:latest")
//!     .file("Dockerfile")
//!     .build_arg("VERSION", "1.0.0")
//!     .execute()
//!     .await?;
//!
//! if let Some(image_id) = &output.image_id {
//!     println!("Built image: {}", image_id);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Listing Containers
//!
//! ```rust,no_run
//! use docker_wrapper::{DockerCommand, PsCommand};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let output = PsCommand::new()
//!     .all()
//!     .format_json()
//!     .execute()
//!     .await?;
//!
//! for container in output.containers {
//!     println!("{}: {}", container.names, container.status);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming Output
//!
//! For long-running commands, use the streaming API to process output in real-time:
//!
//! ```rust,no_run
//! use docker_wrapper::{BuildCommand, StreamHandler, StreamableCommand};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Stream build output to console
//! let result = BuildCommand::new(".")
//!     .tag("my-app:latest")
//!     .stream(StreamHandler::print())
//!     .await?;
//!
//! if result.is_success() {
//!     println!("Build completed successfully!");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Docker Compose Support
//!
//! When the `compose` feature is enabled, you can manage multi-container applications:
//!
//! ```rust,no_run
//! # #[cfg(feature = "compose")]
//! use docker_wrapper::compose::{ComposeCommand, ComposeUpCommand, ComposeDownCommand};
//!
//! # #[cfg(feature = "compose")]
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Start services defined in docker-compose.yml
//! ComposeUpCommand::new()
//!     .file("docker-compose.yml")
//!     .detach()
//!     .execute()
//!     .await?;
//!
//! // Stop and remove services
//! ComposeDownCommand::new()
//!     .volumes()
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "compose"))]
//! # fn main() {}
//! ```
//!
//! ## Architecture
//!
//! The crate is organized around several key design patterns:
//!
//! ### Command Trait Pattern
//!
//! All Docker commands implement the `DockerCommand` trait, providing a consistent interface:
//!
//! - `new()` - Create a new command instance
//! - `execute()` - Run the command and return typed output
//! - Builder methods for setting options
//!
//! ### Builder Pattern
//!
//! Commands use the builder pattern for configuration, allowing fluent and intuitive API usage:
//!
//! ```rust,no_run
//! # use docker_wrapper::{DockerCommand, RunCommand};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! RunCommand::new("alpine")
//!     .name("my-container")
//!     .env("KEY", "value")
//!     .volume("/host/path", "/container/path")
//!     .workdir("/app")
//!     .cmd(vec!["echo".to_string(), "hello".to_string()])
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Error Handling
//!
//! All operations return `Result<T, docker_wrapper::Error>`, providing detailed error information:
//!
//! ```rust,no_run
//! # use docker_wrapper::{DockerCommand, RunCommand};
//! # #[tokio::main]
//! # async fn main() {
//! match RunCommand::new("invalid:image").execute().await {
//!     Ok(output) => println!("Container ID: {}", output.0),
//!     Err(e) => eprintln!("Failed to run container: {}", e),
//! }
//! # }
//! ```
//!
//! ## Command Coverage
//!
//! ### Container Commands
//! - `run` - Run a new container
//! - `exec` - Execute commands in running containers
//! - `ps` - List containers
//! - `create` - Create a new container without starting it
//! - `start` - Start stopped containers
//! - `stop` - Stop running containers
//! - `restart` - Restart containers
//! - `kill` - Kill running containers
//! - `rm` - Remove containers
//! - `pause` - Pause running containers
//! - `unpause` - Unpause paused containers
//! - `attach` - Attach to running containers
//! - `wait` - Wait for containers to stop
//! - `logs` - Fetch container logs
//! - `top` - Display running processes in containers
//! - `stats` - Display resource usage statistics
//! - `port` - List port mappings
//! - `rename` - Rename containers
//! - `update` - Update container configuration
//! - `cp` - Copy files between containers and host
//! - `diff` - Inspect filesystem changes
//! - `export` - Export container filesystem
//! - `commit` - Create image from container
//!
//! ### Image Commands
//! - `images` - List images
//! - `pull` - Pull images from registry
//! - `push` - Push images to registry
//! - `build` - Build images from Dockerfile
//! - `load` - Load images from tar archive
//! - `save` - Save images to tar archive
//! - `rmi` - Remove images
//! - `tag` - Tag images
//! - `import` - Import images from tarball
//! - `history` - Show image history
//! - `inspect` - Display detailed information
//! - `search` - Search Docker Hub for images
//!
//! ### System Commands
//! - `info` - Display system information
//! - `version` - Show Docker version
//! - `events` - Monitor Docker events
//! - `login` - Log in to registry
//! - `logout` - Log out from registry
//!
//! ## Prerequisites Check
//!
//! Ensure Docker is installed and accessible:
//!
//! ```rust,no_run
//! use docker_wrapper::ensure_docker;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Check Docker availability and version
//! let docker_info = ensure_docker().await?;
//! println!("Docker version: {}.{}.{}",
//!     docker_info.version.major,
//!     docker_info.version.minor,
//!     docker_info.version.patch);
//! # Ok(())
//! # }
//! ```
//!
//! ## Best Practices
//!
//! ### Resource Cleanup
//!
//! Always clean up containers and resources:
//!
//! ```rust,no_run
//! # use docker_wrapper::{DockerCommand, RunCommand, RmCommand};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Use auto-remove for temporary containers
//! RunCommand::new("alpine")
//!     .remove()  // Automatically remove when stopped
//!     .execute()
//!     .await?;
//!
//! // Or manually remove containers
//! RmCommand::new("my-container")
//!     .force()
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Error Handling
//!
//! Handle errors appropriately for production use:
//!
//! ```rust,no_run
//! # use docker_wrapper::{DockerCommand, RunCommand, Error};
//! # #[tokio::main]
//! # async fn main() {
//! async fn run_container() -> Result<String, Error> {
//!     let output = RunCommand::new("nginx")
//!         .detach()
//!         .execute()
//!         .await?;
//!     Ok(output.0)
//! }
//!
//! match run_container().await {
//!     Ok(id) => println!("Started container: {}", id),
//!     Err(Error::CommandFailed { stderr, .. }) => {
//!         eprintln!("Docker command failed: {}", stderr);
//!     }
//!     Err(e) => eprintln!("Unexpected error: {}", e),
//! }
//! # }
//! ```
//!
//! ## Examples
//!
//! The `examples/` directory contains comprehensive examples:
//!
//! - `basic_usage.rs` - Common Docker operations
//! - `container_lifecycle.rs` - Container management patterns
//! - `docker_compose.rs` - Docker Compose usage
//! - `streaming.rs` - Real-time output streaming
//! - `error_handling.rs` - Error handling patterns
//!
//! Run examples with:
//!
//! ```bash
//! cargo run --example basic_usage
//! cargo run --example streaming
//! cargo run --features compose --example docker_compose
//! ```
//!
//! ## Migration from Docker CLI
//!
//! Migrating from shell scripts to `docker-wrapper` is straightforward:
//!
//! **Shell:**
//! ```bash
//! docker run -d --name web -p 8080:80 nginx:latest
//! ```
//!
//! **Rust:**
//! ```rust,no_run
//! # use docker_wrapper::{DockerCommand, RunCommand};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! RunCommand::new("nginx:latest")
//!     .detach()
//!     .name("web")
//!     .port(8080, 80)
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Contributing
//!
//! Contributions are welcome! Please see the [GitHub repository](https://github.com/joshrotenberg/docker-wrapper)
//! for contribution guidelines and development setup.
//!
//! ## License
//!
//! This project is licensed under the MIT License - see the LICENSE file for details.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod command;
#[cfg(feature = "compose")]
pub mod compose;
pub mod error;
pub mod platform;
pub mod prerequisites;
pub mod stream;

pub use stream::{OutputLine, StreamHandler, StreamResult, StreamableCommand};

pub use command::{
    attach::{AttachCommand, AttachResult},
    bake::BakeCommand,
    build::{BuildCommand, BuildOutput},
    commit::{CommitCommand, CommitResult},
    container_prune::{ContainerPruneCommand, ContainerPruneResult},
    cp::{CpCommand, CpResult},
    create::{CreateCommand, CreateResult},
    diff::{DiffCommand, DiffResult, FilesystemChange, FilesystemChangeType},
    events::{DockerEvent, EventActor, EventsCommand, EventsResult},
    exec::{ExecCommand, ExecOutput},
    export::{ExportCommand, ExportResult},
    history::{HistoryCommand, HistoryResult, ImageLayer},
    image_prune::{DeletedImage, ImagePruneCommand, ImagePruneResult},
    images::{ImageInfo, ImagesCommand, ImagesOutput},
    import::{ImportCommand, ImportResult},
    info::{DockerInfo as SystemDockerInfo, InfoCommand, InfoOutput, SystemInfo},
    inspect::{InspectCommand, InspectOutput},
    kill::{KillCommand, KillResult},
    load::{LoadCommand, LoadResult},
    login::{LoginCommand, LoginOutput},
    logout::{LogoutCommand, LogoutOutput},
    logs::LogsCommand,
    network::{
        NetworkConnectCommand, NetworkConnectResult, NetworkCreateCommand, NetworkCreateResult,
        NetworkDisconnectCommand, NetworkDisconnectResult, NetworkInfo, NetworkInspectCommand,
        NetworkInspectOutput, NetworkLsCommand, NetworkLsOutput, NetworkPruneCommand,
        NetworkPruneResult, NetworkRmCommand, NetworkRmResult,
    },
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
    system::{
        BuildCacheInfo, BuildCacheUsage, ContainerInfo as SystemContainerInfo, ContainerUsage,
        DiskUsage, ImageInfo as SystemImageInfo, ImageUsage, PruneResult, SystemDfCommand,
        SystemPruneCommand, VolumeInfo as SystemVolumeInfo, VolumeUsage,
    },
    tag::{TagCommand, TagResult},
    top::{ContainerProcess, TopCommand, TopResult},
    unpause::{UnpauseCommand, UnpauseResult},
    update::{UpdateCommand, UpdateResult},
    version::{ClientVersion, ServerVersion, VersionCommand, VersionInfo, VersionOutput},
    volume::{
        VolumeCreateCommand, VolumeCreateResult, VolumeInfo, VolumeInspectCommand,
        VolumeInspectOutput, VolumeLsCommand, VolumeLsOutput, VolumePruneCommand,
        VolumePruneResult, VolumeRmCommand, VolumeRmResult,
    },
    wait::{WaitCommand, WaitResult},
    CommandExecutor, CommandOutput, DockerCommand, EnvironmentBuilder, PortBuilder, PortMapping,
    Protocol,
};
pub use error::{Error, Result};
pub use platform::{Platform, PlatformInfo, Runtime};
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
