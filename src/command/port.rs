//! Docker port command implementation.
//!
//! This module provides the `docker port` command for listing port mappings.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker port command builder
///
/// List port mappings or a specific mapping for a container.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::PortCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // List all port mappings
/// let ports = PortCommand::new("my-container")
///     .run()
///     .await?;
///
/// // Get specific port mapping
/// let port = PortCommand::new("my-container")
///     .port(80)
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PortCommand {
    /// Container name or ID
    container: String,
    /// Specific port to query
    port: Option<u16>,
    /// Command executor
    executor: CommandExecutor,
}

impl PortCommand {
    /// Create a new port command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::PortCommand;
    ///
    /// let cmd = PortCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            port: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Query specific port mapping
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::PortCommand;
    ///
    /// let cmd = PortCommand::new("my-container")
    ///     .port(80);
    /// ```
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Execute the port command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::PortCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = PortCommand::new("my-container")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     for mapping in result.port_mappings() {
    ///         println!("{}:{} -> {}", mapping.host_ip, mapping.host_port, mapping.container_port);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<PortResult> {
        let output = self.execute().await?;

        // Parse port mappings from output
        let port_mappings = Self::parse_port_mappings(&output.stdout);

        Ok(PortResult {
            output,
            container: self.container.clone(),
            port_mappings,
        })
    }

    /// Parse port mappings from command output
    fn parse_port_mappings(stdout: &str) -> Vec<PortMapping> {
        let mut mappings = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Format: "80/tcp -> 0.0.0.0:8080"
            if let Some((container_part, host_part)) = line.split_once(" -> ") {
                if let Some((port_str, protocol)) = container_part.split_once('/') {
                    if let Ok(container_port) = port_str.parse::<u16>() {
                        if let Some((host_ip, host_port_str)) = host_part.rsplit_once(':') {
                            if let Ok(host_port) = host_port_str.parse::<u16>() {
                                mappings.push(PortMapping {
                                    container_port,
                                    host_ip: host_ip.to_string(),
                                    host_port,
                                    protocol: protocol.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        mappings
    }
}

#[async_trait]
impl DockerCommand for PortCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "port"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec![self.container.clone()];

        if let Some(port) = self.port {
            args.push(port.to_string());
        }

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command(self.command_name(), self.build_args())
            .await
    }

    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.executor.add_arg(arg);
        self
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.executor.add_args(args);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.executor.add_flag(flag);
        self
    }

    fn option(&mut self, key: &str, value: &str) -> &mut Self {
        self.executor.add_option(key, value);
        self
    }
}

/// Result from the port command
#[derive(Debug, Clone)]
pub struct PortResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Container that was queried
    pub container: String,
    /// Parsed port mappings
    pub port_mappings: Vec<PortMapping>,
}

impl PortResult {
    /// Check if the port command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the container name
    #[must_use]
    pub fn container(&self) -> &str {
        &self.container
    }

    /// Get the port mappings
    #[must_use]
    pub fn port_mappings(&self) -> &[PortMapping] {
        &self.port_mappings
    }

    /// Get port mapping count
    #[must_use]
    pub fn mapping_count(&self) -> usize {
        self.port_mappings.len()
    }
}

/// Port mapping information
#[derive(Debug, Clone)]
pub struct PortMapping {
    /// Container port
    pub container_port: u16,
    /// Host IP address
    pub host_ip: String,
    /// Host port
    pub host_port: u16,
    /// Protocol (tcp/udp)
    pub protocol: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_basic() {
        let cmd = PortCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_port_with_specific_port() {
        let cmd = PortCommand::new("test-container").port(80);
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container", "80"]);
    }

    #[test]
    fn test_parse_port_mappings() {
        let output = "80/tcp -> 0.0.0.0:8080\n443/tcp -> 127.0.0.1:8443";
        let mappings = PortCommand::parse_port_mappings(output);

        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].container_port, 80);
        assert_eq!(mappings[0].host_ip, "0.0.0.0");
        assert_eq!(mappings[0].host_port, 8080);
        assert_eq!(mappings[0].protocol, "tcp");

        assert_eq!(mappings[1].container_port, 443);
        assert_eq!(mappings[1].host_ip, "127.0.0.1");
        assert_eq!(mappings[1].host_port, 8443);
        assert_eq!(mappings[1].protocol, "tcp");
    }

    #[test]
    fn test_parse_port_mappings_empty() {
        let mappings = PortCommand::parse_port_mappings("");
        assert!(mappings.is_empty());
    }
}
