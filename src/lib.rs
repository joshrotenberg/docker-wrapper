//! # docker-wrapper
//!
//! A type-safe Docker CLI wrapper for Rust.
//!
//! This crate provides an idiomatic Rust interface to the Docker command-line tool.
//! All commands use a builder pattern and async execution via Tokio.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use docker_wrapper::{DockerCommand, RunCommand, StopCommand, RmCommand};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Run a container
//!     let container = RunCommand::new("nginx:alpine")
//!         .name("my-nginx")
//!         .port(8080, 80)
//!         .detach()
//!         .execute()
//!         .await?;
//!
//!     println!("Started: {}", container.short());
//!
//!     // Stop and remove
//!     StopCommand::new("my-nginx").execute().await?;
//!     RmCommand::new("my-nginx").execute().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Core Concepts
//!
//! ## The `DockerCommand` Trait
//!
//! All commands implement [`DockerCommand`], which provides the [`execute()`](DockerCommand::execute)
//! method. You must import this trait to call `.execute()`:
//!
//! ```rust
//! use docker_wrapper::DockerCommand; // Required for .execute()
//! ```
//!
//! ## Builder Pattern
//!
//! Commands are configured using method chaining:
//!
//! ```rust,no_run
//! # use docker_wrapper::{DockerCommand, RunCommand};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! RunCommand::new("alpine")
//!     .name("my-container")
//!     .env("DATABASE_URL", "postgres://localhost/db")
//!     .volume("/data", "/app/data")
//!     .port(3000, 3000)
//!     .memory("512m")
//!     .cpus("0.5")
//!     .detach()
//!     .rm()  // Auto-remove when stopped
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All commands return `Result<T, docker_wrapper::Error>`:
//!
//! ```rust,no_run
//! # use docker_wrapper::{DockerCommand, RunCommand, Error};
//! # async fn example() {
//! match RunCommand::new("nginx").detach().execute().await {
//!     Ok(id) => println!("Started: {}", id.short()),
//!     Err(Error::CommandFailed { stderr, .. }) => {
//!         eprintln!("Docker error: {}", stderr);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! # }
//! ```
//!
//! # Command Categories
//!
//! ## Container Lifecycle
//!
//! ```rust,no_run
//! use docker_wrapper::{
//!     DockerCommand,
//!     RunCommand,      // docker run
//!     CreateCommand,   // docker create
//!     StartCommand,    // docker start
//!     StopCommand,     // docker stop
//!     RestartCommand,  // docker restart
//!     KillCommand,     // docker kill
//!     RmCommand,       // docker rm
//!     PauseCommand,    // docker pause
//!     UnpauseCommand,  // docker unpause
//! };
//! ```
//!
//! ## Container Inspection
//!
//! ```rust,no_run
//! use docker_wrapper::{
//!     DockerCommand,
//!     PsCommand,       // docker ps
//!     LogsCommand,     // docker logs
//!     InspectCommand,  // docker inspect
//!     TopCommand,      // docker top
//!     StatsCommand,    // docker stats
//!     PortCommand,     // docker port
//!     DiffCommand,     // docker diff
//! };
//! ```
//!
//! ## Container Operations
//!
//! ```rust,no_run
//! use docker_wrapper::{
//!     DockerCommand,
//!     ExecCommand,     // docker exec
//!     AttachCommand,   // docker attach
//!     CpCommand,       // docker cp
//!     WaitCommand,     // docker wait
//!     RenameCommand,   // docker rename
//!     UpdateCommand,   // docker update
//!     CommitCommand,   // docker commit
//!     ExportCommand,   // docker export
//! };
//! ```
//!
//! ## Images
//!
//! ```rust,no_run
//! use docker_wrapper::{
//!     DockerCommand,
//!     ImagesCommand,   // docker images
//!     PullCommand,     // docker pull
//!     PushCommand,     // docker push
//!     BuildCommand,    // docker build
//!     TagCommand,      // docker tag
//!     RmiCommand,      // docker rmi
//!     SaveCommand,     // docker save
//!     LoadCommand,     // docker load
//!     ImportCommand,   // docker import
//!     HistoryCommand,  // docker history
//!     SearchCommand,   // docker search
//! };
//! ```
//!
//! ## Networks and Volumes
//!
//! ```rust,no_run
//! use docker_wrapper::{
//!     DockerCommand,
//!     NetworkCreateCommand, NetworkLsCommand, NetworkRmCommand,
//!     VolumeCreateCommand, VolumeLsCommand, VolumeRmCommand,
//! };
//! ```
//!
//! ## System
//!
//! ```rust,no_run
//! use docker_wrapper::{
//!     DockerCommand,
//!     VersionCommand,  // docker version
//!     InfoCommand,     // docker info
//!     EventsCommand,   // docker events
//!     LoginCommand,    // docker login
//!     LogoutCommand,   // docker logout
//!     SystemDfCommand, // docker system df
//!     SystemPruneCommand, // docker system prune
//! };
//! ```
//!
//! # Feature Flags
//!
//! ## `compose` - Docker Compose Support
//!
//! ```toml
//! docker-wrapper = { version = "0.8", features = ["compose"] }
//! ```
//!
//! ```rust,no_run
//! # #[cfg(feature = "compose")]
//! use docker_wrapper::{DockerCommand, compose::{ComposeUpCommand, ComposeDownCommand, ComposeCommand}};
//!
//! # #[cfg(feature = "compose")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Start services
//! ComposeUpCommand::new()
//!     .file("docker-compose.yml")
//!     .detach()
//!     .execute()
//!     .await?;
//!
//! // Stop and clean up
//! ComposeDownCommand::new()
//!     .volumes()
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## `templates` - Pre-configured Containers
//!
//! ```toml
//! docker-wrapper = { version = "0.8", features = ["templates"] }
//! ```
//!
//! Templates provide ready-to-use configurations for common services:
//!
//! ```rust,no_run
//! # #[cfg(feature = "template-redis")]
//! use docker_wrapper::{RedisTemplate, Template};
//!
//! # #[cfg(feature = "template-redis")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let redis = RedisTemplate::new("my-redis")
//!     .port(6379)
//!     .password("secret")
//!     .with_persistence("redis-data");
//!
//! let id = redis.start().await?;
//! // ... use Redis ...
//! redis.stop().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Available templates:
//! - [`RedisTemplate`], [`RedisSentinelTemplate`], [`RedisClusterTemplate`]
//! - [`PostgresTemplate`], [`MysqlTemplate`], [`MongodbTemplate`]
//! - [`NginxTemplate`]
//!
//! ## `swarm` - Docker Swarm Commands
//!
//! ```toml
//! docker-wrapper = { version = "0.8", features = ["swarm"] }
//! ```
//!
//! ## `manifest` - Multi-arch Manifest Commands
//!
//! ```toml
//! docker-wrapper = { version = "0.8", features = ["manifest"] }
//! ```
//!
//! # Streaming Output
//!
//! For long-running commands, stream output in real-time:
//!
//! ```rust,no_run
//! use docker_wrapper::{BuildCommand, StreamHandler, StreamableCommand};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let result = BuildCommand::new(".")
//!     .tag("my-app:latest")
//!     .stream(StreamHandler::print())
//!     .await?;
//!
//! if result.is_success() {
//!     println!("Build complete!");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Checking Docker Availability
//!
//! ```rust,no_run
//! use docker_wrapper::ensure_docker;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let info = ensure_docker().await?;
//! println!("Docker {}.{}.{}", info.version.major, info.version.minor, info.version.patch);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod command;
#[cfg(feature = "compose")]
pub mod compose;
pub mod debug;
pub mod error;
pub mod platform;
pub mod prerequisites;
pub mod stream;
#[cfg(any(
    feature = "templates",
    feature = "template-redis",
    feature = "template-redis-cluster",
    feature = "template-postgres",
    feature = "template-mysql",
    feature = "template-mongodb",
    feature = "template-nginx"
))]
/// Container templates module
///
/// Provides pre-configured container templates with sensible defaults for common services.
/// Templates support custom images, platforms, persistence, and resource configuration.
///
/// See the [Template Guide](https://github.com/joshrotenberg/docker-wrapper/blob/main/docs/TEMPLATES.md) for comprehensive documentation.
///
/// # Available Templates
///
/// ## Redis Templates
/// - [`RedisTemplate`] - Basic Redis server
/// - [`RedisSentinelTemplate`] - High-availability Redis with Sentinel
/// - [`RedisClusterTemplate`] - Sharded Redis cluster
/// - [`RedisEnterpriseTemplate`] - Redis Enterprise with management
/// - [`RedisInsightTemplate`] - Redis management UI
///
/// ## Database Templates
/// - [`PostgresTemplate`] - PostgreSQL database
/// - [`MysqlTemplate`] - MySQL database
/// - [`MongodbTemplate`] - MongoDB document database
///
/// ## Web Server Templates
/// - [`NginxTemplate`] - Nginx web server
///
/// # Quick Start
///
/// ```rust,no_run
/// use docker_wrapper::{RedisTemplate, Template};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let redis = RedisTemplate::new("my-redis")
///     .port(6379)
///     .password("secret")
///     .with_persistence("redis-data");
///
/// let container_id = redis.start().await?;
/// println!("Redis started: {}", container_id);
/// # Ok(())
/// # }
/// ```
#[cfg(any(
    feature = "templates",
    feature = "template-redis",
    feature = "template-redis-cluster",
    feature = "template-postgres",
    feature = "template-mysql",
    feature = "template-mongodb",
    feature = "template-nginx"
))]
pub mod template;

