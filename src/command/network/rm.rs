//! Docker network rm command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker network rm command builder
#[derive(Debug, Clone)]
pub struct NetworkRmCommand {
    /// Networks to remove
    networks: Vec<String>,
    /// Force removal
    force: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl NetworkRmCommand {
    /// Create a new network rm command
    #[must_use]
    pub fn new(network: impl Into<String>) -> Self {
        Self {
            networks: vec![network.into()],
            force: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new network rm command for multiple networks
    #[must_use]
    pub fn new_multiple(networks: Vec<String>) -> Self {
        Self {
            networks,
            force: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add a network to remove
    #[must_use]
    pub fn add_network(mut self, network: impl Into<String>) -> Self {
        self.networks.push(network.into());
        self
    }

    /// Force removal
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker daemon is not running or the command fails
    pub async fn run(&self) -> Result<NetworkRmResult> {
        self.execute().await.map(NetworkRmResult::from)
    }
}

#[async_trait]
impl DockerCommand for NetworkRmCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["network".to_string(), "rm".to_string()];

        if self.force {
            args.push("--force".to_string());
        }

        for network in &self.networks {
            args.push(network.clone());
        }

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

/// Result from network rm command
#[derive(Debug, Clone)]
pub struct NetworkRmResult {
    /// Removed networks
    pub removed_networks: Vec<String>,
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for NetworkRmResult {
    fn from(output: CommandOutput) -> Self {
        let removed_networks = output
            .stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();

        Self {
            removed_networks,
            raw_output: output,
        }
    }
}

impl NetworkRmResult {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.raw_output.success
    }

    /// Get count of removed networks
    #[must_use]
    pub fn count(&self) -> usize {
        self.removed_networks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_rm_single() {
        let cmd = NetworkRmCommand::new("my-network");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "rm", "my-network"]);
    }

    #[test]
    fn test_network_rm_multiple() {
        let cmd =
            NetworkRmCommand::new_multiple(vec!["network1".to_string(), "network2".to_string()]);
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "rm", "network1", "network2"]);
    }

    #[test]
    fn test_network_rm_force() {
        let cmd = NetworkRmCommand::new("my-network").force();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["network", "rm", "--force", "my-network"]);
    }
}
