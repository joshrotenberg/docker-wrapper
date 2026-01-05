//! Docker template system for common container configurations
//!
//! This module provides pre-configured templates for common Docker setups,
//! making it easy to spin up development environments with best practices.

#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::inefficient_to_string)]

use crate::{DockerCommand, RunCommand};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, error, info, trace, warn};

// Redis templates
#[cfg(any(
    feature = "template-redis",
    feature = "template-redis-cluster",
    feature = "template-redis-enterprise",
))]
pub mod redis;

// Database templates
#[cfg(any(
    feature = "template-postgres",
    feature = "template-mysql",
    feature = "template-mongodb"
))]
pub mod database;

// Web server templates
#[cfg(feature = "template-nginx")]
pub mod web;

/// Result type for template operations
pub type Result<T> = std::result::Result<T, TemplateError>;

/// Template-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// Docker command execution failed
    #[error("Docker command failed: {0}")]
    DockerError(#[from] crate::Error),

    /// Invalid template configuration provided
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Attempted to start a template that is already running
    #[error("Template already running: {0}")]
    AlreadyRunning(String),

    /// Attempted to operate on a template that is not running
    #[error("Template not running: {0}")]
    NotRunning(String),
}

/// Configuration for a Docker template
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// Container name
    pub name: String,

    /// Image to use
    pub image: String,

    /// Image tag
    pub tag: String,

    /// Port mappings (host -> container)
    pub ports: Vec<(u16, u16)>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Volume mounts
    pub volumes: Vec<VolumeMount>,

    /// Network to connect to
    pub network: Option<String>,

    /// Health check configuration
    pub health_check: Option<HealthCheck>,

    /// Whether to remove container on stop
    pub auto_remove: bool,

    /// Memory limit
    pub memory_limit: Option<String>,

    /// CPU limit
    pub cpu_limit: Option<String>,

    /// Platform specification (e.g., "linux/amd64", "linux/arm64")
    pub platform: Option<String>,
}

/// Volume mount configuration
#[derive(Debug, Clone)]
pub struct VolumeMount {
    /// Source (host path or volume name)
    pub source: String,

    /// Target (container path)
    pub target: String,

    /// Read-only mount
    pub read_only: bool,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Command to run for health check
    pub test: Vec<String>,

    /// Time between checks
    pub interval: String,

    /// Maximum time to wait for check
    pub timeout: String,

    /// Number of retries before marking unhealthy
    pub retries: i32,

    /// Start period for container initialization
    pub start_period: String,
}

/// Trait for Docker container templates
#[async_trait]
pub trait Template: Send + Sync {
    /// Get the template name
    fn name(&self) -> &str;

    /// Get the template configuration
    fn config(&self) -> &TemplateConfig;

    /// Get a mutable reference to the configuration
    fn config_mut(&mut self) -> &mut TemplateConfig;

    /// Build the RunCommand for this template
    fn build_command(&self) -> RunCommand {
        let config = self.config();
        let mut cmd = RunCommand::new(format!("{}:{}", config.image, config.tag))
            .name(&config.name)
            .detach();

        // Add port mappings
        for (host, container) in &config.ports {
            cmd = cmd.port(*host, *container);
        }

        // Add environment variables
        for (key, value) in &config.env {
            cmd = cmd.env(key, value);
        }

        // Add volume mounts
        for mount in &config.volumes {
            if mount.read_only {
                cmd = cmd.volume_ro(&mount.source, &mount.target);
            } else {
                cmd = cmd.volume(&mount.source, &mount.target);
            }
        }

        // Add network
        if let Some(network) = &config.network {
            cmd = cmd.network(network);
        }

        // Add health check
        if let Some(health) = &config.health_check {
            cmd = cmd
                .health_cmd(&health.test.join(" "))
                .health_interval(&health.interval)
                .health_timeout(&health.timeout)
                .health_retries(health.retries)
                .health_start_period(&health.start_period);
        }

        // Add resource limits
        if let Some(memory) = &config.memory_limit {
            cmd = cmd.memory(memory);
        }

        if let Some(cpu) = &config.cpu_limit {
            cmd = cmd.cpus(cpu);
        }

        // Add platform if specified
        if let Some(platform) = &config.platform {
            cmd = cmd.platform(platform);
        }

        // Auto-remove
        if config.auto_remove {
            cmd = cmd.remove();
        }

        cmd
    }

    /// Start the container with this template
    async fn start(&self) -> Result<String> {
        let config = self.config();
        info!(
            template = %config.name,
            image = %config.image,
            tag = %config.tag,
            "starting container from template"
        );

        let output = self.build_command().execute().await.map_err(|e| {
            error!(
                template = %config.name,
                error = %e,
                "failed to start container"
            );
            e
        })?;

        info!(
            template = %config.name,
            container_id = %output.0,
            "container started successfully"
        );

        Ok(output.0)
    }

