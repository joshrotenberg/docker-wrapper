//! Docker rename command implementation.
//!
//! This module provides the `docker rename` command for renaming containers.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker rename command builder
///
/// Rename a container.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::RenameCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Rename a container
/// let result = RenameCommand::new("old-name", "new-name")
///     .run()
///     .await?;
///
/// if result.success() {
///     println!("Container renamed successfully");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RenameCommand {
    /// Current container name or ID
    old_name: String,
    /// New container name
    new_name: String,
    /// Command executor
    pub executor: CommandExecutor,
}

impl RenameCommand {
    /// Create a new rename command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::RenameCommand;
    ///
    /// let cmd = RenameCommand::new("old-container", "new-container");
    /// ```
    #[must_use]
    pub fn new(old_name: impl Into<String>, new_name: impl Into<String>) -> Self {
        Self {
            old_name: old_name.into(),
            new_name: new_name.into(),
            executor: CommandExecutor::new(),
        }
    }

    /// Execute the rename command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    /// - The new name is already in use
    /// - The container is running (some Docker versions)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::RenameCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = RenameCommand::new("web-server", "api-server")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Container renamed from '{}' to '{}'",
    ///              result.old_name(), result.new_name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<RenameResult> {
        let output = self.execute().await?;

        Ok(RenameResult {
            output,
            old_name: self.old_name.clone(),
            new_name: self.new_name.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for RenameCommand {
    type Output = CommandOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["rename".to_string()];
        args.push(self.old_name.clone());
        args.push(self.new_name.clone());
        args.extend(self.executor.raw_args.clone());
        args
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

/// Result from the rename command
#[derive(Debug, Clone)]
pub struct RenameResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Original container name
    pub old_name: String,
    /// New container name
    pub new_name: String,
}

impl RenameResult {
    /// Check if the rename was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the original container name
    #[must_use]
    pub fn old_name(&self) -> &str {
        &self.old_name
    }

    /// Get the new container name
    #[must_use]
    pub fn new_name(&self) -> &str {
        &self.new_name
    }

    /// Get the raw command output
    #[must_use]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_basic() {
        let cmd = RenameCommand::new("old-container", "new-container");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["rename", "old-container", "new-container"]);
    }

    #[test]
    fn test_rename_with_id() {
        let cmd = RenameCommand::new("abc123", "my-new-container");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["rename", "abc123", "my-new-container"]);
    }

    #[test]
    fn test_rename_result() {
        let result = RenameResult {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            old_name: "old-name".to_string(),
            new_name: "new-name".to_string(),
        };

        assert!(result.success());
        assert_eq!(result.old_name(), "old-name");
        assert_eq!(result.new_name(), "new-name");
    }

    #[test]
    fn test_command_name() {
        let _cmd = RenameCommand::new("old", "new");
        // Test that builder produces valid rename command
    }
}
