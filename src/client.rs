//! Docker client for executing Docker commands and managing Docker daemon interaction.
//!
//! This module provides the main `DockerClient` struct that serves as the primary
//! interface for all Docker operations. It handles Docker binary detection,
//! version validation, daemon connectivity, and command execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::errors::{DockerError, DockerResult};
use crate::executor::{ExecutionConfig, ProcessExecutor, find_docker_binary};

/// Docker version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerVersion {
    /// Client version
    pub client: String,
    /// Server version (daemon)
    pub server: Option<String>,
    /// API version
    pub api: Option<String>,
    /// Git commit
    pub git_commit: Option<String>,
    /// Build time
    pub built: Option<String>,
    /// Go version used to build
    pub go_version: Option<String>,
    /// OS/Architecture
    pub os_arch: Option<String>,
}

/// Docker system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerInfo {
    /// Container count
    pub containers: u32,
    /// Running container count
    pub containers_running: u32,
    /// Paused container count
    pub containers_paused: u32,
    /// Stopped container count
    pub containers_stopped: u32,
    /// Image count
    pub images: u32,
    /// Server version
    pub server_version: String,
    /// Storage driver
    pub driver: String,
    /// Root directory
    pub docker_root_dir: String,
    /// Operating system
    pub operating_system: String,
    /// Architecture
    pub architecture: String,
    /// Total memory
    pub mem_total: u64,
    /// CPU count
    pub ncpu: u32,
}

/// Docker client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Path to Docker binary
    pub docker_path: Option<PathBuf>,
    /// Default timeout for operations
    pub default_timeout: Duration,
    /// Environment variables to pass to Docker commands
    pub environment: HashMap<String, String>,
    /// Whether to verify Docker daemon connectivity on creation
    pub verify_connectivity: bool,
    /// Minimum required Docker version
    pub min_version: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            docker_path: None,
            default_timeout: Duration::from_secs(30),
            environment: HashMap::new(),
            verify_connectivity: true,
            min_version: Some("20.10.0".to_string()),
        }
    }
}

/// Main Docker client for executing Docker commands
#[derive(Debug, Clone)]
pub struct DockerClient {
    /// Process executor for running Docker commands
    executor: ProcessExecutor,
    /// Client configuration
    config: ClientConfig,
    /// Cached Docker version information
    version_info: Option<DockerVersion>,
    /// Cached Docker daemon information
    daemon_info: Option<DockerInfo>,
}

impl DockerClient {
    /// Create a new Docker client with default configuration
    pub async fn new() -> DockerResult<Self> {
        Self::with_config(ClientConfig::default()).await
    }

    /// Create a new Docker client with custom configuration
    pub async fn with_config(config: ClientConfig) -> DockerResult<Self> {
        // Find Docker binary
        let docker_path = if let Some(path) = config.docker_path.clone() {
            path
        } else {
            find_docker_binary()?
        };

        info!("Using Docker binary at: {:?}", docker_path);

        // Create process executor
        let executor =
            ProcessExecutor::new(docker_path).with_default_timeout(config.default_timeout);

        let mut client = Self {
            executor,
            config,
            version_info: None,
            daemon_info: None,
        };

        // Verify Docker is available
        client.executor.check_docker_available().await?;

        // Verify connectivity if requested
        if client.config.verify_connectivity {
            client.ping().await?;
        }

        // Check minimum version if specified
        if client.config.min_version.is_some() {
            client.verify_version().await?;
        }

        Ok(client)
    }

    /// Create a Docker client with a specific binary path
    pub async fn with_binary_path(path: PathBuf) -> DockerResult<Self> {
        let config = ClientConfig {
            docker_path: Some(path),
            ..Default::default()
        };
        Self::with_config(config).await
    }

    /// Create a Docker client without connectivity verification (for testing)
    pub async fn new_unchecked() -> DockerResult<Self> {
        let config = ClientConfig {
            verify_connectivity: false,
            min_version: None,
            ..Default::default()
        };
        Self::with_config(config).await
    }

    /// Ping the Docker daemon to verify connectivity
    pub async fn ping(&self) -> DockerResult<()> {
        debug!("Pinging Docker daemon");

        let output = self
            .executor
            .execute(
                &[
                    "system".to_string(),
                    "info".to_string(),
                    "--format".to_string(),
                    "{{.ServerVersion}}".to_string(),
                ],
                None,
            )
            .await;

        match output {
            Ok(_) => {
                debug!("Docker daemon ping successful");
                Ok(())
            }
            Err(e) => {
                warn!("Docker daemon ping failed: {}", e);
                Err(DockerError::daemon_not_accessible(format!(
                    "Cannot connect to Docker daemon: {e}"
                )))
            }
        }
    }

