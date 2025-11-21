//! Docker network connect command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker network connect command builder
#[derive(Debug, Clone)]
pub struct NetworkConnectCommand {
    /// Network name
    network: String,
    /// Container name or ID
    container: String,
    /// IPv4 address
    ipv4: Option<String>,
    /// IPv6 address
    ipv6: Option<String>,
    /// Container alias
    alias: Vec<String>,
    /// Link to another container
    link: Vec<String>,
    /// Link-local IP addresses
    link_local_ip: Vec<String>,
    /// Driver options
    driver_opt: Vec<(String, String)>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl NetworkConnectCommand {
    /// Create a new network connect command
    #[must_use]
    pub fn new(network: impl Into<String>, container: impl Into<String>) -> Self {
        Self {
            network: network.into(),
            container: container.into(),
            ipv4: None,
            ipv6: None,
            alias: Vec::new(),
            link: Vec::new(),
            link_local_ip: Vec::new(),
            driver_opt: Vec::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Set IPv4 address
    #[must_use]
    pub fn ipv4(mut self, ip: impl Into<String>) -> Self {
        self.ipv4 = Some(ip.into());
        self
    }

    /// Set IPv6 address
    #[must_use]
    pub fn ipv6(mut self, ip: impl Into<String>) -> Self {
        self.ipv6 = Some(ip.into());
        self
    }

    /// Add a network-scoped alias
    #[must_use]
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias.push(alias.into());
        self
    }

    /// Add a link to another container
    #[must_use]
    pub fn link(mut self, container: impl Into<String>) -> Self {
        self.link.push(container.into());
        self
    }

    /// Add a link-local IP address
    #[must_use]
    pub fn link_local_ip(mut self, ip: impl Into<String>) -> Self {
        self.link_local_ip.push(ip.into());
        self
    }

    /// Add a driver option
    #[must_use]
    pub fn driver_opt(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.driver_opt.push((key.into(), value.into()));
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<NetworkConnectResult> {
        self.execute().await.map(NetworkConnectResult::from)
    }
}

#[async_trait]
impl DockerCommand for NetworkConnectCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["network".to_string(), "connect".to_string()];

        if let Some(ref ip) = self.ipv4 {
            args.push("--ip".to_string());
            args.push(ip.clone());
        }

        if let Some(ref ip) = self.ipv6 {
            args.push("--ip6".to_string());
            args.push(ip.clone());
        }

        for alias in &self.alias {
            args.push("--alias".to_string());
            args.push(alias.clone());
        }

        for link in &self.link {
            args.push("--link".to_string());
            args.push(link.clone());
        }

        for ip in &self.link_local_ip {
            args.push("--link-local-ip".to_string());
            args.push(ip.clone());
        }

        for (key, value) in &self.driver_opt {
            args.push("--driver-opt".to_string());
            args.push(format!("{key}={value}"));
        }

        args.push(self.network.clone());
        args.push(self.container.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
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

/// Result from network connect command
#[derive(Debug, Clone)]
pub struct NetworkConnectResult {
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for NetworkConnectResult {
    fn from(output: CommandOutput) -> Self {
        Self { raw_output: output }
    }
}

impl NetworkConnectResult {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.raw_output.success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_connect_basic() {
        let cmd = NetworkConnectCommand::new("my-network", "my-container");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["network", "connect", "my-network", "my-container"]
        );
    }

    #[test]
    fn test_network_connect_with_ip() {
        let cmd = NetworkConnectCommand::new("my-network", "my-container").ipv4("172.20.0.10");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "network",
                "connect",
                "--ip",
                "172.20.0.10",
                "my-network",
                "my-container"
            ]
        );
    }

    #[test]
    fn test_network_connect_with_alias() {
        let cmd = NetworkConnectCommand::new("my-network", "my-container")
            .alias("db")
            .alias("database");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--alias".to_string()));
        assert!(args.contains(&"db".to_string()));
        assert!(args.contains(&"database".to_string()));
    }
}
