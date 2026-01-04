//! Docker manifest command implementations.
//!
//! This module provides commands for managing Docker manifests and manifest lists.
//! Manifest lists allow you to use one name to refer to the same image built for
//! multiple architectures.
//!
//! Note: Docker manifest commands are experimental features.

pub mod annotate;
pub mod create;
pub mod inspect;
pub mod push;
pub mod rm;

pub use annotate::{ManifestAnnotateCommand, ManifestAnnotateResult};
pub use create::{ManifestCreateCommand, ManifestCreateResult};
pub use inspect::{ManifestInfo, ManifestInspectCommand, ManifestPlatform};
pub use push::{ManifestPushCommand, ManifestPushResult};
pub use rm::{ManifestRmCommand, ManifestRmResult};
