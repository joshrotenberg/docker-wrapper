//! # Testing Utilities
//!
//! RAII-style container lifecycle management for integration tests.
//!
//! This module provides [`ContainerGuard`] and [`ContainerGuardSet`] for automatic
//! container lifecycle management. Containers are automatically stopped and removed
//! when guards go out of scope, ensuring clean test environments.
//!
//! ## Why Use This?
//!
//! - **Automatic cleanup**: No more forgotten containers cluttering your Docker
//! - **Panic-safe**: Containers are cleaned up even if your test panics
//! - **Debug-friendly**: Keep containers alive on failure for inspection
//! - **Network support**: Automatic network creation for multi-container tests
//! - **Ready checks**: Wait for services to be ready before running tests
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use docker_wrapper::testing::ContainerGuard;
//! use docker_wrapper::RedisTemplate;
//!
//! #[tokio::test]
//! async fn test_with_redis() -> Result<(), Box<dyn std::error::Error>> {
//!     // Container starts and waits for Redis to be ready
//!     let guard = ContainerGuard::new(RedisTemplate::new("test-redis"))
//!         .wait_for_ready(true)
//!         .start()
//!         .await?;
//!
//!     // Get connection string directly from guard
//!     let url = guard.connection_string();
//!     // Use Redis at: redis://localhost:6379
//!
//!     Ok(())
//!     // Container automatically stopped and removed here
//! }
//! ```
//!
//! ## Configuration Options
//!
//! ### Lifecycle Control
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = ContainerGuard::new(RedisTemplate::new("redis"))
//!     .stop_on_drop(true)      // Stop container on drop (default: true)
//!     .remove_on_drop(true)    // Remove container on drop (default: true)
//!     .start()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Debugging Failed Tests
//!
//! Keep containers running when tests fail for debugging:
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = ContainerGuard::new(RedisTemplate::new("redis"))
//!     .keep_on_panic(true)     // Keep container if test panics
//!     .capture_logs(true)      // Print container logs on panic
//!     .start()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Ready Checks
//!
//! Wait for the service to be ready before proceeding:
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Automatic wait during start
//! let guard = ContainerGuard::new(RedisTemplate::new("redis"))
//!     .wait_for_ready(true)
//!     .start()
//!     .await?;
//! // Redis is guaranteed ready here
//!
//! // Or wait manually later
//! let guard2 = ContainerGuard::new(RedisTemplate::new("redis2"))
//!     .start()
//!     .await?;
//! guard2.wait_for_ready().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Container Reuse
//!
//! Speed up local development by reusing running containers:
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = ContainerGuard::new(RedisTemplate::new("redis"))
//!     .reuse_if_running(true)  // Reuse existing container if found
//!     .remove_on_drop(false)   // Keep it for next test run
//!     .stop_on_drop(false)
//!     .start()
//!     .await?;
//!
//! if guard.was_reused() {
//!     println!("Reused existing container");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Network Support
//!
//! Attach containers to custom networks:
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = ContainerGuard::new(RedisTemplate::new("redis"))
//!     .with_network("my-test-network")  // Create and attach to network
//!     .remove_network_on_drop(true)     // Clean up network after test
//!     .start()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Fast Cleanup
//!
//! Use a short stop timeout for faster test cleanup:
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = ContainerGuard::new(RedisTemplate::new("redis"))
//!     .stop_timeout(Duration::from_secs(1))  // 1 second graceful shutdown
//!     .start()
//!     .await?;
//!
//! // Or immediate SIGKILL
//! let guard2 = ContainerGuard::new(RedisTemplate::new("redis2"))
//!     .stop_timeout(Duration::ZERO)
//!     .start()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Multi-Container Tests
//!
//! Use [`ContainerGuardSet`] for tests requiring multiple services:
//!
//! ```rust,no_run
//! use docker_wrapper::testing::ContainerGuardSet;
//! use docker_wrapper::RedisTemplate;
//!
//! #[tokio::test]
//! async fn test_multi_container() -> Result<(), Box<dyn std::error::Error>> {
//!     let guards = ContainerGuardSet::new()
//!         .with_network("test-network")    // Shared network for all containers
//!         .add(RedisTemplate::new("redis-primary").port(6379))
//!         .add(RedisTemplate::new("redis-replica").port(6380))
//!         .keep_on_panic(true)
//!         .start_all()
//!         .await?;
//!
//!     assert!(guards.contains("redis-primary"));
//!     assert!(guards.contains("redis-replica"));
//!     assert_eq!(guards.len(), 2);
//!
//!     // All containers cleaned up together
//!     Ok(())
//! }
//! ```
//!
//! ## Accessing Container Information
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = ContainerGuard::new(RedisTemplate::new("redis").port(6379))
//!     .start()
//!     .await?;
//!
//! // Connection string (for templates that support it)
//! let conn = guard.connection_string();
//!
//! // Access underlying template
//! let template = guard.template();
//!
//! // Get container ID
//! if let Some(id) = guard.container_id() {
//!     println!("Container ID: {}", id);
//! }
//!
//! // Query host port for a container port
//! let host_port = guard.host_port(6379).await?;
//!
//! // Get container logs
//! let logs = guard.logs().await?;
//!
//! // Check if running
//! let running = guard.is_running().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Common Patterns
//!
//! ### Test Fixtures
//!
//! Create reusable test fixtures:
//!
//! ```rust,no_run
//! use docker_wrapper::testing::ContainerGuard;
//! use docker_wrapper::RedisTemplate;
//! use docker_wrapper::template::TemplateError;
//!
//! async fn redis_fixture(name: &str) -> Result<ContainerGuard<RedisTemplate>, TemplateError> {
//!     ContainerGuard::new(RedisTemplate::new(name))
//!         .wait_for_ready(true)
//!         .keep_on_panic(true)
//!         .capture_logs(true)
//!         .start()
//!         .await
//! }
//!
//! #[tokio::test]
//! async fn test_using_fixture() -> Result<(), Box<dyn std::error::Error>> {
//!     let redis = redis_fixture("test-redis").await?;
//!     // Use redis...
//!     Ok(())
//! }
//! ```
//!
//! ### Unique Container Names
//!
//! Use UUIDs to avoid name conflicts in parallel tests:
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! fn unique_name(prefix: &str) -> String {
//!     format!("{}-{}", prefix, uuid::Uuid::new_v4())
//! }
//!
//! #[tokio::test]
//! async fn test_parallel_safe() -> Result<(), Box<dyn std::error::Error>> {
//!     let name = unique_name("redis");
//!     let guard = ContainerGuard::new(RedisTemplate::new(&name))
//!         .start()
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Manual Cleanup
//!
//! Trigger cleanup explicitly when needed:
//!
//! ```rust,no_run
//! # use docker_wrapper::testing::ContainerGuard;
//! # use docker_wrapper::RedisTemplate;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = ContainerGuard::new(RedisTemplate::new("redis"))
//!     .start()
//!     .await?;
//!
//! // Do some work...
//!
//! // Explicitly cleanup (idempotent - safe to call multiple times)
//! guard.cleanup().await?;
//!
//! // Drop will not try to clean up again
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flag
//!
//! This module requires the `testing` feature:
//!
//! ```toml
//! [dev-dependencies]
//! docker-wrapper = { version = "0.10", features = ["testing", "template-redis"] }
//! ```

use crate::command::DockerCommand;
use crate::template::{HasConnectionString, Template, TemplateError};
use crate::{
    LogsCommand, NetworkCreateCommand, NetworkRmCommand, PortCommand, RmCommand, StopCommand,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Options for controlling container lifecycle behavior.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct GuardOptions {
    /// Remove container on drop (default: true)
    pub remove_on_drop: bool,
    /// Stop container on drop (default: true)
    pub stop_on_drop: bool,
    /// Keep container running if test panics (default: false)
    pub keep_on_panic: bool,
    /// Capture container logs and print on panic (default: false)
    pub capture_logs: bool,
    /// Reuse existing container if already running (default: false)
    pub reuse_if_running: bool,
    /// Automatically wait for container to be ready after start (default: false)
    pub wait_for_ready: bool,
    /// Network to attach the container to (default: None)
    pub network: Option<String>,
    /// Create the network if it doesn't exist (default: true when network is set)
    pub create_network: bool,
    /// Remove the network on drop (default: false)
    pub remove_network_on_drop: bool,
    /// Timeout for stop operations during cleanup (default: None, uses Docker default)
    pub stop_timeout: Option<Duration>,
}

impl Default for GuardOptions {
    fn default() -> Self {
        Self {
            remove_on_drop: true,
            stop_on_drop: true,
            keep_on_panic: false,
            capture_logs: false,
            reuse_if_running: false,
            wait_for_ready: false,
            network: None,
            create_network: true,
            remove_network_on_drop: false,
            stop_timeout: None,
        }
    }
}

/// Builder for creating a [`ContainerGuard`] with custom options.
pub struct ContainerGuardBuilder<T: Template> {
    template: T,
    options: GuardOptions,
}

impl<T: Template> ContainerGuardBuilder<T> {
    /// Create a new builder with the given template.
    #[must_use]
    pub fn new(template: T) -> Self {
        Self {
            template,
            options: GuardOptions::default(),
        }
    }

    /// Set whether to remove the container on drop (default: true).
    #[must_use]
    pub fn remove_on_drop(mut self, remove: bool) -> Self {
        self.options.remove_on_drop = remove;
        self
    }

    /// Set whether to stop the container on drop (default: true).
    #[must_use]
    pub fn stop_on_drop(mut self, stop: bool) -> Self {
        self.options.stop_on_drop = stop;
        self
    }

    /// Set whether to keep the container running if the test panics (default: false).
    ///
    /// This is useful for debugging failed tests - you can inspect the container
    /// state after the test fails.
    #[must_use]
    pub fn keep_on_panic(mut self, keep: bool) -> Self {
        self.options.keep_on_panic = keep;
        self
    }

    /// Set whether to capture container logs and print them on panic (default: false).
    ///
    /// When enabled, container logs are buffered and printed to stderr if the
    /// test panics, making it easier to debug failures.
    #[must_use]
    pub fn capture_logs(mut self, capture: bool) -> Self {
        self.options.capture_logs = capture;
        self
    }

    /// Set whether to reuse an existing container if already running (default: false).
    ///
    /// This is useful for faster local development iteration - containers can
    /// be kept running between test runs.
    #[must_use]
    pub fn reuse_if_running(mut self, reuse: bool) -> Self {
        self.options.reuse_if_running = reuse;
        self
    }

    /// Set whether to automatically wait for the container to be ready after starting (default: false).
    ///
    /// When enabled, `start()` will not return until the container passes its
    /// readiness check. This is useful for tests that need to immediately connect
    /// to the service.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use docker_wrapper::testing::ContainerGuard;
    /// # use docker_wrapper::RedisTemplate;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let guard = ContainerGuard::new(RedisTemplate::new("test"))
    ///     .wait_for_ready(true)
    ///     .start()
    ///     .await?;
    /// // Container is guaranteed ready at this point
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn wait_for_ready(mut self, wait: bool) -> Self {
        self.options.wait_for_ready = wait;
        self
    }

    /// Attach the container to a Docker network.
    ///
    /// By default, the network will be created if it doesn't exist. Use
    /// `create_network(false)` to disable automatic network creation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use docker_wrapper::testing::ContainerGuard;
    /// # use docker_wrapper::RedisTemplate;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let guard = ContainerGuard::new(RedisTemplate::new("redis"))
    ///     .with_network("test-network")
    ///     .start()
    ///     .await?;
    /// // Container is attached to "test-network"
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.options.network = Some(network.into());
        self
    }

    /// Set whether to create the network if it doesn't exist (default: true).
    ///
    /// Only applies when a network is specified via `with_network()`.
    #[must_use]
    pub fn create_network(mut self, create: bool) -> Self {
        self.options.create_network = create;
        self
    }

    /// Set whether to remove the network on drop (default: false).
    ///
    /// This is useful for cleaning up test-specific networks. Only applies
    /// when a network is specified via `with_network()`.
    ///
    /// Note: The network removal will fail silently if other containers are
    /// still using it.
    #[must_use]
    pub fn remove_network_on_drop(mut self, remove: bool) -> Self {
        self.options.remove_network_on_drop = remove;
        self
    }

    /// Set the timeout for stop operations during cleanup (default: Docker default).
    ///
    /// This controls how long Docker waits for the container to stop gracefully
    /// before sending SIGKILL.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use docker_wrapper::testing::ContainerGuard;
    /// # use docker_wrapper::RedisTemplate;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Fast cleanup with 1 second timeout
    /// let guard = ContainerGuard::new(RedisTemplate::new("redis"))
    ///     .stop_timeout(Duration::from_secs(1))
    ///     .start()
    ///     .await?;
    ///
    /// // Immediate SIGKILL with zero timeout
    /// let guard = ContainerGuard::new(RedisTemplate::new("redis2"))
    ///     .stop_timeout(Duration::ZERO)
    ///     .start()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn stop_timeout(mut self, timeout: Duration) -> Self {
        self.options.stop_timeout = Some(timeout);
        self
    }

    /// Start the container and return a guard that manages its lifecycle.
    ///
    /// If `reuse_if_running` is enabled and a container is already running,
    /// it will be reused instead of starting a new one.
    ///
    /// If `wait_for_ready` is enabled, this method will block until the
    /// container passes its readiness check.
    ///
    /// If a network is specified via `with_network()`, the container will be
    /// attached to that network. The network will be created if it doesn't
    /// exist (unless `create_network(false)` was called).
    ///
    /// # Errors
    ///
    /// Returns an error if the container fails to start or the readiness check times out.
    pub async fn start(mut self) -> Result<ContainerGuard<T>, TemplateError> {
        let wait_for_ready = self.options.wait_for_ready;
        let mut network_created = false;

        // Create network if specified and create_network is enabled
        if let Some(ref network) = self.options.network {
            if self.options.create_network {
                // Try to create the network (ignore errors if it already exists)
                let result = NetworkCreateCommand::new(network)
                    .driver("bridge")
                    .execute()
                    .await;

                // Track if we successfully created it (for cleanup purposes)
                network_created = result.is_ok();
            }

            // Set the network on the template
            self.template.config_mut().network = Some(network.clone());
        }

        // Check if we should reuse an existing container
        if self.options.reuse_if_running {
            if let Ok(true) = self.template.is_running().await {
                let guard = ContainerGuard {
                    template: self.template,
                    container_id: None, // We don't have the ID for reused containers
                    options: self.options,
                    was_reused: true,
                    network_created,
                    cleaned_up: Arc::new(AtomicBool::new(false)),
                };

                // Wait for ready if configured (even for reused containers)
                if wait_for_ready {
                    guard.wait_for_ready().await?;
                }

                return Ok(guard);
            }
        }

        // Start the container
        let container_id = self.template.start_and_wait().await?;

        let guard = ContainerGuard {
            template: self.template,
            container_id: Some(container_id),
            options: self.options,
            was_reused: false,
            network_created,
            cleaned_up: Arc::new(AtomicBool::new(false)),
        };

        // Wait for ready if configured
        if wait_for_ready {
            guard.wait_for_ready().await?;
        }

        Ok(guard)
    }
}

