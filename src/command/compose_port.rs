//! Docker Compose port command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose port command builder
#[derive(Debug, Clone)]
pub struct ComposePortCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Service name
    pub service: String,
    /// Private port to query
    pub private_port: Option<u16>,
    /// Protocol (tcp/udp)
    pub protocol: Option<String>,
    /// Index of container if service has multiple instances
    pub index: Option<u16>,
}

/// Result from compose port command
#[derive(Debug, Clone)]
pub struct ComposePortResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Service that was queried
    pub service: String,
    /// Port mappings found
    pub port_mappings: Vec<String>,
}

impl ComposePortCommand {
    /// Create a new compose port command
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            service: service.into(),
            private_port: None,
            protocol: None,
            index: None,
        }
    }

    /// Set private port to query
    #[must_use]
    pub fn private_port(mut self, port: u16) -> Self {
        self.private_port = Some(port);
        self
    }

    /// Set protocol (tcp or udp)
    #[must_use]
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Set container index if service has multiple instances
    #[must_use]
    pub fn index(mut self, index: u16) -> Self {
        self.index = Some(index);
        self
    }
}

#[async_trait]
impl DockerCommandV2 for ComposePortCommand {
    type Output = ComposePortResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        let port_mappings = output
            .stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        Ok(ComposePortResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            service: self.service.clone(),
            port_mappings,
        })
    }
}

impl ComposeCommand for ComposePortCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "port"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(protocol) = &self.protocol {
            args.push("--protocol".to_string());
            args.push(protocol.clone());
        }

        if let Some(index) = self.index {
            args.push("--index".to_string());
            args.push(index.to_string());
        }

        args.push(self.service.clone());

        if let Some(port) = self.private_port {
            args.push(port.to_string());
        }

        args
    }
}

impl ComposePortResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the service that was queried
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }

    /// Get port mappings
    #[must_use]
    pub fn port_mappings(&self) -> &[String] {
        &self.port_mappings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_port_basic() {
        let cmd = ComposePortCommand::new("web");
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"port".to_string()));
    }

    #[test]
    fn test_compose_port_with_options() {
        let cmd = ComposePortCommand::new("api")
            .private_port(8080)
            .protocol("tcp")
            .index(1);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--protocol".to_string()));
        assert!(args.contains(&"tcp".to_string()));
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"1".to_string()));
        assert!(args.contains(&"api".to_string()));
        assert!(args.contains(&"8080".to_string()));
    }
}
