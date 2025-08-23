//! Docker network disconnect command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;

/// Docker network disconnect command builder
#[derive(Debug, Clone)]
pub struct NetworkDisconnectCommand {
    /// Network name
    network: String,
    /// Container name or ID
    container: String,
    /// Force disconnection
    force: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl NetworkDisconnectCommand {
    /// Create a new network disconnect command
    #[must_use]
    pub fn new(network: impl Into<String>, container: impl Into<String>) -> Self {
        Self {
            network: network.into(),
            container: container.into(),
            force: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Force disconnection
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
    pub async fn run(&self) -> Result<NetworkDisconnectResult> {
        self.execute().await.map(NetworkDisconnectResult::from)
    }
}

#[async_trait]
impl DockerCommandV2 for NetworkDisconnectCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["network".to_string(), "disconnect".to_string()];

        if self.force {
            args.push("--force".to_string());
        }

        args.push(self.network.clone());
        args.push(self.container.clone());
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

/// Result from network disconnect command
#[derive(Debug, Clone)]
pub struct NetworkDisconnectResult {
    /// Raw command output
    pub raw_output: CommandOutput,
}

impl From<CommandOutput> for NetworkDisconnectResult {
    fn from(output: CommandOutput) -> Self {
        Self { raw_output: output }
    }
}

impl NetworkDisconnectResult {
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
    fn test_network_disconnect_basic() {
        let cmd = NetworkDisconnectCommand::new("my-network", "my-container");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["network", "disconnect", "my-network", "my-container"]
        );
    }

    #[test]
    fn test_network_disconnect_force() {
        let cmd = NetworkDisconnectCommand::new("my-network", "my-container").force();
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "network",
                "disconnect",
                "--force",
                "my-network",
                "my-container"
            ]
        );
    }
}
