//! Docker pause command implementation.
//!
//! This module provides the `docker pause` command for pausing all processes within containers.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker pause command builder
///
/// Pause all processes within one or more containers.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::PauseCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Pause a single container
/// PauseCommand::new("my-container")
///     .run()
///     .await?;
///
/// // Pause multiple containers
/// PauseCommand::new_multiple(vec!["web", "db", "cache"])
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PauseCommand {
    /// Container names or IDs to pause
    containers: Vec<String>,
    /// Command executor
    executor: CommandExecutor,
}

impl PauseCommand {
    /// Create a new pause command for a single container
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::PauseCommand;
    ///
    /// let cmd = PauseCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            containers: vec![container.into()],
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new pause command for multiple containers
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::PauseCommand;
    ///
    /// let cmd = PauseCommand::new_multiple(vec!["web", "db", "cache"]);
    /// ```
    #[must_use]
    pub fn new_multiple(containers: Vec<impl Into<String>>) -> Self {
        Self {
            containers: containers.into_iter().map(Into::into).collect(),
            executor: CommandExecutor::new(),
        }
    }

    /// Add another container to pause
    #[must_use]
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Execute the pause command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Any of the specified containers don't exist
    /// - Any container is not running
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::PauseCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = PauseCommand::new("my-container")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Paused {} containers", result.paused_containers().len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<PauseResult> {
        let output = self.execute().await?;
        Ok(PauseResult {
            output,
            containers: self.containers.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for PauseCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "pause"
    }

    fn build_args(&self) -> Vec<String> {
        self.containers.clone()
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(crate::error::Error::invalid_config(
                "No containers specified for pausing",
            ));
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

/// Result from the pause command
#[derive(Debug, Clone)]
pub struct PauseResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Containers that were paused
    pub containers: Vec<String>,
}

impl PauseResult {
    /// Check if the pause was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the paused container names
    #[must_use]
    pub fn paused_containers(&self) -> &[String] {
        &self.containers
    }

    /// Get the count of paused containers
    #[must_use]
    pub fn paused_count(&self) -> usize {
        self.containers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_single_container() {
        let cmd = PauseCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_pause_multiple_containers() {
        let cmd = PauseCommand::new_multiple(vec!["web", "db", "cache"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "db", "cache"]);
    }

    #[test]
    fn test_pause_add_container() {
        let cmd = PauseCommand::new("web").container("db").container("cache");
        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "db", "cache"]);
    }
}
