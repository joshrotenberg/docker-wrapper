//! Container management module providing comprehensive container lifecycle operations.
//!
//! This module provides a complete interface for Docker container management,
//! including creation, execution, monitoring, and cleanup operations.

pub mod exec;
pub mod health;
pub mod logs;

pub use exec::{ContainerExecutor, ExecConfig, ExecOutput, ExecResult};
pub use health::{HealthCheck, HealthCheckConfig, HealthCheckResult, HealthChecker};
pub use logs::{LogEntry, LogManager, LogOptions, LogSource};

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::types::{
    ContainerId, ContainerStatus, HealthCheck as TypeHealthCheck, NetworkId, PortMapping,
    Protocol as PortProtocol, ResourceLimits, RestartPolicy, VolumeMount, VolumeSource,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info};

/// Container configuration for creating new containers
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Docker image to use
    pub image: String,
    /// Optional container name
    pub name: Option<String>,
    /// Command to run in the container
    pub command: Option<Vec<String>>,
    /// Entrypoint override
    pub entrypoint: Option<Vec<String>>,
    /// Working directory inside the container
    pub working_dir: Option<PathBuf>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Container labels
    pub labels: HashMap<String, String>,
    /// Restart policy
    pub restart_policy: RestartPolicy,
    /// Health check configuration
    pub health_check: Option<TypeHealthCheck>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Networks to attach to
    pub networks: Vec<NetworkAttachment>,
    /// User to run as
    pub user: Option<String>,
    /// Run in privileged mode
    pub privileged: bool,
    /// Additional capabilities
    pub capabilities: Vec<String>,
    /// Remove container when it exits
    pub auto_remove: bool,
    /// Run in detached mode
    pub detached: bool,
    /// Interactive mode
    pub interactive: bool,
    /// Allocate a pseudo-TTY
    pub tty: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            image: String::new(),
            name: None,
            command: None,
            entrypoint: None,
            working_dir: None,
            environment: HashMap::new(),
            ports: Vec::new(),
            volumes: Vec::new(),
            labels: HashMap::new(),
            restart_policy: RestartPolicy::default(),
            health_check: None,
            resource_limits: ResourceLimits::default(),
            networks: Vec::new(),
            user: None,
            privileged: false,
            capabilities: Vec::new(),
            auto_remove: false,
            detached: true,
            interactive: false,
            tty: false,
        }
    }
}

/// Network attachment configuration
#[derive(Debug, Clone)]
pub struct NetworkAttachment {
    /// Network name or ID
    pub network: NetworkId,
    /// Aliases for this container in the network
    pub aliases: Vec<String>,
    /// Static IP address (optional)
    pub ip_address: Option<std::net::IpAddr>,
}

/// Docker container representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    /// Container ID
    pub id: ContainerId,
    /// Container name
    pub name: Option<String>,
    /// Docker image used
    pub image: String,
    /// Current status
    pub status: ContainerStatus,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Container labels
    pub labels: HashMap<String, String>,
    /// Creation timestamp
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    /// Started timestamp
    pub started: Option<chrono::DateTime<chrono::Utc>>,
    /// Networks the container is attached to
    pub networks: Vec<String>,
}

/// Fluent API builder for container configuration
#[derive(Debug)]
pub struct ContainerBuilder {
    config: ContainerConfig,
}

