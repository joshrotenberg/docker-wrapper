//! Docker kill command implementation.
//!
//! This module provides the `docker kill` command for sending signals to running containers.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker kill command builder
#[derive(Debug, Clone)]
pub struct KillCommand {
    /// Container names or IDs to kill
    containers: Vec<String>,
    /// Signal to send (default: SIGKILL)
    signal: Option<String>,
    /// Command executor
    executor: CommandExecutor,
}

impl KillCommand {
    /// Create a new kill command for a single container
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            containers: vec![container.into()],
            signal: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new kill command for multiple containers
    #[must_use]
    pub fn new_multiple(containers: Vec<impl Into<String>>) -> Self {
        Self {
            containers: containers.into_iter().map(Into::into).collect(),
            signal: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another container to kill
    #[must_use]
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Set the signal to send (e.g., "SIGTERM", "SIGKILL", "9")
    #[must_use]
    pub fn signal(mut self, signal: impl Into<String>) -> Self {
        self.signal = Some(signal.into());
        self
    }

    /// Execute the kill command
    ///
    /// # Errors
    /// Returns an error if:
    /// - No containers are specified
    /// - The Docker daemon is not running
    /// - Any of the specified containers don't exist
    /// - The signal is invalid
    pub async fn run(&self) -> Result<KillResult> {
        let output = self.execute().await?;

        // Parse killed container IDs from output
        let killed: Vec<String> = output
            .stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();

        Ok(KillResult {
            killed,
            signal: self.signal.clone(),
            output,
        })
    }
}

#[async_trait]
impl DockerCommand for KillCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "kill"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref sig) = self.signal {
            args.push("--signal".to_string());
            args.push(sig.clone());
        }

        // Add container names/IDs
        args.extend(self.containers.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(Error::invalid_config("No containers specified for kill"));
        }

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

/// Result from the kill command
#[derive(Debug, Clone)]
pub struct KillResult {
    /// List of killed container IDs
    pub killed: Vec<String>,
    /// Signal that was sent
    pub signal: Option<String>,
    /// Raw command output
    pub output: CommandOutput,
}

impl KillResult {
    /// Check if all containers were killed successfully
    #[must_use]
    pub fn all_killed(&self) -> bool {
        self.output.success
    }

    /// Get the number of containers killed
    #[must_use]
    pub fn count(&self) -> usize {
        self.killed.len()
    }

    /// Get the signal that was sent
    #[must_use]
    pub fn signal_sent(&self) -> &str {
        self.signal.as_deref().unwrap_or("SIGKILL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_single_container() {
        let cmd = KillCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_kill_multiple_containers() {
        let cmd = KillCommand::new_multiple(vec!["container1", "container2", "container3"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["container1", "container2", "container3"]);
    }

    #[test]
    fn test_kill_with_signal() {
        let cmd = KillCommand::new("test-container").signal("SIGTERM");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--signal", "SIGTERM", "test-container"]);
    }

    #[test]
    fn test_kill_with_numeric_signal() {
        let cmd = KillCommand::new("test-container").signal("9");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--signal", "9", "test-container"]);
    }

    #[test]
    fn test_kill_builder_chain() {
        let cmd = KillCommand::new("container1")
            .container("container2")
            .container("container3")
            .signal("SIGTERM");
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "--signal",
                "SIGTERM",
                "container1",
                "container2",
                "container3"
            ]
        );
    }
}
