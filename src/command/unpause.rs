//! Docker unpause command implementation.
//!
//! This module provides the `docker unpause` command for unpausing all processes within containers.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker unpause command builder
///
/// Unpause all processes within one or more containers.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::UnpauseCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Unpause a single container
/// UnpauseCommand::new("my-container")
///     .run()
///     .await?;
///
/// // Unpause multiple containers
/// UnpauseCommand::new_multiple(vec!["web", "db", "cache"])
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct UnpauseCommand {
    /// Container names or IDs to unpause
    containers: Vec<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl UnpauseCommand {
    /// Create a new unpause command for a single container
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UnpauseCommand;
    ///
    /// let cmd = UnpauseCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            containers: vec![container.into()],
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new unpause command for multiple containers
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::UnpauseCommand;
    ///
    /// let cmd = UnpauseCommand::new_multiple(vec!["web", "db", "cache"]);
    /// ```
    #[must_use]
    pub fn new_multiple(containers: Vec<impl Into<String>>) -> Self {
        Self {
            containers: containers.into_iter().map(Into::into).collect(),
            executor: CommandExecutor::new(),
        }
    }

    /// Add another container to unpause
    #[must_use]
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Execute the unpause command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Any of the specified containers don't exist
    /// - Any container is not paused
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::UnpauseCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = UnpauseCommand::new("my-container")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Unpaused {} containers", result.unpaused_containers().len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<UnpauseResult> {
        let output = self.execute().await?;
        Ok(UnpauseResult {
            output,
            containers: self.containers.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for UnpauseCommand {
    type Output = CommandOutput;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["unpause".to_string()];
        args.extend(self.containers.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(crate::error::Error::invalid_config(
                "No containers specified for unpausing",
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

/// Result from the unpause command
#[derive(Debug, Clone)]
pub struct UnpauseResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Containers that were unpaused
    pub containers: Vec<String>,
}

impl UnpauseResult {
    /// Check if the unpause was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the unpaused container names
    #[must_use]
    pub fn unpaused_containers(&self) -> &[String] {
        &self.containers
    }

    /// Get the count of unpaused containers
    #[must_use]
    pub fn unpaused_count(&self) -> usize {
        self.containers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpause_single_container() {
        let cmd = UnpauseCommand::new("test-container");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["unpause", "test-container"]);
    }

    #[test]
    fn test_unpause_multiple_containers() {
        let cmd = UnpauseCommand::new_multiple(vec!["web", "db", "cache"]);
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["unpause", "web", "db", "cache"]);
    }

    #[test]
    fn test_unpause_add_container() {
        let cmd = UnpauseCommand::new("web")
            .container("db")
            .container("cache");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["unpause", "web", "db", "cache"]);
    }
}