/// RAII guard for automatic container lifecycle management.
///
/// When this guard is dropped, the container is automatically stopped and
/// removed (unless configured otherwise via [`ContainerGuardBuilder`]).
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::testing::ContainerGuard;
/// use docker_wrapper::RedisTemplate;
///
/// #[tokio::test]
/// async fn test_example() -> Result<(), Box<dyn std::error::Error>> {
///     let guard = ContainerGuard::new(RedisTemplate::new("test"))
///         .keep_on_panic(true)  // Keep container for debugging if test fails
///         .capture_logs(true)   // Print logs on failure
///         .start()
///         .await?;
///
///     // Container is automatically cleaned up when guard goes out of scope
///     Ok(())
/// }
/// ```
pub struct ContainerGuard<T: Template> {
    template: T,
    container_id: Option<String>,
    options: GuardOptions,
    was_reused: bool,
    network_created: bool,
    cleaned_up: Arc<AtomicBool>,
}

impl<T: Template> ContainerGuard<T> {
    /// Create a new builder for a container guard.
    ///
    /// Note: This returns a builder, not a `ContainerGuard`. Call `.start().await`
    /// on the builder to create the guard.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(template: T) -> ContainerGuardBuilder<T> {
        ContainerGuardBuilder::new(template)
    }

    /// Get a reference to the underlying template.
    #[must_use]
    pub fn template(&self) -> &T {
        &self.template
    }

    /// Get the container ID, if available.
    ///
    /// This may be `None` if the container was reused from a previous run.
    #[must_use]
    pub fn container_id(&self) -> Option<&str> {
        self.container_id.as_deref()
    }

    /// Check if this guard is reusing an existing container.
    #[must_use]
    pub fn was_reused(&self) -> bool {
        self.was_reused
    }

    /// Get the network name, if one was configured.
    #[must_use]
    pub fn network(&self) -> Option<&str> {
        self.options.network.as_deref()
    }

    /// Check if the container is currently running.
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker command fails.
    pub async fn is_running(&self) -> Result<bool, TemplateError> {
        self.template.is_running().await
    }

    /// Wait for the container to be ready.
    ///
    /// This calls the underlying template's readiness check. The exact behavior
    /// depends on the template implementation - for example, Redis templates
    /// wait for a successful PING response.
    ///
    /// # Errors
    ///
    /// Returns an error if the readiness check times out or the Docker command fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use docker_wrapper::testing::ContainerGuard;
    /// # use docker_wrapper::RedisTemplate;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let guard = ContainerGuard::new(RedisTemplate::new("test"))
    ///     .start()
    ///     .await?;
    ///
    /// // Wait for Redis to be ready to accept connections
    /// guard.wait_for_ready().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_ready(&self) -> Result<(), TemplateError> {
        self.template.wait_for_ready().await
    }

    /// Get the host port mapped to a container port.
    ///
    /// This is useful when using dynamic port allocation - Docker assigns
    /// a random available host port which you can query with this method.
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker command fails or no port mapping is found.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use docker_wrapper::testing::ContainerGuard;
    /// # use docker_wrapper::RedisTemplate;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let guard = ContainerGuard::new(RedisTemplate::new("test"))
    ///     .start()
    ///     .await?;
    ///
    /// let host_port = guard.host_port(6379).await?;
    /// println!("Redis available at localhost:{}", host_port);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn host_port(&self, container_port: u16) -> Result<u16, TemplateError> {
        let container_name = self.template.config().name.clone();
        let result = PortCommand::new(&container_name)
            .port(container_port)
            .run()
            .await
            .map_err(TemplateError::DockerError)?;

        // Return the first matching port mapping
        if let Some(mapping) = result.port_mappings.first() {
            return Ok(mapping.host_port);
        }

        Err(TemplateError::InvalidConfig(format!(
            "No host port mapping found for container port {container_port}"
        )))
    }

    /// Get the container logs.
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker command fails.
    pub async fn logs(&self) -> Result<String, TemplateError> {
        let container_name = self.template.config().name.clone();
        let result = LogsCommand::new(&container_name)
            .execute()
            .await
            .map_err(TemplateError::DockerError)?;

        Ok(format!("{}{}", result.stdout, result.stderr))
    }

    /// Manually stop the container.
    ///
    /// The container will still be removed on drop if `remove_on_drop` is enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker command fails.
    pub async fn stop(&self) -> Result<(), TemplateError> {
        self.template.stop().await
    }

    /// Manually clean up the container (stop and remove).
    ///
    /// After calling this, the drop implementation will not attempt cleanup again.
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker commands fail.
    pub async fn cleanup(&self) -> Result<(), TemplateError> {
        if self.cleaned_up.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already cleaned up
        }

        if self.options.stop_on_drop {
            let _ = self.template.stop().await;
        }
        if self.options.remove_on_drop {
            let _ = self.template.remove().await;
        }
        Ok(())
    }
}