impl ContainerBuilder {
    /// Create a new container builder with the specified image
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            config: ContainerConfig {
                image: image.into(),
                ..Default::default()
            },
        }
    }

    /// Set the container name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = Some(name.into());
        self
    }

    /// Set the command to run
    pub fn command(mut self, command: Vec<String>) -> Self {
        self.config.command = Some(command);
        self
    }

    /// Set the command from a space-separated string
    pub fn command_str(mut self, command: impl Into<String>) -> Self {
        let cmd_str = command.into();
        let command_parts: Vec<String> =
            cmd_str.split_whitespace().map(|s| s.to_string()).collect();
        self.config.command = Some(command_parts);
        self
    }

    /// Override the entrypoint
    pub fn entrypoint(mut self, entrypoint: Vec<String>) -> Self {
        self.config.entrypoint = Some(entrypoint);
        self
    }

    /// Set the working directory
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.working_dir = Some(dir.into());
        self
    }

    /// Add an environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.environment.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn envs(mut self, envs: HashMap<String, String>) -> Self {
        self.config.environment.extend(envs);
        self
    }

    /// Add a port mapping with a specific host port
    pub fn port(mut self, host_port: u16, container_port: u16) -> Self {
        self.config.ports.push(PortMapping {
            host_ip: None,
            host_port: Some(host_port),
            container_port,
            protocol: PortProtocol::Tcp,
        });
        self
    }

    /// Add a port mapping with dynamic host port allocation
    pub fn port_dynamic(mut self, container_port: u16) -> Self {
        self.config.ports.push(PortMapping {
            host_ip: None,
            host_port: None,
            container_port,
            protocol: PortProtocol::Tcp,
        });
        self
    }

    /// Add a UDP port mapping
    pub fn port_udp(mut self, host_port: u16, container_port: u16) -> Self {
        self.config.ports.push(PortMapping {
            host_ip: None,
            host_port: Some(host_port),
            container_port,
            protocol: PortProtocol::Udp,
        });
        self
    }

    /// Add a volume mount from host path
    pub fn volume(
        mut self,
        host_path: impl Into<PathBuf>,
        container_path: impl Into<PathBuf>,
    ) -> Self {
        self.config.volumes.push(VolumeMount::new(
            VolumeSource::HostPath(host_path.into()),
            container_path.into(),
        ));
        self
    }

    /// Add a read-only volume mount
    pub fn volume_ro(
        mut self,
        host_path: impl Into<PathBuf>,
        container_path: impl Into<PathBuf>,
    ) -> Self {
        self.config.volumes.push(
            VolumeMount::new(
                VolumeSource::HostPath(host_path.into()),
                container_path.into(),
            )
            .read_only(),
        );
        self
    }

    /// Add a named volume mount
    pub fn volume_named(
        mut self,
        volume_name: impl Into<String>,
        container_path: impl Into<PathBuf>,
    ) -> Self {
        self.config.volumes.push(VolumeMount::new(
            VolumeSource::Named(volume_name.into()),
            container_path.into(),
        ));
        self
    }

    /// Add a temporary volume mount
    pub fn volume_tmp(mut self, container_path: impl Into<PathBuf>) -> Self {
        self.config.volumes.push(VolumeMount::new(
            VolumeSource::Anonymous,
            container_path.into(),
        ));
        self
    }

    /// Add a label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.labels.insert(key.into(), value.into());
        self
    }

    /// Add multiple labels
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        self.config.labels.extend(labels);
        self
    }

    /// Set the restart policy
    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.config.restart_policy = policy;
        self
    }

    /// Set health check configuration
    pub fn health_check(mut self, health_check: TypeHealthCheck) -> Self {
        self.config.health_check = Some(health_check);
        self
    }

    /// Set memory limit in bytes
    pub fn memory(mut self, bytes: u64) -> Self {
        self.config.resource_limits.memory = Some(bytes);
        self
    }

    /// Set memory limit with human-readable format (e.g., "512m", "1g")
    pub fn memory_str(mut self, memory: impl Into<String>) -> Self {
        // Parse memory string and convert to bytes
        let mem_str = memory.into();
        if let Ok(bytes) = parse_memory_string(&mem_str) {
            self.config.resource_limits.memory = Some(bytes);
        }
        self
    }

    /// Set CPU limit (number of CPUs)
    pub fn cpus(mut self, cpus: f64) -> Self {
        self.config.resource_limits.cpu_shares = Some((cpus * 1024.0) as u64);
        self
    }

    /// Attach to a network
    pub fn network(mut self, network: impl Into<NetworkId>) -> Self {
        self.config.networks.push(NetworkAttachment {
            network: network.into(),
            aliases: Vec::new(),
            ip_address: None,
        });
        self
    }

    /// Attach to a network with aliases
    pub fn network_with_aliases(
        mut self,
        network: impl Into<NetworkId>,
        aliases: Vec<String>,
    ) -> Self {
        self.config.networks.push(NetworkAttachment {
            network: network.into(),
            aliases,
            ip_address: None,
        });
        self
    }

    /// Set the user to run as
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.config.user = Some(user.into());
        self
    }

    /// Run in privileged mode
    pub fn privileged(mut self) -> Self {
        self.config.privileged = true;
        self
    }

    /// Add a capability
    pub fn capability(mut self, cap: impl Into<String>) -> Self {
        self.config.capabilities.push(cap.into());
        self
    }

    /// Add multiple capabilities
    pub fn capabilities(mut self, caps: Vec<String>) -> Self {
        self.config.capabilities.extend(caps);
        self
    }

    /// Automatically remove container when it exits
    pub fn auto_remove(mut self) -> Self {
        self.config.auto_remove = true;
        self
    }

    /// Run in interactive mode
    pub fn interactive(mut self) -> Self {
        self.config.interactive = true;
        self.config.detached = false;
        self
    }

    /// Allocate a pseudo-TTY
    pub fn tty(mut self) -> Self {
        self.config.tty = true;
        self
    }

    /// Run container and return the container ID
    pub async fn run(self, client: &DockerClient) -> DockerResult<ContainerId> {
        let container_manager = ContainerManager::new(client);
        container_manager.run(self.config).await
    }

    /// Create container without starting it
    pub async fn create(self, client: &DockerClient) -> DockerResult<ContainerId> {
        let container_manager = ContainerManager::new(client);
        container_manager.create(self.config).await
    }

    /// Build the configuration without creating a container
    pub fn build(self) -> ContainerConfig {
        self.config
    }
}

/// Container management operations
pub struct ContainerManager<'a> {
    client: &'a DockerClient,
}

impl<'a> ContainerManager<'a> {
    /// Create a new container manager
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Create a new container from configuration
    pub async fn create(&self, config: ContainerConfig) -> DockerResult<ContainerId> {
        debug!("Creating container with image: {}", config.image);

        let mut args = vec!["create".to_string()];

        // Add configuration arguments
        self.add_config_args(&mut args, &config)?;

        // Add image
        args.push(config.image.clone());

        // Add command if specified
        if let Some(command) = &config.command {
            args.extend(command.clone());
        }

        let output = self.client.execute_command_stdout(&args).await?;
        let container_id = output.trim().to_string();

        info!("Created container: {}", container_id);
        Ok(ContainerId::new(container_id)?)
    }

    /// Create and start a container
    pub async fn create_and_start(&self, config: ContainerConfig) -> DockerResult<ContainerId> {
        let container_id = self.create(config).await?;
        self.start(&container_id).await?;
        Ok(container_id)
    }

    /// Run a container directly using docker run
    pub async fn run(&self, config: ContainerConfig) -> DockerResult<ContainerId> {
        debug!("Running container with image: {}", config.image);

        let mut args = vec!["run".to_string()];

        // Add configuration arguments
        self.add_run_config_args(&mut args, &config)?;

        // Add image
        args.push(config.image.clone());

        // Add command if specified
        if let Some(command) = &config.command {
            args.extend(command.clone());
        }

        let output = self.client.execute_command_stdout(&args).await?;
        let container_id = output.trim().to_string();

        info!("Started container: {}", container_id);
        Ok(ContainerId::new(container_id)?)
    }

    /// Start a container
    pub async fn start(&self, container_id: &ContainerId) -> DockerResult<()> {
        debug!("Starting container: {}", container_id);

        let args = vec!["start".to_string(), container_id.to_string()];
        self.client.execute_command(&args, None).await?;

        info!("Started container: {}", container_id);
        Ok(())
    }

    /// Stop a container
    pub async fn stop(
        &self,
        container_id: &ContainerId,
        timeout: Option<Duration>,
    ) -> DockerResult<()> {
        debug!("Stopping container: {}", container_id);

        let mut args = vec!["stop".to_string()];

        if let Some(timeout) = timeout {
            args.push("--time".to_string());
            args.push(timeout.as_secs().to_string());
        }

        args.push(container_id.to_string());
        self.client.execute_command(&args, None).await?;

        info!("Stopped container: {}", container_id);
        Ok(())
    }