pub use stream::{OutputLine, StreamHandler, StreamResult, StreamableCommand};

pub use command::{
    attach::{AttachCommand, AttachResult},
    bake::BakeCommand,
    build::{BuildCommand, BuildOutput},
    builder::{
        BuilderBuildCommand, BuilderInfo, BuilderPruneCommand, BuilderPruneResult,
        BuildxCreateCommand, BuildxCreateResult, BuildxInspectCommand, BuildxInspectResult,
        BuildxLsCommand, BuildxLsResult, BuildxRmCommand, BuildxRmResult, BuildxStopCommand,
        BuildxStopResult, BuildxUseCommand, BuildxUseResult,
    },
    commit::{CommitCommand, CommitResult},
    container_prune::{ContainerPruneCommand, ContainerPruneResult},
    context::{
        ContextCreateCommand, ContextInfo, ContextInspectCommand, ContextLsCommand,
        ContextRmCommand, ContextUpdateCommand, ContextUseCommand,
    },
    cp::{CpCommand, CpResult},
    create::{CreateCommand, CreateResult},
    diff::{DiffCommand, DiffResult, FilesystemChange, FilesystemChangeType},
    events::{DockerEvent, EventActor, EventsCommand, EventsResult},
    exec::{ExecCommand, ExecOutput},
    export::{ExportCommand, ExportResult},
    generic::GenericCommand,
    history::{HistoryCommand, HistoryResult, ImageLayer},
    image_prune::{DeletedImage, ImagePruneCommand, ImagePruneResult},
    images::{ImageInfo, ImagesCommand, ImagesOutput},
    import::{ImportCommand, ImportResult},
    info::{DockerInfo as SystemDockerInfo, InfoCommand, InfoOutput, SystemInfo},
    init::{InitCommand, InitOutput, InitTemplate},
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
    port::{PortCommand, PortMapping as PortMappingInfo, PortResult},
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
    Protocol, DEFAULT_COMMAND_TIMEOUT,
};
pub use debug::{BackoffStrategy, DebugConfig, DebugExecutor, DryRunPreview, RetryPolicy};
pub use error::{Error, Result};
pub use platform::{Platform, PlatformInfo, Runtime};