impl<T: Template + HasConnectionString> ContainerGuard<T> {
    /// Get the connection string for the underlying service.
    ///
    /// This is a convenience method that delegates to the template's
    /// `connection_string()` implementation. The format depends on the
    /// service type (e.g., `redis://host:port` for Redis).
    ///
    /// This method is only available for templates that implement
    /// [`HasConnectionString`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use docker_wrapper::testing::ContainerGuard;
    /// # use docker_wrapper::RedisTemplate;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let guard = ContainerGuard::new(RedisTemplate::new("redis").port(6379))
    ///     .start()
    ///     .await?;
    ///
    /// // Direct access to connection string
    /// let conn = guard.connection_string();
    /// // Instead of: guard.template().connection_string()
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn connection_string(&self) -> String {
        self.template.connection_string()
    }
}

impl<T: Template> Drop for ContainerGuard<T> {
    fn drop(&mut self) {
        // Skip cleanup if already done
        if self.cleaned_up.load(Ordering::SeqCst) {
            return;
        }

        // Skip cleanup for reused containers if not configured to clean them
        if self.was_reused && !self.options.remove_on_drop {
            return;
        }

        // Check if we're panicking
        let panicking = std::thread::panicking();

        if panicking && self.options.keep_on_panic {
            let name = &self.template.config().name;
            eprintln!("[ContainerGuard] Test panicked, keeping container '{name}' for debugging");

            if self.options.capture_logs {
                // Try to get logs - spawn a thread to avoid runtime conflicts
                let container_name = self.template.config().name.clone();
                let _ = std::thread::spawn(move || {
                    if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        if let Ok(result) =
                            rt.block_on(async { LogsCommand::new(&container_name).execute().await })
                        {
                            let logs = format!("{}{}", result.stdout, result.stderr);
                            eprintln!("[ContainerGuard] Container logs for '{container_name}':");
                            eprintln!("{logs}");
                        }
                    }
                })
                .join();
            }
            return;
        }

