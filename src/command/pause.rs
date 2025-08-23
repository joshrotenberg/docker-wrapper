//! Docker pause command implementation.
//!
//! This module provides the `docker pause` command for pausing all processes within containers.

use super::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;

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
    pub executor: CommandExecutor,
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
impl DockerCommandV2 for PauseCommand {
    type Output = CommandOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["pause".to_string()];
        args.extend(self.containers.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(crate::error::Error::invalid_config(
                "No containers specified for pausing",
            ));
        }

        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
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
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["pause", "test-container"]);
    }

    #[test]
    fn test_pause_multiple_containers() {
        let cmd = PauseCommand::new_multiple(vec!["web", "db", "cache"]);
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["pause", "web", "db", "cache"]);
    }

    #[test]
    fn test_pause_add_container() {
        let cmd = PauseCommand::new("web").container("db").container("cache");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["pause", "web", "db", "cache"]);
    }
}
