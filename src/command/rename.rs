//! Docker rename command implementation.
//!
//! This module provides the `docker rename` command for renaming containers.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

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
    executor: CommandExecutor,
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

    fn command_name(&self) -> &'static str {
        "rename"
    }

    fn build_args(&self) -> Vec<String> {
        vec![self.old_name.clone(), self.new_name.clone()]
    }

    async fn execute(&self) -> Result<Self::Output> {
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
        let args = cmd.build_args();
        assert_eq!(args, vec!["old-container", "new-container"]);
    }

    #[test]
    fn test_rename_with_id() {
        let cmd = RenameCommand::new("abc123", "my-new-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["abc123", "my-new-container"]);
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
        let cmd = RenameCommand::new("old", "new");
        assert_eq!(cmd.command_name(), "rename");
    }
}
