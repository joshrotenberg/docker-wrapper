//! Testing utilities for container lifecycle management.
//!
//! This module provides [`ContainerGuard`], an RAII wrapper that automatically
//! manages container lifecycle. When the guard goes out of scope, containers
//! are stopped and removed automatically.
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::testing::ContainerGuard;
//! use docker_wrapper::RedisTemplate;
//!
//! #[tokio::test]
//! async fn test_redis() -> Result<(), Box<dyn std::error::Error>> {
//!     let guard = ContainerGuard::new(RedisTemplate::new("test-redis"))
//!         .start()
//!         .await?;
//!
//!     // Use the container...
//!     let url = guard.template().connection_url();
//!
//!     Ok(())
//!     // Container automatically stopped and removed here
//! }
//! ```

use crate::command::DockerCommand;
use crate::template::{Template, TemplateError};
use crate::{
    LogsCommand, NetworkCreateCommand, NetworkRmCommand, PortCommand, RmCommand, StopCommand,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
                        let _ = StopCommand::new(&container_name_clone).execute().await;
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
                        let _ = StopCommand::new(&container_name).execute().await;
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
    }

    #[test]
    fn test_builder_options() {
        // We can't easily test the builder without a real template,
        // but we can at least verify the module compiles
    }
}
