//! Docker Compose command implementations.
//!
//! This module provides support for Docker Compose commands, enabling
//! multi-container application management.
//!
//! All compose commands follow the unified `DockerCommand` trait pattern,
//! providing consistent API with all other Docker commands in the crate.
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::compose::{ComposeUpCommand, ComposeDownCommand};
//! use docker_wrapper::DockerCommand;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Start services
//! ComposeUpCommand::new()
//!     .file("docker-compose.yml")
//!     .detach()
//!     .execute()
//!     .await?;
//!
//! // Stop services
//! ComposeDownCommand::new()
//!     .volumes()
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```

// Re-export compose types from command module
pub use crate::command::{AnsiMode, ComposeCommand, ComposeConfig, ProgressType};

// Re-export all compose commands
pub use crate::command::compose::{
    AttachResult, ComposeAttachCommand, ComposeBuildCommand, ComposeBuildResult,
    ComposeConfigCommand, ComposeConfigResult, ComposeContainerInfo, ComposeConvertCommand,
    ComposeConvertResult, ComposeCpCommand, ComposeCpResult, ComposeCreateCommand,
    ComposeCreateResult, ComposeDownCommand, ComposeDownResult, ComposeEvent, ComposeEventsCommand,
    ComposeEventsResult, ComposeExecCommand, ComposeExecResult, ComposeImagesCommand,
    ComposeImagesResult, ComposeKillCommand, ComposeKillResult, ComposeLogsCommand,
    ComposeLogsResult, ComposeLsCommand, ComposePauseCommand, ComposePauseResult,
    ComposePortCommand, ComposePortResult, ComposeProject, ComposePsCommand, ComposePsResult,
    ComposePushCommand, ComposePushResult, ComposeRestartCommand, ComposeRestartResult,
    ComposeRmCommand, ComposeRmResult, ComposeRunCommand, ComposeRunResult, ComposeScaleCommand,
    ComposeScaleResult, ComposeStartCommand, ComposeStartResult, ComposeStopCommand,
    ComposeStopResult, ComposeTopCommand, ComposeTopResult, ComposeUnpauseCommand,
    ComposeUnpauseResult, ComposeUpCommand, ComposeUpResult, ComposeVersionCommand,
    ComposeVersionResult, ComposeWaitCommand, ComposeWaitResult, ComposeWatchCommand,
    ComposeWatchResult, ConfigFormat, ContainerStatus, ConvertFormat, ImageInfo, ImagesFormat,
    LsFormat, LsResult, PortPublisher, ProgressOutput, PullPolicy, RemoveImages, VersionFormat,
    VersionInfo,
};