        // Mark as cleaned up
        self.cleaned_up.store(true, Ordering::SeqCst);

        // Perform cleanup - need to spawn a runtime since Drop isn't async
        let should_stop = self.options.stop_on_drop;
        let should_remove = self.options.remove_on_drop;
        let should_remove_network = self.options.remove_network_on_drop && self.network_created;
        let container_name = self.template.config().name.clone();
        let network_name = self.options.network.clone();
        let stop_timeout = self.options.stop_timeout;

        if !should_stop && !should_remove && !should_remove_network {
            return;
        }

        // Perform cleanup - try to use existing runtime if available,
        // otherwise create a new one (for non-async contexts)
        if tokio::runtime::Handle::try_current().is_ok() {
            // We're in an async context - use spawn_blocking to avoid blocking the runtime
            let container_name_clone = container_name.clone();
            let network_name_clone = network_name.clone();
            let _ = std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create runtime for cleanup");
                rt.block_on(async {
                    if should_stop {
                        let mut cmd = StopCommand::new(&container_name_clone);
                        if let Some(timeout) = stop_timeout {
                            cmd = cmd.timeout_duration(timeout);
                        }
                        let _ = cmd.execute().await;
                    }
                    if should_remove {
                        let _ = RmCommand::new(&container_name_clone).force().run().await;
                    }
                    // Remove network after container (network must be empty)
                    if should_remove_network {
                        if let Some(ref network) = network_name_clone {
                            let _ = NetworkRmCommand::new(network).execute().await;
                        }
                    }
                });
            })
            .join();
        } else {
            // Not in an async context - create a new runtime
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(async {
                    if should_stop {
                        let mut cmd = StopCommand::new(&container_name);
                        if let Some(timeout) = stop_timeout {
                            cmd = cmd.timeout_duration(timeout);
                        }
                        let _ = cmd.execute().await;
                    }
                    if should_remove {
                        let _ = RmCommand::new(&container_name).force().run().await;
                    }
                    // Remove network after container (network must be empty)
                    if should_remove_network {
                        if let Some(ref network) = network_name {
                            let _ = NetworkRmCommand::new(network).execute().await;
                        }
                    }
                });
            }
        }
    }
}

