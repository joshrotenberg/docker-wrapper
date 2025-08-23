//! Docker Compose port command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose port command
///
/// Print the public port for a port binding.
#[derive(Debug, Clone, Default)]
pub struct ComposePortCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Service name
    pub service: String,
    /// Private port number
    pub private_port: u16,
    /// Protocol (tcp/udp)
    pub protocol: Option<String>,
    /// Index of the container (if service has multiple instances)
    pub index: Option<u32>,
}

/// Result from port command
#[derive(Debug, Clone)]
pub struct PortResult {
    /// The public port binding (host:port)
    pub binding: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposePortCommand {
    /// Create a new port command
    #[must_use]
    pub fn new(service: impl Into<String>, private_port: u16) -> Self {
        Self {
            service: service.into(),
            private_port,
            ..Default::default()
        }
    }

    /// Add a compose file
    #[must_use]
    pub fn file<P: Into<std::path::PathBuf>>(mut self, file: P) -> Self {
        self.config.files.push(file.into());
        self
    }

    /// Set project name
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config.project_name = Some(name.into());
        self
    }

    /// Set protocol (tcp/udp)
    #[must_use]
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Set container index
    #[must_use]
    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["port".to_string()];

        // Add index if specified
        if let Some(index) = self.index {
            args.push("--index".to_string());
            args.push(index.to_string());
        }

        // Add protocol if specified
        if let Some(protocol) = &self.protocol {
            args.push("--protocol".to_string());
            args.push(protocol.clone());
        }

        // Add service and port
        args.push(self.service.clone());
        args.push(self.private_port.to_string());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposePortCommand {
    type Output = PortResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(PortResult {
            binding: output.stdout.trim().to_string(),
            success: output.success,
        })
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.execute_compose(args).await
    }
}

impl PortResult {
    /// Parse the binding into host and port
    #[must_use]
    pub fn parse_binding(&self) -> Option<(String, u16)> {
        let parts: Vec<&str> = self.binding.split(':').collect();
        if parts.len() == 2 {
            if let Ok(port) = parts[1].parse::<u16>() {
                return Some((parts[0].to_string(), port));
            }
        }
        None
    }

    /// Get just the port number
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        self.parse_binding().map(|(_, port)| port)
    }

    /// Get just the host
    #[must_use]
    pub fn host(&self) -> Option<String> {
        self.parse_binding().map(|(host, _)| host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_command_basic() {
        let cmd = ComposePortCommand::new("web", 80);
        let args = cmd.build_args();
        assert_eq!(args[0], "port");
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"80".to_string()));
    }

    #[test]
    fn test_port_command_with_protocol() {
        let cmd = ComposePortCommand::new("web", 53).protocol("udp");
        let args = cmd.build_args();
        assert!(args.contains(&"--protocol".to_string()));
        assert!(args.contains(&"udp".to_string()));
    }

    #[test]
    fn test_port_command_with_index() {
        let cmd = ComposePortCommand::new("web", 8080).index(2);
        let args = cmd.build_args();
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"2".to_string()));
    }

    #[test]
    fn test_port_result_parsing() {
        let result = PortResult {
            binding: "0.0.0.0:32768".to_string(),
            success: true,
        };

        assert_eq!(result.port(), Some(32768));
        assert_eq!(result.host(), Some("0.0.0.0".to_string()));

        let (host, port) = result.parse_binding().unwrap();
        assert_eq!(host, "0.0.0.0");
        assert_eq!(port, 32768);
    }
}
