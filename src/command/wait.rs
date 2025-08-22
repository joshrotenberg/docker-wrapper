//! Docker wait command implementation.
//!
//! This module provides the `docker wait` command for waiting until containers stop.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker wait command builder
///
/// Block until one or more containers stop, then print their exit codes.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::WaitCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Wait for a single container
/// let result = WaitCommand::new("my-container")
///     .run()
///     .await?;
///
/// println!("Exit code: {}", result.exit_codes()[0]);
///
/// // Wait for multiple containers
/// let result = WaitCommand::new_multiple(vec!["web", "db"])
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct WaitCommand {
    /// Container names or IDs to wait for
    containers: Vec<String>,
    /// Command executor
    executor: CommandExecutor,
}

impl WaitCommand {
    /// Create a new wait command for a single container
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::WaitCommand;
    ///
    /// let cmd = WaitCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            containers: vec![container.into()],
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new wait command for multiple containers
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::WaitCommand;
    ///
    /// let cmd = WaitCommand::new_multiple(vec!["web", "db", "cache"]);
    /// ```
    #[must_use]
    pub fn new_multiple(containers: Vec<impl Into<String>>) -> Self {
        Self {
            containers: containers.into_iter().map(Into::into).collect(),
            executor: CommandExecutor::new(),
        }
    }

    /// Add another container to wait for
    #[must_use]
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Execute the wait command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Any of the specified containers don't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::WaitCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = WaitCommand::new("my-container")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Container exited with code: {}", result.exit_codes()[0]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<WaitResult> {
        let output = self.execute().await?;

        // Parse exit codes from output
        let exit_codes = Self::parse_exit_codes(&output.stdout);

        Ok(WaitResult {
            output,
            containers: self.containers.clone(),
            exit_codes,
        })
    }

    /// Parse exit codes from command output
    fn parse_exit_codes(stdout: &str) -> Vec<i32> {
        stdout
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect()
    }
}

#[async_trait]
impl DockerCommand for WaitCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "wait"
    }

    fn build_args(&self) -> Vec<String> {
        self.containers.clone()
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(crate::error::Error::invalid_config(
                "No containers specified for waiting",
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

/// Result from the wait command
#[derive(Debug, Clone)]
pub struct WaitResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Containers that were waited for
    pub containers: Vec<String>,
    /// Exit codes from the containers
    pub exit_codes: Vec<i32>,
}

impl WaitResult {
    /// Check if the wait was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the waited container names
    #[must_use]
    pub fn containers(&self) -> &[String] {
        &self.containers
    }

    /// Get the exit codes
    #[must_use]
    pub fn exit_codes(&self) -> &[i32] {
        &self.exit_codes
    }

    /// Check if all containers exited successfully (exit code 0)
    #[must_use]
    pub fn all_successful(&self) -> bool {
        self.exit_codes.iter().all(|&code| code == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_single_container() {
        let cmd = WaitCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_wait_multiple_containers() {
        let cmd = WaitCommand::new_multiple(vec!["web", "db", "cache"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "db", "cache"]);
    }

    #[test]
    fn test_wait_add_container() {
        let cmd = WaitCommand::new("web").container("db").container("cache");
        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "db", "cache"]);
    }

    #[test]
    fn test_parse_exit_codes() {
        let output = "0\n1\n130";
        let codes = WaitCommand::parse_exit_codes(output);
        assert_eq!(codes, vec![0, 1, 130]);
    }

    #[test]
    fn test_parse_exit_codes_empty() {
        let codes = WaitCommand::parse_exit_codes("");
        assert!(codes.is_empty());
    }

    #[test]
    fn test_all_successful() {
        let result = WaitResult {
            output: CommandOutput {
                stdout: "0\n0".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            containers: vec!["web".to_string(), "db".to_string()],
            exit_codes: vec![0, 0],
        };
        assert!(result.all_successful());

        let result_with_failure = WaitResult {
            output: CommandOutput {
                stdout: "0\n1".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            containers: vec!["web".to_string(), "db".to_string()],
            exit_codes: vec![0, 1],
        };
        assert!(!result_with_failure.all_successful());
    }
}