/// A type-erased container guard entry for use in `ContainerGuardSet`.
///
/// This allows storing guards with different template types in the same collection.
#[allow(dead_code)]
struct GuardEntry {
    /// Container name for lookup
    name: String,
    /// Cleanup function to stop and remove the container
    cleanup_fn: Box<dyn FnOnce() + Send>,
}

/// Options for `ContainerGuardSet`.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct GuardSetOptions {
    /// Shared network for all containers
    pub network: Option<String>,
    /// Create the network if it doesn't exist (default: true)
    pub create_network: bool,
    /// Remove the network on drop (default: true when network is set)
    pub remove_network_on_drop: bool,
    /// Keep containers running if test panics (default: false)
    pub keep_on_panic: bool,
    /// Wait for each container to be ready after starting (default: true)
    pub wait_for_ready: bool,
}

impl GuardSetOptions {
    fn new() -> Self {
        Self {
            network: None,
            create_network: true,
            remove_network_on_drop: true,
            keep_on_panic: false,
            wait_for_ready: true,
        }
    }
}

/// A pending template entry waiting to be started.
struct PendingEntry<T: Template + 'static> {
    template: T,
}

/// Type-erased pending entry trait.
trait PendingEntryTrait: Send {
    /// Get the container name
    fn name(&self) -> String;
    /// Start the container and return a cleanup function
    fn start(
        self: Box<Self>,
        network: Option<String>,
        wait_for_ready: bool,
        keep_on_panic: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GuardEntry, TemplateError>> + Send>,
    >;
}

impl<T: Template + 'static> PendingEntryTrait for PendingEntry<T> {
    fn name(&self) -> String {
        self.template.config().name.clone()
    }