    /// Start the container and wait for it to be ready
    async fn start_and_wait(&self) -> Result<String> {
        let config = self.config();
        info!(
            template = %config.name,
            "starting container and waiting for ready"
        );

        let container_id = self.start().await?;
        self.wait_for_ready().await?;

        info!(
            template = %config.name,
            container_id = %container_id,
            "container started and ready"
        );

        Ok(container_id)
    }

    /// Stop the container
    async fn stop(&self) -> Result<()> {
        use crate::StopCommand;

        let name = self.config().name.as_str();
        info!(template = %name, "stopping container");

        StopCommand::new(name).execute().await.map_err(|e| {
            error!(template = %name, error = %e, "failed to stop container");
            e
        })?;

        debug!(template = %name, "container stopped");
        Ok(())
    }

    /// Remove the container
    async fn remove(&self) -> Result<()> {
        use crate::RmCommand;

        let name = self.config().name.as_str();
        info!(template = %name, "removing container");

        RmCommand::new(name)
            .force()
            .volumes()
            .execute()
            .await
            .map_err(|e| {
                error!(template = %name, error = %e, "failed to remove container");
                e
            })?;

        debug!(template = %name, "container removed");
        Ok(())
    }

    /// Check if the container is running
    async fn is_running(&self) -> Result<bool> {
        use crate::PsCommand;

        let name = &self.config().name;

        let output = PsCommand::new()
            .filter(format!("name={name}"))
            .quiet()
            .execute()
            .await?;

        // In quiet mode, check if stdout contains any container IDs
        let running = !output.stdout.trim().is_empty();
        trace!(template = %name, running = running, "checked container running status");

        Ok(running)
    }

    /// Get container logs
    async fn logs(&self, follow: bool, tail: Option<&str>) -> Result<crate::CommandOutput> {
        use crate::LogsCommand;

        let mut cmd = LogsCommand::new(&self.config().name);

        if follow {
            cmd = cmd.follow();
        }

        if let Some(lines) = tail {
            cmd = cmd.tail(lines);
        }

        cmd.execute().await.map_err(Into::into)
    }

    /// Execute a command in the running container
    async fn exec(&self, command: Vec<&str>) -> Result<crate::ExecOutput> {
        use crate::ExecCommand;

        let cmd_vec: Vec<String> = command.iter().map(|s| s.to_string()).collect();
        let cmd = ExecCommand::new(&self.config().name, cmd_vec);

        cmd.execute().await.map_err(Into::into)
    }

