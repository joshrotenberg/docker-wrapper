//! Docker Compose command implementations using the unified trait pattern.
//!
//! This module provides Docker Compose commands that follow the same
//! `DockerCommand` trait pattern as all other commands in the crate.

pub mod attach;
pub mod build;
pub mod config;
pub mod convert;
pub mod cp;
pub mod create;
pub mod down;
pub mod events;
pub mod exec;
pub mod images;
pub mod kill;
pub mod logs;
pub mod ls;
pub mod pause;
pub mod port;
pub mod ps;
pub mod push;
pub mod restart;
pub mod rm;
pub mod run;
pub mod scale;
pub mod start;
pub mod stop;
pub mod top;
pub mod unpause;
pub mod up;
pub mod version;
pub mod wait;
pub mod watch;

pub use attach::{AttachResult, ComposeAttachCommand};
pub use build::{ComposeBuildCommand, ComposeBuildResult, ProgressOutput};
pub use config::{ComposeConfigCommand, ComposeConfigResult, ConfigFormat};
pub use convert::{ComposeConvertCommand, ComposeConvertResult, ConvertFormat};
pub use cp::{ComposeCpCommand, ComposeCpResult};
pub use create::{ComposeCreateCommand, ComposeCreateResult, PullPolicy};
pub use down::{ComposeDownCommand, ComposeDownResult, RemoveImages};
pub use events::{ComposeEvent, ComposeEventsCommand, ComposeEventsResult};
pub use exec::{ComposeExecCommand, ComposeExecResult};
pub use images::{ComposeImagesCommand, ComposeImagesResult, ImageInfo, ImagesFormat};
pub use kill::{ComposeKillCommand, ComposeKillResult};
pub use logs::{ComposeLogsCommand, ComposeLogsResult};
pub use ls::{ComposeLsCommand, ComposeProject, LsFormat, LsResult};
pub use pause::{ComposePauseCommand, ComposePauseResult};
pub use port::{ComposePortCommand, ComposePortResult};
pub use ps::{
    ComposeContainerInfo, ComposePsCommand, ComposePsResult, ContainerStatus, PortPublisher,
};
pub use push::{ComposePushCommand, ComposePushResult};
pub use restart::{ComposeRestartCommand, ComposeRestartResult};
pub use rm::{ComposeRmCommand, ComposeRmResult};
pub use run::{ComposeRunCommand, ComposeRunResult};
pub use scale::{ComposeScaleCommand, ComposeScaleResult};
pub use start::{ComposeStartCommand, ComposeStartResult};
pub use stop::{ComposeStopCommand, ComposeStopResult};
pub use top::{ComposeTopCommand, ComposeTopResult};
pub use unpause::{ComposeUnpauseCommand, ComposeUnpauseResult};
pub use up::{ComposeUpCommand, ComposeUpResult};
pub use version::{ComposeVersionCommand, ComposeVersionResult, VersionFormat, VersionInfo};
pub use wait::{ComposeWaitCommand, ComposeWaitResult};
pub use watch::{ComposeWatchCommand, ComposeWatchResult};