    fn start(
        self: Box<Self>,
        network: Option<String>,
        wait_for_ready: bool,
        keep_on_panic: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<GuardEntry, TemplateError>> + Send>,
    > {
        Box::pin(async move {
            let mut template = self.template;
            let name = template.config().name.clone();

            // Set network if provided
            if let Some(ref net) = network {
                template.config_mut().network = Some(net.clone());
            }

            // Start the container
            template.start_and_wait().await?;

            // Wait for ready if configured
            if wait_for_ready {
                template.wait_for_ready().await?;
            }

            // Create cleanup function
            let cleanup_name = name.clone();
            let cleanup_fn: Box<dyn FnOnce() + Send> = Box::new(move || {
                // Check if panicking and should keep
                if std::thread::panicking() && keep_on_panic {
                    eprintln!(
                        "[ContainerGuardSet] Test panicked, keeping container '{cleanup_name}' for debugging"
                    );
                    return;
                }

                // Perform cleanup in a new runtime
                let _ = std::thread::spawn(move || {
                    if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        rt.block_on(async {
                            let _ = StopCommand::new(&cleanup_name).execute().await;
                            let _ = RmCommand::new(&cleanup_name).force().run().await;
                        });
                    }
                })
                .join();
            });

            Ok(GuardEntry { name, cleanup_fn })
        })
    }
}

