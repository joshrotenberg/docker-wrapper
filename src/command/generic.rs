//! Generic Docker command for executing any Docker CLI command.
//!
//! This module provides an escape hatch for running arbitrary Docker commands,
//! including plugin commands, experimental features, or commands not yet
//! implemented in this library.
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::{DockerCommand, GenericCommand};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Run a plugin command
//! let output = GenericCommand::new("scan")
//!     .arg("alpine:latest")
//!     .execute()
//!     .await?;
//!
//! println!("{}", output.stdout);
//!
//! // Run an experimental command
//! let output = GenericCommand::new("debug")
//!     .args(["container-id", "--shell", "bash"])
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Generic Docker command for executing any Docker CLI command.
///
/// This provides an escape hatch for running arbitrary Docker commands that
/// may not have dedicated command types in this library, such as:
///
/// - Plugin commands (e.g., `docker scan`, `docker scout`)
/// - Experimental features
/// - Future commands not yet implemented
/// - Custom or third-party extensions
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, GenericCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Execute a plugin command
/// let result = GenericCommand::new("scout")
///     .arg("cves")
///     .arg("alpine:latest")
///     .execute()
///     .await?;
///
/// if result.success {
///     println!("Scan complete:\n{}", result.stdout);
/// }
///
/// // Execute with multiple arguments
/// let result = GenericCommand::new("trust")
///     .args(["inspect", "--pretty", "alpine:latest"])
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GenericCommand {
    /// The Docker subcommand to execute
    command: String,
    /// Arguments to pass to the command
    args: Vec<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl GenericCommand {
    /// Create a new generic command.
    ///
    /// # Arguments
    ///
    /// * `command` - The Docker subcommand to execute (e.g., "scan", "scout", "trust")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use docker_wrapper::GenericCommand;
    ///
    /// let cmd = GenericCommand::new("scan");
    /// ```
    #[must_use]
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Add a single argument to the command.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use docker_wrapper::GenericCommand;
    ///
    /// let cmd = GenericCommand::new("scan")
    ///     .arg("--severity")
    ///     .arg("high")
    ///     .arg("alpine:latest");
    /// ```
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments to the command.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use docker_wrapper::GenericCommand;
    ///
    /// let cmd = GenericCommand::new("scout")
    ///     .args(["cves", "--format", "json", "alpine:latest"]);
    /// ```
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Build the command arguments.
    fn build_args(&self) -> Vec<String> {
        let mut args = vec![self.command.clone()];
        args.extend(self.args.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }
}

#[async_trait]
impl DockerCommand for GenericCommand {
    type Output = CommandOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.execute_command(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_command_basic() {
        let cmd = GenericCommand::new("scan");
        let args = cmd.build_args();
        assert_eq!(args, vec!["scan"]);
    }

    #[test]
    fn test_generic_command_with_arg() {
        let cmd = GenericCommand::new("scan").arg("alpine:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["scan", "alpine:latest"]);
    }

    #[test]
    fn test_generic_command_with_multiple_args() {
        let cmd = GenericCommand::new("scan")
            .arg("--severity")
            .arg("high")
            .arg("alpine:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["scan", "--severity", "high", "alpine:latest"]);
    }

    #[test]
    fn test_generic_command_with_args_iterator() {
        let cmd = GenericCommand::new("scout").args(["cves", "--format", "json", "alpine:latest"]);
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["scout", "cves", "--format", "json", "alpine:latest"]
        );
    }

    #[test]
    fn test_generic_command_complex() {
        let cmd = GenericCommand::new("trust")
            .arg("inspect")
            .args(["--pretty", "alpine:latest"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["trust", "inspect", "--pretty", "alpine:latest"]);
    }

    #[test]
    fn test_generic_command_subcommand_with_spaces() {
        // Test multi-word commands like "container ls"
        let cmd = GenericCommand::new("container").args(["ls", "-a"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["container", "ls", "-a"]);
    }
}
