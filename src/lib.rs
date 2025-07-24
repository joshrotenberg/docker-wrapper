//! # docker-wrapper
//!
//! A comprehensive Docker CLI wrapper for Rust with full container lifecycle management.
//!
//! This crate provides a type-safe, async-first interface to Docker operations,
//! eliminating the limitations of existing solutions by providing direct access
//! to all Docker CLI functionality.
//!
//! ## Features
//!
//! - **Complete Docker CLI coverage** - All commonly used Docker operations
//! - **Type-safe interface** - Strong Rust types for all Docker concepts
//! - **Async-first design** - Built for modern async Rust applications
//! - **Zero limitations** - Full access to all Docker features
//! - **Production ready** - Robust error handling, logging, and debugging
//! - **Testing focused** - Designed for integration testing workflows
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use docker_wrapper::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DockerError> {
//!     let client = DockerClient::new().await?;
//!
//!     // Build and run a container
//!     let container_id = ContainerBuilder::new("redis:7.2-alpine")
//!         .name("my-redis")
//!         .port_dynamic(6379)
//!         .env("REDIS_PASSWORD", "secret")
//!         .command(vec!["redis-server".to_string(), "--requirepass".to_string(), "secret".to_string()])
//!         .run(&client)
//!         .await?;
//!
//!     // Wait for it to be ready
//!     client.wait_for_ready(&container_id, std::time::Duration::from_secs(30)).await?;
//!
//!     // Get the dynamic port
//!     let host_port = client.port(&container_id, 6379).await?.unwrap();
//!     println!("Redis running on port: {}", host_port);
//!
//!     // Cleanup
//!     client.stop(&container_id, Some(std::time::Duration::from_secs(10))).await?;
//!     client.remove(&container_id, RemoveOptions::default()).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into several key modules:
//!
//! - [`client`] - Core Docker client and command execution
//! - [`container`] - Container lifecycle management
//! - [`image`] - Image operations and management
//! - [`network`] - Network creation and management
//! - [`volume`] - Volume operations
//! - [`types`] - Core types and data structures
//! - [`errors`] - Error types and handling
//! - [`executor`] - Low-level process execution

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // TODO: Add error docs in later phases

// Re-export the most commonly used types and traits
pub use client::{DockerClient, DockerInfo, DockerVersion};
pub use container::{
    ContainerBuilder, ContainerConfig, ContainerExecutor, ContainerManager, DockerContainer,
    ExecConfig, ExecOutput, ExecResult, HealthCheck, HealthCheckConfig, HealthCheckResult,
    HealthChecker, LogEntry, LogManager, LogOptions, LogSource, RemoveOptions,
};
pub use errors::{DockerError, DockerResult};
pub use events::{
    ContainerEvent, DockerEvent, EventFilter, EventManager, EventStats, EventStream, EventType,
    ImageEvent, NetworkEvent, VolumeEvent,
};
pub use image::{
    BuildOptions, BuildProgress, DockerImage, ImageHistoryItem, ImageInspect, ImageManager,
    ImageRef, ListImagesOptions, PruneResult, PullOptions, PullProgress, RegistryAuth,
    RemoveImageOptions, RemoveImageResult,
};
pub use network::{
    ConnectOptions, DisconnectOptions, DockerNetwork, IPAMConfig, ListNetworksOptions,
    NetworkConfig, NetworkContainer, NetworkDriver, NetworkIPAM, NetworkInspect, NetworkManager,
    NetworkPruneResult,
};
pub use stats::{
    ContainerStats, StatsAggregator, StatsManager, StatsOptions, StatsStream, SystemStats,
};
pub use types::{ContainerId, ContainerStatus, ImageRef, NetworkId, PortMapping, VolumeMount};
pub use volume::{
    DockerVolume, ListVolumesOptions, RemoveVolumeOptions, VolumeConfig, VolumeInspect,
    VolumeManager, VolumePruneResult, VolumeSource, VolumeUsageData, VolumeUsageStats,
};

// Core modules
pub mod client;
pub mod container;
pub mod errors;
pub mod events;
pub mod executor;
pub mod image;
pub mod network;
pub mod stats;
pub mod types;
pub mod volume;

// Feature-gated modules would go here
// #[cfg(feature = "json")]
// pub mod json;

// Optional CLI module - not implemented yet
// #[cfg(feature = "cli")]
// pub mod cli;

// Internal utilities
mod utils;

// Version information
/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The minimum supported Docker version
pub const MIN_DOCKER_VERSION: &str = "20.10.0";

/// Default timeout for Docker operations
pub const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!MIN_DOCKER_VERSION.is_empty());
    }

    #[test]
    fn test_timeout_constants() {
        assert!(DEFAULT_TIMEOUT > std::time::Duration::from_secs(0));
    }
}