/// Builder for creating a [`ContainerGuardSet`].
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::testing::ContainerGuardSet;
/// use docker_wrapper::RedisTemplate;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let guards = ContainerGuardSet::new()
///     .with_network("test-network")
///     .add(RedisTemplate::new("redis-1"))
///     .add(RedisTemplate::new("redis-2"))
///     .start_all()
///     .await?;
///
/// // Access by name
/// assert!(guards.contains("redis-1"));
/// # Ok(())
/// # }
/// ```
pub struct ContainerGuardSetBuilder {
    entries: Vec<Box<dyn PendingEntryTrait>>,
    options: GuardSetOptions,
}

impl ContainerGuardSetBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            options: GuardSetOptions::new(),
        }
    }

    /// Add a template to the set.
    ///
    /// The container name from the template's config is used as the key for lookup.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add<T: Template + 'static>(mut self, template: T) -> Self {
        self.entries.push(Box::new(PendingEntry { template }));
        self
    }

    /// Set a shared network for all containers.
    ///
    /// The network will be created if it doesn't exist (unless `create_network(false)` is called).
    #[must_use]
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.options.network = Some(network.into());
        self
    }

    /// Set whether to create the network if it doesn't exist (default: true).
    #[must_use]
    pub fn create_network(mut self, create: bool) -> Self {
        self.options.create_network = create;
        self
    }

    /// Set whether to remove the network on drop (default: true when network is set).
    #[must_use]
    pub fn remove_network_on_drop(mut self, remove: bool) -> Self {
        self.options.remove_network_on_drop = remove;
        self
    }

    /// Set whether to keep containers running if the test panics (default: false).
    #[must_use]
    pub fn keep_on_panic(mut self, keep: bool) -> Self {
        self.options.keep_on_panic = keep;
        self
    }

    /// Set whether to wait for each container to be ready (default: true).
    #[must_use]
    pub fn wait_for_ready(mut self, wait: bool) -> Self {
        self.options.wait_for_ready = wait;
        self
    }

    /// Start all containers and return a guard set.
    ///
    /// Containers are started sequentially in the order they were added.
    ///
    /// # Errors
    ///
    /// Returns an error if any container fails to start. Containers that were
    /// successfully started before the failure will be cleaned up.
    pub async fn start_all(self) -> Result<ContainerGuardSet, TemplateError> {
        let mut network_created = false;

        // Create network if needed
        if let Some(ref network) = self.options.network {
            if self.options.create_network {
                let result = NetworkCreateCommand::new(network)
                    .driver("bridge")
                    .execute()
                    .await;
                network_created = result.is_ok();
            }
        }

        let mut guards: Vec<GuardEntry> = Vec::new();
        let mut names: HashMap<String, usize> = HashMap::new();

        // Start each container
        for entry in self.entries {
            let name = entry.name();
            match entry
                .start(
                    self.options.network.clone(),
                    self.options.wait_for_ready,
                    self.options.keep_on_panic,
                )
                .await
            {
                Ok(guard) => {
                    names.insert(name, guards.len());
                    guards.push(guard);
                }
                Err(e) => {
                    // Clean up already-started containers on failure
                    for guard in guards {
                        (guard.cleanup_fn)();
                    }
                    // Clean up network if we created it
                    if network_created {
                        if let Some(ref network) = self.options.network {
                            let net = network.clone();
                            let _ = std::thread::spawn(move || {
                                if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                                    .enable_all()
                                    .build()
                                {
                                    rt.block_on(async {
                                        let _ = NetworkRmCommand::new(&net).execute().await;
                                    });
                                }
                            })
                            .join();
                        }
                    }
                    return Err(e);
                }
            }
        }

        Ok(ContainerGuardSet {
            guards,
            names,
            options: self.options,
            network_created,
        })
    }
}