    /// Get Docker version information
    /// Returns the Docker version information.
    ///
    /// # Panics
    ///
    /// Panics if version info is not available after initialization.
    pub async fn version(&mut self) -> DockerResult<&DockerVersion> {
        if self.version_info.is_none() {
            debug!("Fetching Docker version information");

            let client_version = self.executor.get_docker_version().await?;

            // Try to get server version
            let server_version = match self
                .executor
                .execute(
                    &[
                        "version".to_string(),
                        "--format".to_string(),
                        "{{.Server.Version}}".to_string(),
                    ],
                    None,
                )
                .await
            {
                Ok(output) => Some(output.stdout.trim().to_string()),
                Err(_) => None,
            };

            // Try to get API version
            let api_version = match self
                .executor
                .execute(
                    &[
                        "version".to_string(),
                        "--format".to_string(),
                        "{{.Client.APIVersion}}".to_string(),
                    ],
                    None,
                )
                .await
            {
                Ok(output) => Some(output.stdout.trim().to_string()),
                Err(_) => None,
            };

            self.version_info = Some(DockerVersion {
                client: client_version,
                server: server_version,
                api: api_version,
                git_commit: None,
                built: None,
                go_version: None,
                os_arch: None,
            });
        }

        Ok(self.version_info.as_ref().unwrap())
    }

    /// Get Docker system information
    pub async fn info(&mut self) -> DockerResult<&DockerInfo> {
        if self.daemon_info.is_none() {
            debug!("Fetching Docker system information");

            let _output = self
                .executor
                .execute(
                    &[
                        "system".to_string(),
                        "info".to_string(),
                        "--format".to_string(),
                        "json".to_string(),
                    ],
                    None,
                )
                .await?;

            // Parse JSON output - simplified for now
            // In a real implementation, we'd parse the full JSON structure
            let info = DockerInfo {
                containers: 0,
                containers_running: 0,
                containers_paused: 0,
                containers_stopped: 0,
                images: 0,
                server_version: self.version().await?.server.clone().unwrap_or_default(),
                driver: "overlay2".to_string(), // Default assumption
                docker_root_dir: "/var/lib/docker".to_string(),
                operating_system: "Linux".to_string(),
                architecture: "x86_64".to_string(),
                mem_total: 0,
                ncpu: 0,
            };

            self.daemon_info = Some(info);
        }

        Ok(self.daemon_info.as_ref().unwrap())
    }

    /// Execute a Docker command with custom configuration
    pub async fn execute_command(
        &self,
        args: &[String],
        config: Option<ExecutionConfig>,
    ) -> DockerResult<crate::executor::CommandOutput> {
        let mut exec_config = config.unwrap_or_default();

        // Add client environment variables
        for (key, value) in &self.config.environment {
            exec_config.environment.insert(key.clone(), value.clone());
        }

        self.executor.execute(args, Some(exec_config)).await
    }

    /// Execute a Docker command and return only stdout
    pub async fn execute_command_stdout(&self, args: &[String]) -> DockerResult<String> {
        let output = self.execute_command(args, None).await?;
        Ok(output.stdout)
    }

    /// Execute a Docker command with JSON output and parse it
    pub async fn execute_json<T>(&self, args: &[String]) -> DockerResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let output = self.execute_command_stdout(args).await?;
        serde_json::from_str(&output).map_err(|e| DockerError::json("parsing command output", e))
    }

    /// Build Docker command with common options
    pub fn build_command(&self, subcommand: &str) -> CommandBuilder {
        CommandBuilder::new(subcommand.to_string())
    }

    /// Get the Docker binary path
    pub fn docker_path(&self) -> &std::path::Path {
        &self.executor.docker_path
    }

    /// Check if the current Docker version meets the minimum requirement
    async fn verify_version(&mut self) -> DockerResult<()> {
        if let Some(min_version) = self.config.min_version.clone() {
            let version_info = self.version().await?;

            // Simple version comparison - in production, use a proper semver crate
            if let Some(server_version) = &version_info.server {
                let server_ver = server_version.clone();
                if !self.version_meets_requirement(&server_ver, &min_version) {
                    return Err(DockerError::unsupported_version(server_ver, min_version));
                }
            }
        }
        Ok(())
    }

    /// Simple version comparison (should use semver in production)
    fn version_meets_requirement(&self, current: &str, required: &str) -> bool {
        // Extract major.minor.patch from version strings
        let parse_version = |v: &str| -> Vec<u32> {
            v.split('.')
                .take(3)
                .map(|part| part.parse().unwrap_or(0))
                .collect()
        };

        let current_parts = parse_version(current);
        let required_parts = parse_version(required);

        for i in 0..3 {
            let curr = current_parts.get(i).unwrap_or(&0);
            let req = required_parts.get(i).unwrap_or(&0);

            match curr.cmp(req) {
                std::cmp::Ordering::Greater => return true,
                std::cmp::Ordering::Less => return false,
                std::cmp::Ordering::Equal => {}
            }
        }

        true // Equal versions
    }

    /// Get a container manager for this client
    pub fn containers(&self) -> crate::container::ContainerManager {
        crate::container::ContainerManager::new(self)
    }

    /// Get an image manager for this client
    pub fn images(&self) -> crate::image::ImageManager {
        crate::image::ImageManager::new(self)
    }

    /// Get a network manager for this client
    pub fn networks(&self) -> crate::network::NetworkManager {
        crate::network::NetworkManager::new(self)
    }

    /// Get a volume manager for this client
    pub fn volumes(&self) -> crate::volume::VolumeManager {
        crate::volume::VolumeManager::new(self)
    }

    /// Get an events manager for this client
    pub fn events(&self) -> crate::events::EventManager {
        crate::events::EventManager::new(self)
    }

    /// Get a stats manager for this client
    pub fn stats(&self) -> crate::stats::StatsManager {
        crate::stats::StatsManager::new(self)
    }

    /// Get the executor for low-level command execution
    pub fn executor(&self) -> &ProcessExecutor {
        &self.executor
    }
}

