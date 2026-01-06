//! Docker port command implementation.
//!
//! This module provides the `docker port` command for listing port mappings.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

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
    pub executor: CommandExecutor,
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

        // Parse port mappings from output, passing the queried port for simple format parsing
        let port_mappings = Self::parse_port_mappings(&output.stdout, self.port);

        Ok(PortResult {
            output,
            container: self.container.clone(),
            port_mappings,
        })
    }

    /// Parse port mappings from command output.
    ///
    /// Handles two formats:
    /// - Full format (all ports): `80/tcp -> 0.0.0.0:8080`
    /// - Simple format (specific port query): `0.0.0.0:8080`
    ///
    /// When `queried_port` is provided and the simple format is detected,
    /// the container port is inferred from the queried port.
    fn parse_port_mappings(stdout: &str, queried_port: Option<u16>) -> Vec<PortMapping> {
        let mut mappings = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Try full format first: "80/tcp -> 0.0.0.0:8080"
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
            } else if let Some(container_port) = queried_port {
                // Simple format (specific port query): "0.0.0.0:8080" or "[::]:8080"
                if let Some((host_ip, host_port_str)) = line.rsplit_once(':') {
                    if let Ok(host_port) = host_port_str.parse::<u16>() {
                        mappings.push(PortMapping {
                            container_port,
                            host_ip: host_ip.to_string(),
                            host_port,
                            protocol: "tcp".to_string(), // Default to tcp when not specified
                        });
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

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["port".to_string(), self.container.clone()];

        if let Some(port) = self.port {
            args.push(port.to_string());
        }

        args.extend(self.executor.raw_args.clone());
        args
    }

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
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
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["port", "test-container"]);
    }

    #[test]
    fn test_port_with_specific_port() {
        let cmd = PortCommand::new("test-container").port(80);
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["port", "test-container", "80"]);
    }

    #[test]
    fn test_parse_port_mappings_full_format() {
        let output = "80/tcp -> 0.0.0.0:8080\n443/tcp -> 127.0.0.1:8443";
        let mappings = PortCommand::parse_port_mappings(output, None);

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
    fn test_parse_port_mappings_simple_format() {
        // Format returned when querying a specific port: docker port <container> 6379
        let output = "0.0.0.0:40998\n[::]:40998";
        let mappings = PortCommand::parse_port_mappings(output, Some(6379));

        assert_eq!(mappings.len(), 2);

        // IPv4 mapping
        assert_eq!(mappings[0].container_port, 6379);
        assert_eq!(mappings[0].host_ip, "0.0.0.0");
        assert_eq!(mappings[0].host_port, 40998);
        assert_eq!(mappings[0].protocol, "tcp");

        // IPv6 mapping
        assert_eq!(mappings[1].container_port, 6379);
        assert_eq!(mappings[1].host_ip, "[::]");
        assert_eq!(mappings[1].host_port, 40998);
        assert_eq!(mappings[1].protocol, "tcp");
    }

    #[test]
    fn test_parse_port_mappings_simple_format_without_queried_port() {
        // Without queried_port, simple format lines are ignored
        let output = "0.0.0.0:40998\n[::]:40998";
        let mappings = PortCommand::parse_port_mappings(output, None);

        assert!(mappings.is_empty());
    }

    #[test]
    fn test_parse_port_mappings_empty() {
        let mappings = PortCommand::parse_port_mappings("", None);
        assert!(mappings.is_empty());
    }

    #[test]
    fn test_parse_port_mappings_mixed_format() {
        // In practice this wouldn't happen, but test robustness
        let output = "80/tcp -> 0.0.0.0:8080\n0.0.0.0:9000";
        let mappings = PortCommand::parse_port_mappings(output, Some(443));

        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].container_port, 80);
        assert_eq!(mappings[0].host_port, 8080);
        assert_eq!(mappings[1].container_port, 443);
        assert_eq!(mappings[1].host_port, 9000);
    }
}