impl Default for ContainerGuardSetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages multiple containers as a group with coordinated lifecycle.
///
/// All containers are cleaned up when the set is dropped. This is useful for
/// integration tests that require multiple services.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::testing::ContainerGuardSet;
/// use docker_wrapper::RedisTemplate;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let guards = ContainerGuardSet::new()
///     .with_network("test-network")
///     .add(RedisTemplate::new("redis"))
///     .keep_on_panic(true)
///     .start_all()
///     .await?;
///
/// // Check if container exists
/// assert!(guards.contains("redis"));
///
/// // Get container names
/// for name in guards.names() {
///     println!("Container: {}", name);
/// }
/// # Ok(())
/// # }
/// ```
pub struct ContainerGuardSet {
    guards: Vec<GuardEntry>,
    names: HashMap<String, usize>,
    options: GuardSetOptions,
    network_created: bool,
}

impl ContainerGuardSet {
    /// Create a new builder for a container guard set.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> ContainerGuardSetBuilder {
        ContainerGuardSetBuilder::new()
    }

    /// Check if a container with the given name exists in the set.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    /// Get an iterator over container names in the set.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.keys().map(String::as_str)
    }

    /// Get the number of containers in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.guards.len()
    }

    /// Check if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }

    /// Get the shared network name, if one was configured.
    #[must_use]
    pub fn network(&self) -> Option<&str> {
        self.options.network.as_deref()
    }
}

impl Default for ContainerGuardSet {
    fn default() -> Self {
        Self {
            guards: Vec::new(),
            names: HashMap::new(),
            options: GuardSetOptions::new(),
            network_created: false,
        }
    }
}

impl Drop for ContainerGuardSet {
    fn drop(&mut self) {
        // Clean up all containers
        for guard in self.guards.drain(..) {
            (guard.cleanup_fn)();
        }

        // Clean up network if we created it
        if self.network_created && self.options.remove_network_on_drop {
            if let Some(ref network) = self.options.network {
                let net = network.clone();
                let _ = std::thread::spawn(move || {
                    if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        rt.block_on(async {
                            let _ = NetworkRmCommand::new(&net).execute().await;
                        });
                    }
                })
                .join();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_options_default() {
        let opts = GuardOptions::default();
        assert!(opts.remove_on_drop);
        assert!(opts.stop_on_drop);
        assert!(!opts.keep_on_panic);
        assert!(!opts.capture_logs);
        assert!(!opts.reuse_if_running);
        assert!(!opts.wait_for_ready);
        assert!(opts.network.is_none());
        assert!(opts.create_network);
        assert!(!opts.remove_network_on_drop);
        assert!(opts.stop_timeout.is_none());
    }

    #[test]
    fn test_builder_options() {
        // We can't easily test the builder without a real template,
        // but we can at least verify the module compiles
    }
}