    /// Wait for the container to be ready
    ///
    /// This method will wait for the container to pass its health checks
    /// or reach a ready state. The default implementation waits for the
    /// container to be running and healthy (if health checks are configured).
    ///
    /// Templates can override this to provide custom readiness checks.
    #[allow(clippy::too_many_lines)]
    async fn wait_for_ready(&self) -> Result<()> {
        use std::time::Duration;
        use tokio::time::{sleep, timeout, Instant};

        let name = &self.config().name;
        let has_health_check = self.config().health_check.is_some();

        // Default timeout of 60 seconds (increased for slower systems/Windows)
        let wait_timeout = Duration::from_secs(60);
        let check_interval = Duration::from_millis(500);

        info!(
            template = %name,
            timeout_secs = wait_timeout.as_secs(),
            has_health_check = has_health_check,
            "waiting for container to be ready"
        );

        let start_time = Instant::now();
        let mut check_count = 0u32;

        let result = timeout(wait_timeout, async {
            loop {
                check_count += 1;

                // Check if container is running - keep retrying if not yet started
                // Don't fail immediately as the container may still be starting up
                let running = self.is_running().await.unwrap_or(false);
                if !running {
                    trace!(
                        template = %name,
                        check = check_count,
                        "container not yet running, waiting"
                    );
                    sleep(check_interval).await;
                    continue;
                }

                // If there's a health check configured, wait for it
                if has_health_check {
                    use crate::InspectCommand;

                    if let Ok(inspect) = InspectCommand::new(name).execute().await {
                        // Check health status in the inspect output
                        if let Ok(containers) =
                            serde_json::from_str::<serde_json::Value>(&inspect.stdout)
                        {
                            if let Some(first) = containers.as_array().and_then(|arr| arr.first()) {
                                if let Some(state) = first.get("State") {
                                    if let Some(health) = state.get("Health") {
                                        if let Some(status) =
                                            health.get("Status").and_then(|s| s.as_str())
                                        {
                                            trace!(
                                                template = %name,
                                                check = check_count,
                                                health_status = %status,
                                                "health check status"
                                            );

                                            if status == "healthy" {
                                                #[allow(clippy::cast_possible_truncation)]
                                                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                                                debug!(
                                                    template = %name,
                                                    checks = check_count,
                                                    elapsed_ms = elapsed_ms,
                                                    "container healthy"
                                                );
                                                return Ok(());
                                            } else if status == "unhealthy" {
                                                warn!(
                                                    template = %name,
                                                    "container reported unhealthy, continuing to wait"
                                                );
                                            }
                                        }
                                    } else if let Some(running) =
                                        state.get("Running").and_then(|r| r.as_bool())
                                    {
                                        // No health check configured, just check if running
                                        if running {
                                            #[allow(clippy::cast_possible_truncation)]
                                            let elapsed_ms = start_time.elapsed().as_millis() as u64;
                                            debug!(
                                                template = %name,
                                                checks = check_count,
                                                elapsed_ms = elapsed_ms,
                                                "container running (no health check)"
                                            );
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // No health check, just ensure it's running
                    #[allow(clippy::cast_possible_truncation)]
                    let elapsed_ms = start_time.elapsed().as_millis() as u64;
                    debug!(
                        template = %name,
                        checks = check_count,
                        elapsed_ms = elapsed_ms,
                        "container running (no health check configured)"
                    );
                    return Ok(());
                }

                sleep(check_interval).await;
            }
        })
        .await;

        if let Ok(inner) = result {
            inner
        } else {
            error!(
                template = %name,
                timeout_secs = wait_timeout.as_secs(),
                checks = check_count,
                "container failed to become ready within timeout"
            );
            Err(TemplateError::InvalidConfig(format!(
                "Container {name} failed to become ready within timeout"
            )))
        }
    }
}

/// Builder for creating custom templates
pub struct TemplateBuilder {
    config: TemplateConfig,
}

impl TemplateBuilder {
    /// Create a new template builder
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            config: TemplateConfig {
                name: name.into(),
                image: image.into(),
                tag: "latest".to_string(),
                ports: Vec::new(),
                env: HashMap::new(),
                volumes: Vec::new(),
                network: None,
                health_check: None,
                auto_remove: false,
                memory_limit: None,
                cpu_limit: None,
                platform: None,
            },
        }
    }

    /// Set the image tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.config.tag = tag.into();
        self
    }

    /// Add a port mapping
    pub fn port(mut self, host: u16, container: u16) -> Self {
        self.config.ports.push((host, container));
        self
    }

    /// Add an environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env.insert(key.into(), value.into());
        self
    }

    /// Add a volume mount
    pub fn volume(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: source.into(),
            target: target.into(),
            read_only: false,
        });
        self
    }

    /// Add a read-only volume mount
    pub fn volume_ro(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.config.volumes.push(VolumeMount {
            source: source.into(),
            target: target.into(),
            read_only: true,
        });
        self
    }

    /// Set the network
    pub fn network(mut self, network: impl Into<String>) -> Self {
        self.config.network = Some(network.into());
        self
    }

    /// Enable auto-remove
    pub fn auto_remove(mut self) -> Self {
        self.config.auto_remove = true;
        self
    }

    /// Set memory limit
    pub fn memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.memory_limit = Some(limit.into());
        self
    }

    /// Set CPU limit
    pub fn cpu_limit(mut self, limit: impl Into<String>) -> Self {
        self.config.cpu_limit = Some(limit.into());
        self
    }

    /// Build into a custom template
    pub fn build(self) -> CustomTemplate {
        CustomTemplate {
            config: self.config,
        }
    }
}

/// A custom template created from `TemplateBuilder`
pub struct CustomTemplate {
    config: TemplateConfig,
}

#[async_trait]
impl Template for CustomTemplate {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &TemplateConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut TemplateConfig {
        &mut self.config
    }
}

// Compatibility re-exports for backward compatibility
// These allow users to still import directly from template::
#[cfg(feature = "template-redis")]
pub use redis::RedisTemplate;

#[cfg(feature = "template-redis-cluster")]
pub use redis::{ClusterInfo, NodeInfo, NodeRole, RedisClusterConnection, RedisClusterTemplate};

#[cfg(feature = "template-postgres")]
pub use database::postgres::{PostgresConnectionString, PostgresTemplate};

#[cfg(feature = "template-mysql")]
pub use database::mysql::{MysqlConnectionString, MysqlTemplate};

#[cfg(feature = "template-mongodb")]
pub use database::mongodb::{MongodbConnectionString, MongodbTemplate};

#[cfg(feature = "template-nginx")]
pub use web::nginx::NginxTemplate;
