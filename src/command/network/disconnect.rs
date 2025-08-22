//! Docker network disconnect command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

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
    executor: CommandExecutor,
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
impl DockerCommand for NetworkDisconnectCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "network disconnect"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["disconnect".to_string()];

        if self.force {
            args.push("--force".to_string());
        }

        args.push(self.network.clone());
        args.push(self.container.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command("network", self.build_args())
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
        let args = cmd.build_args();
        assert_eq!(args, vec!["disconnect", "my-network", "my-container"]);
    }

    #[test]
    fn test_network_disconnect_force() {
        let cmd = NetworkDisconnectCommand::new("my-network", "my-container").force();
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["disconnect", "--force", "my-network", "my-container"]
        );
    }
}