// Swarm commands (feature-gated)
#[cfg(feature = "swarm")]
pub use command::swarm::{
    SwarmCaCommand, SwarmCaResult, SwarmInitCommand, SwarmInitResult, SwarmJoinCommand,
    SwarmJoinResult, SwarmJoinTokenCommand, SwarmJoinTokenResult, SwarmLeaveCommand,
    SwarmLeaveResult, SwarmNodeRole, SwarmUnlockCommand, SwarmUnlockKeyCommand,
    SwarmUnlockKeyResult, SwarmUnlockResult, SwarmUpdateCommand, SwarmUpdateResult,
};

// Manifest commands (feature-gated)
#[cfg(feature = "manifest")]
pub use command::manifest::{
    ManifestAnnotateCommand, ManifestAnnotateResult, ManifestCreateCommand, ManifestCreateResult,
    ManifestInfo, ManifestInspectCommand, ManifestPlatform, ManifestPushCommand,
    ManifestPushResult, ManifestRmCommand, ManifestRmResult,
};

pub use prerequisites::{
    ensure_docker, ensure_docker_with_timeout, DockerInfo, DockerPrerequisites,
    DEFAULT_PREREQ_TIMEOUT,
};

#[cfg(any(
    feature = "templates",
    feature = "template-redis",
    feature = "template-redis-cluster",
    feature = "template-postgres",
    feature = "template-mysql",
    feature = "template-mongodb",
    feature = "template-nginx"
))]
pub use template::{Template, TemplateBuilder, TemplateConfig, TemplateError};

// Redis templates
#[cfg(feature = "template-redis")]
pub use template::redis::{RedisInsightTemplate, RedisTemplate};

#[cfg(feature = "template-redis")]
pub use template::redis::{RedisSentinelTemplate, SentinelConnectionInfo, SentinelInfo};

#[cfg(feature = "template-redis-cluster")]
pub use template::redis::{
    ClusterInfo, NodeInfo, NodeRole, RedisClusterConnection, RedisClusterTemplate,
};

#[cfg(feature = "template-redis-enterprise")]
pub use template::redis::{RedisEnterpriseConnectionInfo, RedisEnterpriseTemplate};

// Database templates
#[cfg(feature = "template-postgres")]
pub use template::database::{PostgresConnectionString, PostgresTemplate};

#[cfg(feature = "template-mysql")]
pub use template::database::{MysqlConnectionString, MysqlTemplate};

#[cfg(feature = "template-mongodb")]
pub use template::database::{MongodbConnectionString, MongodbTemplate};

// Web server templates
#[cfg(feature = "template-nginx")]
pub use template::web::NginxTemplate;

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