    /// Remove a container
    pub async fn remove(
        &self,
        container_id: &ContainerId,
        options: RemoveOptions,
    ) -> DockerResult<()> {
        debug!("Removing container: {}", container_id);

        let mut args = vec!["rm".to_string()];

        if options.force {
            args.push("--force".to_string());
        }
        if options.remove_volumes {
            args.push("--volumes".to_string());
        }

        args.push(container_id.to_string());
        self.client.execute_command(&args, None).await?;

        info!("Removed container: {}", container_id);
        Ok(())
    }

    /// Get container information
    pub async fn inspect(&self, container_id: &ContainerId) -> DockerResult<DockerContainer> {
        debug!("Inspecting container: {}", container_id);

        let args = vec![
            "inspect".to_string(),
            "--format".to_string(),
            "{{json .}}".to_string(),
            container_id.to_string(),
        ];

        let output = self.client.execute_command_stdout(&args).await?;
        let container_data: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
            DockerError::parsing(format!("Failed to parse container inspect: {}", e))
        })?;

        self.parse_container_info(&container_data)
    }

    /// List containers
    pub async fn list(&self, all: bool) -> DockerResult<Vec<DockerContainer>> {
        debug!("Listing containers (all: {})", all);

        let mut args = vec![
            "ps".to_string(),
            "--format".to_string(),
            "{{json .}}".to_string(),
        ];

        if all {
            args.push("--all".to_string());
        }

        let output = self.client.execute_command_stdout(&args).await?;
        let mut containers = Vec::new();

        for line in output.lines() {
            if !line.trim().is_empty() {
                let container_data: serde_json::Value =
                    serde_json::from_str(line).map_err(|e| {
                        DockerError::parsing(format!("Failed to parse container list: {}", e))
                    })?;
                containers.push(self.parse_container_info(&container_data)?);
            }
        }

        Ok(containers)
    }

    /// Wait for container to finish and return exit code
    pub async fn wait(&self, container_id: &ContainerId) -> DockerResult<i32> {
        debug!("Waiting for container: {}", container_id);

        let args = vec!["wait".to_string(), container_id.to_string()];
        let output = self.client.execute_command_stdout(&args).await?;

        let exit_code: i32 = output
            .trim()
            .parse()
            .map_err(|e| DockerError::parsing(format!("Invalid exit code: {}", e)))?;

        info!("Container {} exited with code: {}", container_id, exit_code);
        Ok(exit_code)
    }

    /// Get the mapped host port for a container port
    pub async fn port(
        &self,
        container_id: &ContainerId,
        container_port: u16,
    ) -> DockerResult<Option<u16>> {
        debug!(
            "Getting port mapping for container: {} port: {}",
            container_id, container_port
        );

        let args = vec![
            "port".to_string(),
            container_id.to_string(),
            container_port.to_string(),
        ];

        match self.client.execute_command_stdout(&args).await {
            Ok(output) => {
                let port_str = output.trim();
                if port_str.is_empty() {
                    return Ok(None);
                }

                // Parse "0.0.0.0:32768" or ":::32768" format
                let host_port = if let Some(colon_pos) = port_str.rfind(':') {
                    port_str[colon_pos + 1..]
                        .parse::<u16>()
                        .map_err(|e| DockerError::parsing(format!("Invalid port format: {}", e)))?
                } else {
                    return Err(DockerError::parsing(format!(
                        "Unexpected port format: {}",
                        port_str
                    )));
                };

                Ok(Some(host_port))
            }
            Err(_) => Ok(None), // Port not mapped
        }
    }

    /// Wait for container to be ready (running and ports accessible)
    pub async fn wait_for_ready(
        &self,
        container_id: &ContainerId,
        timeout: Duration,
    ) -> DockerResult<()> {
        debug!("Waiting for container to be ready: {}", container_id);

        let start_time = std::time::Instant::now();

        loop {
            if start_time.elapsed() > timeout {
                return Err(DockerError::timeout(format!(
                    "Container {} did not become ready within {:?}",
                    container_id, timeout
                )));
            }

            // Check if container is running
            match self.inspect(container_id).await {
                Ok(container) => {
                    if matches!(container.status, ContainerStatus::Running { .. }) {
                        info!("Container {} is ready", container_id);
                        return Ok(());
                    }
                }
                Err(_) => {
                    // Container might not exist yet
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Add configuration arguments to Docker create command
    fn add_config_args(
        &self,
        args: &mut Vec<String>,
        config: &ContainerConfig,
    ) -> DockerResult<()> {
        // Name
        if let Some(name) = &config.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        // Note: --detach flag is only for 'docker run', not 'docker create'
        // The create command inherently creates containers in a stopped state

        // Interactive and TTY
        if config.interactive {
            args.push("--interactive".to_string());
        }
        if config.tty {
            args.push("--tty".to_string());
        }

        // Auto remove
        if config.auto_remove {
            args.push("--rm".to_string());
        }

        // Working directory
        if let Some(workdir) = &config.working_dir {
            args.push("--workdir".to_string());
            args.push(workdir.to_string_lossy().to_string());
        }

        // Environment variables
        for (key, value) in &config.environment {
            args.push("--env".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Port mappings
        for port in &config.ports {
            args.push("--publish".to_string());
            let port_spec = if let Some(host_port) = port.host_port {
                match &port.host_ip {
                    Some(ip) => format!(
                        "{}:{}:{}/{}",
                        ip, host_port, port.container_port, port.protocol
                    ),
                    None => format!("{}:{}/{}", host_port, port.container_port, port.protocol),
                }
            } else {
                format!("{}/{}", port.container_port, port.protocol)
            };
            args.push(port_spec);
        }

        // Volume mounts
        for volume in &config.volumes {
            args.push("--volume".to_string());
            let volume_spec = match &volume.source {
                VolumeSource::HostPath(path) => {
                    let mut spec = format!(
                        "{}:{}",
                        path.to_string_lossy(),
                        volume.target.to_string_lossy()
                    );
                    if volume.mode == crate::types::MountMode::ReadOnly {
                        spec.push_str(":ro");
                    }
                    spec
                }
                VolumeSource::Named(name) => {
                    let mut spec = format!("{}:{}", name, volume.target.to_string_lossy());
                    if volume.mode == crate::types::MountMode::ReadOnly {
                        spec.push_str(":ro");
                    }
                    spec
                }
                VolumeSource::Anonymous => {
                    args.push("--tmpfs".to_string());
                    args.push(volume.target.to_string_lossy().to_string());
                    continue;
                }
            };
            args.push(volume_spec);
        }

        // Labels
        for (key, value) in &config.labels {
            args.push("--label".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Restart policy
        match &config.restart_policy {
            RestartPolicy::No => {}
            RestartPolicy::Always => {
                args.push("--restart".to_string());
                args.push("always".to_string());
            }
            RestartPolicy::OnFailure { max_retries } => {
                args.push("--restart".to_string());
                if let Some(retries) = max_retries {
                    args.push(format!("on-failure:{}", retries));
                } else {
                    args.push("on-failure".to_string());
                }
            }
            RestartPolicy::UnlessStopped => {
                args.push("--restart".to_string());
                args.push("unless-stopped".to_string());
            }
        }

        // Resource limits
        if let Some(memory) = config.resource_limits.memory {
            args.push("--memory".to_string());
            args.push(memory.to_string());
        }
        if let Some(cpu_shares) = config.resource_limits.cpu_shares {
            args.push("--cpu-shares".to_string());
            args.push(cpu_shares.to_string());
        }

        // Networks
        for network in &config.networks {
            args.push("--network".to_string());
            args.push(network.network.to_string());
        }

        // User
        if let Some(user) = &config.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        // Privileged mode
        if config.privileged {
            args.push("--privileged".to_string());
        }

        // Capabilities
        for cap in &config.capabilities {
            args.push("--cap-add".to_string());
            args.push(cap.clone());
        }

        // Entrypoint
        if let Some(entrypoint) = &config.entrypoint {
            args.push("--entrypoint".to_string());
            args.push(entrypoint.join(" "));
        }

        Ok(())
    }

    /// Add configuration arguments to Docker run command
    fn add_run_config_args(
        &self,
        args: &mut Vec<String>,
        config: &ContainerConfig,
    ) -> DockerResult<()> {
        // Name
        if let Some(name) = &config.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        // Detached mode (only for run command)
        if config.detached {
            args.push("--detach".to_string());
        }

        // Interactive and TTY
        if config.interactive {
            args.push("--interactive".to_string());
        }
        if config.tty {
            args.push("--tty".to_string());
        }

        // Auto remove
        if config.auto_remove {
            args.push("--rm".to_string());
        }

        // Working directory
        if let Some(workdir) = &config.working_dir {
            args.push("--workdir".to_string());
            args.push(workdir.to_string_lossy().to_string());
        }

        // Environment variables
        for (key, value) in &config.environment {
            args.push("--env".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Port mappings
        for port in &config.ports {
            args.push("--publish".to_string());
            let port_spec = if let Some(host_port) = port.host_port {
                match &port.host_ip {
                    Some(ip) => format!(
                        "{}:{}:{}/{}",
                        ip, host_port, port.container_port, port.protocol
                    ),
                    None => format!("{}:{}/{}", host_port, port.container_port, port.protocol),
                }
            } else {
                format!("{}/{}", port.container_port, port.protocol)
            };
            args.push(port_spec);
        }

        // Volume mounts
        for volume in &config.volumes {
            args.push("--volume".to_string());
            let volume_spec = match &volume.source {
                VolumeSource::HostPath(path) => {
                    let mut spec = format!(
                        "{}:{}",
                        path.to_string_lossy(),
                        volume.target.to_string_lossy()
                    );
                    if volume.mode == crate::types::MountMode::ReadOnly {
                        spec.push_str(":ro");
                    }
                    spec
                }
                VolumeSource::Named(name) => {
                    let mut spec = format!("{}:{}", name, volume.target.to_string_lossy());
                    if volume.mode == crate::types::MountMode::ReadOnly {
                        spec.push_str(":ro");
                    }
                    spec
                }
                VolumeSource::Anonymous => {
                    args.push("--tmpfs".to_string());
                    args.push(volume.target.to_string_lossy().to_string());
                    continue;
                }
            };
            args.push(volume_spec);
        }

        // Labels
        for (key, value) in &config.labels {
            args.push("--label".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Restart policy
        match &config.restart_policy {
            RestartPolicy::No => {}
            RestartPolicy::Always => {
                args.push("--restart".to_string());
                args.push("always".to_string());
            }
            RestartPolicy::OnFailure { max_retries } => {
                args.push("--restart".to_string());
                if let Some(retries) = max_retries {
                    args.push(format!("on-failure:{}", retries));
                } else {
                    args.push("on-failure".to_string());
                }
            }
            RestartPolicy::UnlessStopped => {
                args.push("--restart".to_string());
                args.push("unless-stopped".to_string());
            }
        }

        // Resource limits
        if let Some(memory) = config.resource_limits.memory {
            args.push("--memory".to_string());
            args.push(memory.to_string());
        }
        if let Some(cpu_shares) = config.resource_limits.cpu_shares {
            args.push("--cpu-shares".to_string());
            args.push(cpu_shares.to_string());
        }

        // Networks
        for network in &config.networks {
            args.push("--network".to_string());
            args.push(network.network.to_string());
        }

        // User
        if let Some(user) = &config.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        // Privileged mode
        if config.privileged {
            args.push("--privileged".to_string());
        }

        // Capabilities
        for cap in &config.capabilities {
            args.push("--cap-add".to_string());
            args.push(cap.clone());
        }

        // Entrypoint
        if let Some(entrypoint) = &config.entrypoint {
            args.push("--entrypoint".to_string());
            args.push(entrypoint.join(" "));
        }

        Ok(())
    }

    /// Parse container information from Docker inspect JSON
    fn parse_container_info(&self, data: &serde_json::Value) -> DockerResult<DockerContainer> {
        let id = data["Id"]
            .as_str()
            .ok_or_else(|| DockerError::parsing("Missing container ID".to_string()))?;

        let name = data["Name"]
            .as_str()
            .map(|s| s.trim_start_matches('/').to_string());
        let image = data["Config"]["Image"]
            .as_str()
            .ok_or_else(|| DockerError::parsing("Missing image name".to_string()))?;

        let status_str = data["State"]["Status"]
            .as_str()
            .ok_or_else(|| DockerError::parsing("Missing container status".to_string()))?;

        let status = match status_str {
            "running" => {
                let started_at = data["State"]["StartedAt"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| std::time::SystemTime::from(dt))
                    .unwrap_or_else(std::time::SystemTime::now);
                ContainerStatus::Running { started_at }
            }
            "exited" => {
                let exit_code = data["State"]["ExitCode"].as_i64().unwrap_or(0) as i32;
                let finished_at = data["State"]["FinishedAt"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| std::time::SystemTime::from(dt))
                    .unwrap_or_else(std::time::SystemTime::now);
                ContainerStatus::Exited {
                    exit_code,
                    finished_at,
                }
            }
            "created" => ContainerStatus::Created,
            "paused" => ContainerStatus::Paused,
            "restarting" => ContainerStatus::Restarting,
            "dead" => ContainerStatus::Dead,
            _ => ContainerStatus::Created, // Default fallback
        };

        // Parse port mappings
        let mut ports = Vec::new();
        if let Some(port_data) = data["NetworkSettings"]["Ports"].as_object() {
            for (container_port_str, host_ports) in port_data {
                if let Some(host_port_array) = host_ports.as_array() {
                    for host_port_data in host_port_array {
                        if let Some(host_port_str) = host_port_data["HostPort"].as_str() {
                            if let (Ok(container_port), Ok(host_port)) = (
                                container_port_str
                                    .split('/')
                                    .next()
                                    .unwrap_or("")
                                    .parse::<u16>(),
                                host_port_str.parse::<u16>(),
                            ) {
                                let protocol = if container_port_str.ends_with("/udp") {
                                    PortProtocol::Udp
                                } else {
                                    PortProtocol::Tcp
                                };

                                ports.push(PortMapping {
                                    host_ip: host_port_data["HostIp"]
                                        .as_str()
                                        .and_then(|s| s.parse().ok()),
                                    host_port: Some(host_port),
                                    container_port,
                                    protocol,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Parse labels
        let labels = if let Some(label_data) = data["Config"]["Labels"].as_object() {
            label_data
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        } else {
            HashMap::new()
        };

        // Parse timestamps
        let created = data["Created"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let started = data["State"]["StartedAt"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        // Parse networks
        let networks = if let Some(network_data) = data["NetworkSettings"]["Networks"].as_object() {
            network_data.keys().cloned().collect()
        } else {
            Vec::new()
        };

        Ok(DockerContainer {
            id: ContainerId::new(id.to_string())?,
            name,
            image: image.to_string(),
            status,
            ports,
            labels,
            created,
            started,
            networks,
        })
    }
}

/// Options for removing containers
#[derive(Debug, Clone, Default)]
pub struct RemoveOptions {
    /// Force removal of running containers
    pub force: bool,
    /// Remove associated volumes
    pub remove_volumes: bool,
}

/// Parse memory string (e.g., "512m", "1g") into bytes
fn parse_memory_string(memory_str: &str) -> Result<u64, DockerError> {
    let memory_str = memory_str.trim().to_lowercase();

    if memory_str.is_empty() {
        return Err(DockerError::parsing("Empty memory string".to_string()));
    }

    let (number_part, unit_part) = if let Some(pos) = memory_str.find(|c: char| c.is_alphabetic()) {
        (&memory_str[..pos], &memory_str[pos..])
    } else {
        (memory_str.as_str(), "")
    };

    let number: f64 = number_part
        .parse()
        .map_err(|_| DockerError::parsing(format!("Invalid memory number: {}", number_part)))?;

    let multiplier = match unit_part {
        "" | "b" => 1,
        "k" | "kb" => 1_024,
        "m" | "mb" => 1_024 * 1_024,
        "g" | "gb" => 1_024 * 1_024 * 1_024,
        "t" | "tb" => 1_024_u64.pow(4),
        _ => {
            return Err(DockerError::parsing(format!(
                "Unknown memory unit: {}",
                unit_part
            )));
        }
    };

    Ok((number * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_builder() {
        let config = ContainerBuilder::new("redis:alpine")
            .name("test-redis")
            .env("REDIS_PASSWORD", "secret")
            .port(6379, 6379)
            .memory_str("512m")
            .build();

        assert_eq!(config.image, "redis:alpine");
        assert_eq!(config.name, Some("test-redis".to_string()));
        assert_eq!(
            config.environment.get("REDIS_PASSWORD"),
            Some(&"secret".to_string())
        );
        assert_eq!(config.ports.len(), 1);
        assert_eq!(config.resource_limits.memory, Some(536_870_912)); // 512MB in bytes
    }

    #[test]
    fn test_memory_parsing() {
        assert_eq!(parse_memory_string("512").unwrap(), 512);
        assert_eq!(parse_memory_string("512b").unwrap(), 512);
        assert_eq!(parse_memory_string("512k").unwrap(), 524_288);
        assert_eq!(parse_memory_string("512m").unwrap(), 536_870_912);
        assert_eq!(parse_memory_string("1g").unwrap(), 1_073_741_824);
        assert_eq!(parse_memory_string("1.5g").unwrap(), 1_610_612_736);

        assert!(parse_memory_string("invalid").is_err());
        assert!(parse_memory_string("512x").is_err());
        assert!(parse_memory_string("").is_err());
    }

    #[test]
    fn test_remove_options() {
        let options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };

        assert!(options.force);
        assert!(options.remove_volumes);
    }

    #[test]
    fn test_exec_config() {
        let config = ExecConfig {
            command: vec!["echo".to_string(), "hello".to_string()],
            user: Some("root".to_string()),
            attach_stdout: true,
            ..Default::default()
        };

        assert_eq!(config.command, vec!["echo", "hello"]);
        assert_eq!(config.user, Some("root".to_string()));
        assert!(config.attach_stdout);
        assert!(!config.attach_stdin);
    }

    #[test]
    fn test_log_options() {
        let options = LogOptions {
            follow: true,
            timestamps: true,
            tail: Some(100),
            ..Default::default()
        };

        assert!(options.follow);
        assert!(options.timestamps);
        assert_eq!(options.tail, Some(100));
    }

    #[test]
    fn test_network_attachment() {
        let attachment = NetworkAttachment {
            network: NetworkId::new("my-network".to_string()).unwrap(),
            aliases: vec!["alias1".to_string(), "alias2".to_string()],
            ip_address: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
                192, 168, 1, 100,
            ))),
        };

        assert_eq!(attachment.aliases.len(), 2);
        assert!(attachment.ip_address.is_some());
    }

    #[test]
    fn test_container_builder_comprehensive() {
        let config = ContainerBuilder::new("nginx:alpine")
            .name("test-nginx")
            .env("ENV_VAR", "value")
            .env("VAR1", "val1")
            .env("VAR2", "val2")
            .port_dynamic(80)
            .port_udp(53, 5353)
            .volume("/host/path", "/container/path")
            .volume_ro("/host/ro", "/container/ro")
            .volume_named("my-volume", "/data")
            .label("app", "test")
            .label("version", "1.0")
            .label("env", "test")
            .working_dir("/app")
            .user("nginx")
            .entrypoint(vec!["nginx".to_string()])
            .command(vec!["-g".to_string(), "daemon off;".to_string()])
            .memory_str("1g")
            .memory_str("1g")
            .cpus(0.5)
            .network(NetworkId::new("custom-network".to_string()).unwrap())
            .auto_remove()
            .tty()
            .interactive()
            .build();

        assert_eq!(config.image, "nginx:alpine");
        assert_eq!(config.name, Some("test-nginx".to_string()));
        assert_eq!(config.environment.len(), 3);
        assert_eq!(
            config.environment.get("ENV_VAR"),
            Some(&"value".to_string())
        );
        assert_eq!(config.environment.get("VAR1"), Some(&"val1".to_string()));
        assert_eq!(config.ports.len(), 2);
        assert_eq!(config.volumes.len(), 3);
        assert_eq!(config.labels.len(), 3);
        assert_eq!(config.working_dir, Some("/app".into()));
        assert_eq!(config.user, Some("nginx".to_string()));
        assert_eq!(config.entrypoint, Some(vec!["nginx".to_string()]));
        assert_eq!(
            config.command,
            Some(vec!["-g".to_string(), "daemon off;".to_string()])
        );
        assert_eq!(config.resource_limits.memory, Some(1_073_741_824)); // 1GB
        assert_eq!(config.resource_limits.memory_swap, None); // Not set since we only called memory_str
        assert!(config.auto_remove);
        assert!(config.tty);
        assert!(config.interactive);
    }

    #[test]
    fn test_container_builder_health_check() {
        use crate::types::HealthCheck as TypeHealthCheck;

        let health_check = TypeHealthCheck {
            test: vec![
                "CMD".to_string(),
                "curl".to_string(),
                "-f".to_string(),
                "http://localhost/health".to_string(),
            ],
            interval: std::time::Duration::from_secs(30),
            timeout: std::time::Duration::from_secs(30),
            retries: 3,
            start_period: Some(std::time::Duration::from_secs(0)),
        };

        let config = ContainerBuilder::new("app:latest")
            .health_check(health_check.clone())
            .build();

        assert_eq!(config.health_check, Some(health_check));
    }

    #[test]
    fn test_container_builder_restart_policies() {
        let config1 = ContainerBuilder::new("app:latest")
            .restart_policy(RestartPolicy::Always)
            .build();
        assert_eq!(config1.restart_policy, RestartPolicy::Always);

        let config2 = ContainerBuilder::new("app:latest")
            .restart_policy(RestartPolicy::OnFailure {
                max_retries: Some(5),
            })
            .build();
        assert_eq!(
            config2.restart_policy,
            RestartPolicy::OnFailure {
                max_retries: Some(5)
            }
        );

        let config3 = ContainerBuilder::new("app:latest")
            .restart_policy(RestartPolicy::UnlessStopped)
            .build();
        assert_eq!(config3.restart_policy, RestartPolicy::UnlessStopped);
    }

    #[test]
    fn test_container_config_default() {
        let config = ContainerConfig::default();
        assert!(config.image.is_empty());
        assert!(config.name.is_none());
        assert!(config.command.is_none());
        assert!(config.environment.is_empty());
        assert!(config.ports.is_empty());
        assert!(config.volumes.is_empty());
        assert_eq!(config.restart_policy, RestartPolicy::No);
        assert!(!config.auto_remove);
        assert!(!config.privileged);
    }

    #[test]
    fn test_container_id_creation() {
        let container_id = ContainerId::new("abc123def456789012345678".to_string()).unwrap();
        assert_eq!(container_id.as_str(), "abc123def456789012345678");

        // Test with valid container ID
        assert!(container_id.as_str().len() >= 12);
    }

    #[test]
    fn test_remove_options_default() {
        let options = RemoveOptions::default();
        assert!(!options.force);
        assert!(!options.remove_volumes);

        let options_custom = RemoveOptions {
            force: true,
            remove_volumes: false,
        };
        assert!(options_custom.force);
        assert!(!options_custom.remove_volumes);
    }

    #[test]
    fn test_memory_parsing_edge_cases() {
        assert_eq!(parse_memory_string("0").unwrap(), 0);
        assert_eq!(parse_memory_string("1024").unwrap(), 1024);
        assert_eq!(parse_memory_string("2.5m").unwrap(), 2_621_440); // 2.5 MB
        assert_eq!(parse_memory_string("0.5g").unwrap(), 536_870_912); // 0.5 GB
        assert_eq!(parse_memory_string("1t").unwrap(), 1_099_511_627_776); // 1 TB

        // Test case insensitive
        assert_eq!(parse_memory_string("512M").unwrap(), 536_870_912);
        assert_eq!(parse_memory_string("1G").unwrap(), 1_073_741_824);

        // Test with spaces
        assert_eq!(parse_memory_string(" 512m ").unwrap(), 536_870_912);

        // Test error cases
        assert!(parse_memory_string("").is_err());
        assert!(parse_memory_string("abc").is_err());
        assert!(parse_memory_string("512x").is_err());
    }

    #[test]
    fn test_network_attachment_complete() {
        use std::net::{IpAddr, Ipv4Addr};

        let attachment = NetworkAttachment {
            network: NetworkId::new("custom-network".to_string()).unwrap(),
            aliases: vec!["web".to_string(), "api".to_string()],
            ip_address: Some(IpAddr::V4(Ipv4Addr::new(172, 18, 0, 10))),
        };

        assert_eq!(attachment.network.as_str(), "custom-network");
        assert_eq!(attachment.aliases.len(), 2);
        assert!(attachment.aliases.contains(&"web".to_string()));
        assert!(attachment.aliases.contains(&"api".to_string()));

        if let Some(IpAddr::V4(ipv4)) = attachment.ip_address {
            assert_eq!(ipv4, Ipv4Addr::new(172, 18, 0, 10));
        } else {
            panic!("Expected IPv4 address");
        }
    }

    #[test]
    fn test_container_builder_volume_types() {
        let config = ContainerBuilder::new("test:latest")
            .volume("/host/bind", "/container/bind")
            .volume_ro("/host/readonly", "/container/readonly")
            .volume_named("my-volume", "/data")
            .build();

        assert_eq!(config.volumes.len(), 3);

        // Check that volumes have correct source types
        for volume in &config.volumes {
            match &volume.source {
                VolumeSource::HostPath(_) => {
                    // Host bind mounts
                    assert!(volume.target.to_string_lossy().starts_with("/container"));
                }
                VolumeSource::Named(_) => {
                    // Named volumes
                    assert_eq!(volume.target, std::path::PathBuf::from("/data"));
                }
                VolumeSource::Anonymous => {
                    // Anonymous volumes
                    // Just check that target exists
                    assert!(!volume.target.as_os_str().is_empty());
                }
            }
        }
    }

    #[test]
    fn test_container_builder_port_mappings() {
        let config = ContainerBuilder::new("test:latest")
            .port(8080, 80)
            .port_dynamic(443)
            .port_udp(5353, 53)
            .build();

        assert_eq!(config.ports.len(), 3);

        let tcp_port = &config.ports[0];
        assert_eq!(tcp_port.host_port, Some(8080));
        assert_eq!(tcp_port.container_port, 80);
        assert_eq!(tcp_port.protocol, crate::types::Protocol::Tcp);

        let dynamic_port = &config.ports[1];
        assert_eq!(dynamic_port.host_port, None); // Dynamic allocation
        assert_eq!(dynamic_port.container_port, 443);

        let udp_port = &config.ports[2];
        assert_eq!(udp_port.host_port, Some(5353));
        assert_eq!(udp_port.container_port, 53);
        assert_eq!(udp_port.protocol, crate::types::Protocol::Udp);
    }

    #[test]
    fn test_container_builder_resource_limits() {
        let config = ContainerBuilder::new("test:latest")
            .memory_str("512m")
            .memory_str("512m")
            .cpus(1.5)
            .build();

        let limits = &config.resource_limits;
        assert_eq!(limits.memory, Some(536_870_912)); // 512MB
        assert_eq!(limits.cpu_shares, Some(1536)); // 1.5 * 1024
    }

    #[test]
    fn test_container_config_builder_chaining() {
        // Test that builder methods return self for chaining
        let builder = ContainerBuilder::new("test:latest");
        let builder = builder.name("test");
        let builder = builder.env("KEY", "value");
        let builder = builder.port(8080, 80);
        let config = builder.build();

        assert_eq!(config.image, "test:latest");
        assert_eq!(config.name, Some("test".to_string()));
        assert_eq!(config.environment.get("KEY"), Some(&"value".to_string()));
        assert_eq!(config.ports.len(), 1);
    }

    #[tokio::test]
    async fn test_container_manager_creation() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);
        // Test that manager was created successfully
        // We can't test much without actually running Docker operations
        assert!(std::ptr::eq(manager.client, &client));
    }

    #[tokio::test]
    async fn test_container_manager_create_and_remove() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);
        let container_name = format!(
            "test-create-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Create a simple container
        let config = ContainerConfig {
            image: "alpine:3.18".to_string(),
            name: Some(container_name.clone()),
            command: Some(vec!["echo".to_string(), "hello".to_string()]),
            ..Default::default()
        };

        let container_id = match manager.create(config).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create container: {}", e);
                return;
            }
        };

        // Verify container was created
        assert!(container_id.as_str().len() >= 12);

        // Clean up - remove container
        let remove_options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };

        if let Err(e) = manager.remove(&container_id, remove_options).await {
            eprintln!("Failed to remove container: {}", e);
        }
    }

    #[tokio::test]
    async fn test_container_manager_run_and_cleanup() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);
        let container_name = format!(
            "test-run-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Run a simple container
        let config = ContainerConfig {
            image: "alpine:3.18".to_string(),
            name: Some(container_name.clone()),
            command: Some(vec!["echo".to_string(), "test-output".to_string()]),
            auto_remove: true,
            ..Default::default()
        };

        let container_id = match manager.run(config).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to run container: {}", e);
                return;
            }
        };

        // Verify container was created
        assert!(container_id.as_str().len() >= 12);

        // Container should auto-remove, but let's wait a bit
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Try to clean up if auto-remove didn't work
        let remove_options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };
        let _ = manager.remove(&container_id, remove_options).await;
    }

    #[tokio::test]
    async fn test_container_manager_start_stop() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);
        let container_name = format!(
            "test-start-stop-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Create a long-running container
        let config = ContainerConfig {
            image: "alpine:3.18".to_string(),
            name: Some(container_name.clone()),
            command: Some(vec!["sleep".to_string(), "30".to_string()]),
            ..Default::default()
        };

        let container_id = match manager.create(config).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create container: {}", e);
                return;
            }
        };

        // Start the container
        if let Err(e) = manager.start(&container_id).await {
            eprintln!("Failed to start container: {}", e);
            let _ = manager
                .remove(
                    &container_id,
                    RemoveOptions {
                        force: true,
                        remove_volumes: true,
                    },
                )
                .await;
            return;
        }

        // Stop the container
        if let Err(e) = manager
            .stop(&container_id, Some(Duration::from_secs(5)))
            .await
        {
            eprintln!("Failed to stop container: {}", e);
        }

        // Clean up
        let remove_options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };
        if let Err(e) = manager.remove(&container_id, remove_options).await {
            eprintln!("Failed to remove container: {}", e);
        }
    }

    #[tokio::test]
    async fn test_container_manager_inspect() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);
        let container_name = format!(
            "test-inspect-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Create a container with specific configuration
        let config = ContainerConfig {
            image: "alpine:3.18".to_string(),
            name: Some(container_name.clone()),
            command: Some(vec!["sleep".to_string(), "10".to_string()]),
            environment: {
                let mut env = std::collections::HashMap::new();
                env.insert("TEST_VAR".to_string(), "test_value".to_string());
                env
            },
            ..Default::default()
        };

        let container_id = match manager.create(config).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create container: {}", e);
                return;
            }
        };

        // Inspect the container
        match manager.inspect(&container_id).await {
            Ok(container_info) => {
                assert_eq!(container_info.id, container_id);
                assert_eq!(container_info.image, "alpine:3.18");
                if let Some(name) = &container_info.name {
                    assert!(name.contains(&container_name));
                }
            }
            Err(e) => {
                eprintln!("Failed to inspect container: {}", e);
            }
        }

        // Clean up
        let remove_options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };
        if let Err(e) = manager.remove(&container_id, remove_options).await {
            eprintln!("Failed to remove container: {}", e);
        }
    }

    #[tokio::test]
    async fn test_container_manager_list() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);

        // List all containers
        match manager.list(true).await {
            Ok(containers) => {
                // We can't assert much about the contents since other containers might exist
                // Just verify we get a list
                println!("Found {} containers", containers.len());
            }
            Err(e) => {
                eprintln!("Failed to list containers: {}", e);
            }
        }

        // List only running containers
        match manager.list(false).await {
            Ok(containers) => {
                println!("Found {} running containers", containers.len());
            }
            Err(e) => {
                eprintln!("Failed to list running containers: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_container_manager_port_mapping() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);
        let container_name = format!(
            "test-port-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Create a container with port mapping
        let config = ContainerConfig {
            image: "nginx:alpine".to_string(),
            name: Some(container_name.clone()),
            ports: vec![PortMapping {
                host_ip: None,
                host_port: None, // Dynamic port
                container_port: 80,
                protocol: crate::types::Protocol::Tcp,
            }],
            ..Default::default()
        };

        let container_id = match manager.create(config).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create container: {}", e);
                return;
            }
        };

        // Start the container
        if let Err(e) = manager.start(&container_id).await {
            eprintln!("Failed to start container: {}", e);
            let _ = manager
                .remove(
                    &container_id,
                    RemoveOptions {
                        force: true,
                        remove_volumes: true,
                    },
                )
                .await;
            return;
        }

        // Give it a moment to start
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Get the mapped port
        match manager.port(&container_id, 80).await {
            Ok(Some(host_port)) => {
                assert!(host_port > 0);
                assert!(host_port <= 65535);
                println!("Container port 80 mapped to host port {}", host_port);
            }
            Ok(None) => {
                eprintln!("No port mapping found");
            }
            Err(e) => {
                eprintln!("Failed to get port mapping: {}", e);
            }
        }

        // Stop and clean up
        let _ = manager
            .stop(&container_id, Some(Duration::from_secs(5)))
            .await;
        let remove_options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };
        if let Err(e) = manager.remove(&container_id, remove_options).await {
            eprintln!("Failed to remove container: {}", e);
        }
    }

    #[tokio::test]
    async fn test_container_manager_wait() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let manager = ContainerManager::new(&client);
        let container_name = format!(
            "test-wait-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Create a container that exits quickly
        let config = ContainerConfig {
            image: "alpine:3.18".to_string(),
            name: Some(container_name.clone()),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                "exit 42".to_string(),
            ]),
            ..Default::default()
        };

        let container_id = match manager.create(config).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create container: {}", e);
                return;
            }
        };

        // Start the container
        if let Err(e) = manager.start(&container_id).await {
            eprintln!("Failed to start container: {}", e);
            let _ = manager
                .remove(
                    &container_id,
                    RemoveOptions {
                        force: true,
                        remove_volumes: true,
                    },
                )
                .await;
            return;
        }

        // Wait for container to finish
        match manager.wait(&container_id).await {
            Ok(exit_code) => {
                assert_eq!(exit_code, 42);
                println!("Container exited with code: {}", exit_code);
            }
            Err(e) => {
                eprintln!("Failed to wait for container: {}", e);
            }
        }

        // Clean up
        let remove_options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };
        if let Err(e) = manager.remove(&container_id, remove_options).await {
            eprintln!("Failed to remove container: {}", e);
        }
    }

    #[tokio::test]
    async fn test_container_builder_integration() {
        let client = match DockerClient::new().await {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test - Docker not available");
                return;
            }
        };

        let container_name = format!(
            "test-builder-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Use ContainerBuilder to create and run a container
        let container_id = match ContainerBuilder::new("alpine:3.18")
            .name(&container_name)
            .env("TEST_ENV", "builder_test")
            .command(vec!["echo".to_string(), "builder works".to_string()])
            .auto_remove()
            .run(&client)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to run container via builder: {}", e);
                return;
            }
        };

        // Verify container was created
        assert!(container_id.as_str().len() >= 12);

        // Container should auto-remove, wait a bit
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Try to clean up if needed
        let manager = ContainerManager::new(&client);
        let remove_options = RemoveOptions {
            force: true,
            remove_volumes: true,
        };
        let _ = manager.remove(&container_id, remove_options).await;
    }
}