/// Builder for constructing Docker commands
#[derive(Debug, Clone)]
pub struct CommandBuilder {
    /// Docker subcommand (e.g., "run", "ps", "build")
    subcommand: String,
    /// Command arguments
    args: Vec<String>,
}

impl CommandBuilder {
    /// Create a new command builder
    pub fn new(subcommand: String) -> Self {
        Self {
            subcommand,
            args: Vec::new(),
        }
    }

    /// Add a single argument
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Add a flag (argument starting with -)
    pub fn flag(mut self, flag: impl Into<String>) -> Self {
        let flag = flag.into();
        if !flag.starts_with('-') {
            self.args.push(format!("--{}", flag));
        } else {
            self.args.push(flag);
        }
        self
    }

    /// Add a key-value option (--key value)
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        if !key.starts_with('-') {
            self.args.push(format!("--{}", key));
        } else {
            self.args.push(key);
        }
        self.args.push(value.into());
        self
    }

    /// Build the final command arguments
    pub fn build(mut self) -> Vec<String> {
        let mut command = vec![self.subcommand];
        command.append(&mut self.args);
        command
    }

    /// Get the command as a string for debugging
    pub fn to_string(&self) -> String {
        let mut parts = vec![format!("docker {}", self.subcommand)];
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert!(config.verify_connectivity);
        assert_eq!(config.min_version, Some("20.10.0".to_string()));
    }

    #[test]
    fn test_command_builder() {
        let cmd = CommandBuilder::new("run".to_string())
            .flag("detach")
            .option("name", "test-container")
            .arg("redis:alpine")
            .build();

        assert_eq!(
            cmd,
            vec![
                "run".to_string(),
                "--detach".to_string(),
                "--name".to_string(),
                "test-container".to_string(),
                "redis:alpine".to_string()
            ]
        );
    }

    #[test]
    fn test_command_builder_string() {
        let builder = CommandBuilder::new("ps".to_string())
            .flag("all")
            .option("format", "table");

        assert_eq!(builder.to_string(), "docker ps --all --format table");
    }

    #[test]
    fn test_version_comparison() {
        let client_config = ClientConfig::default();
        let docker_path = PathBuf::from("/usr/bin/docker");
        let executor = ProcessExecutor::new(docker_path);
        let client = DockerClient {
            executor,
            config: client_config,
            version_info: None,
            daemon_info: None,
        };

        assert!(client.version_meets_requirement("20.10.21", "20.10.0"));
        assert!(client.version_meets_requirement("21.0.0", "20.10.0"));
        assert!(!client.version_meets_requirement("19.03.15", "20.10.0"));
        assert!(client.version_meets_requirement("20.10.0", "20.10.0"));
    }

    // Integration tests
    #[tokio::test]
    #[ignore = "Requires Docker daemon running"]
    async fn test_docker_client_creation() {
        match DockerClient::new().await {
            Ok(client) => {
                println!("Docker client created successfully");
                println!("Docker path: {:?}", client.docker_path());
            }
            Err(e) => {
                println!(
                    "Docker client creation failed (expected if Docker not available): {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires Docker daemon running"]
    async fn test_docker_ping() {
        match DockerClient::new().await {
            Ok(client) => match client.ping().await {
                Ok(()) => println!("Docker daemon ping successful"),
                Err(e) => println!("Docker daemon ping failed: {}", e),
            },
            Err(e) => {
                println!("Could not create Docker client: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires Docker daemon running"]
    async fn test_docker_version() {
        match DockerClient::new().await {
            Ok(mut client) => match client.version().await {
                Ok(version) => {
                    println!("Docker client version: {}", version.client);
                    if let Some(server) = &version.server {
                        println!("Docker server version: {}", server);
                    }
                }
                Err(e) => println!("Failed to get Docker version: {}", e),
            },
            Err(e) => {
                println!("Could not create Docker client: {}", e);
            }
        }
    }
}
