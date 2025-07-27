//! Docker restart command implementation.
//!
//! This module provides a comprehensive implementation of the `docker restart` command
//! with support for all native options and an extensible architecture.

use super::{CommandExecutor, DockerCommand};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::ffi::OsStr;
use std::time::Duration;

/// Docker restart command builder with fluent API
#[derive(Debug, Clone)]
pub struct RestartCommand {
    /// Command executor for extensibility
    executor: CommandExecutor,
    /// Container IDs or names to restart
    containers: Vec<String>,
    /// Signal to send to the container
    signal: Option<String>,
    /// Seconds to wait before killing the container
    timeout: Option<u32>,
}

/// Result of a restart command execution
#[derive(Debug, Clone, PartialEq)]
pub struct RestartResult {
    /// Raw stdout from the command
    pub stdout: String,
    /// Raw stderr from the command
    pub stderr: String,
    /// Container IDs that were restarted
    pub restarted_containers: Vec<String>,
}

impl RestartCommand {
    /// Create a new restart command for the specified container(s)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RestartCommand;
    ///
    /// let cmd = RestartCommand::new("my-container");
    /// ```
    ///
    /// ```
    /// use docker_wrapper::RestartCommand;
    ///
    /// let cmd = RestartCommand::new_multiple(vec!["container1", "container2"]);
    /// ```
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            executor: CommandExecutor::new(),
            containers: vec![container.into()],
            signal: None,
            timeout: None,
        }
    }

    /// Create a new restart command for multiple containers
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RestartCommand;
    ///
    /// let cmd = RestartCommand::new_multiple(vec!["container1", "container2"]);
    /// ```
    pub fn new_multiple<I, S>(containers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            executor: CommandExecutor::new(),
            containers: containers.into_iter().map(Into::into).collect(),
            signal: None,
            timeout: None,
        }
    }

    /// Set the signal to send to the container
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RestartCommand;
    ///
    /// let cmd = RestartCommand::new("my-container")
    ///     .signal("SIGTERM");
    /// ```
    #[must_use]
    pub fn signal(mut self, signal: impl Into<String>) -> Self {
        self.signal = Some(signal.into());
        self
    }

    /// Set the timeout in seconds to wait before killing the container
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RestartCommand;
    ///
    /// let cmd = RestartCommand::new("my-container")
    ///     .timeout(30);
    /// ```
    #[must_use]
    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the timeout using a Duration
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::RestartCommand;
    /// use std::time::Duration;
    ///
    /// let cmd = RestartCommand::new("my-container")
    ///     .timeout_duration(Duration::from_secs(30));
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn timeout_duration(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout.as_secs().min(u64::from(u32::MAX)) as u32);
        self
    }
}

#[async_trait]
impl DockerCommand for RestartCommand {
    type Output = RestartResult;

    fn command_name(&self) -> &'static str {
        "restart"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add signal option
        if let Some(signal) = &self.signal {
            args.push("--signal".to_string());
            args.push(signal.clone());
        }

        // Add timeout option
        if let Some(timeout) = self.timeout {
            args.push("--timeout".to_string());
            args.push(timeout.to_string());
        }

        // Add container names/IDs
        args.extend(self.containers.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(Error::invalid_config("No containers specified"));
        }

        let args = self.build_args();
        let output = self
            .executor
            .execute_command(self.command_name(), args)
            .await?;

        // Parse the output to extract restarted container IDs
        let restarted_containers = if output.stdout.trim().is_empty() {
            // If no stdout, assume the containers specified were restarted
            self.containers.clone()
        } else {
            // Parse container IDs from stdout (each line is a container ID)
            output
                .stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect()
        };

        Ok(RestartResult {
            stdout: output.stdout,
            stderr: output.stderr,
            restarted_containers,
        })
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

impl RestartCommand {
    /// Get the command arguments (for testing)
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        let mut args = vec!["restart".to_string()];
        args.extend(self.build_args());
        args
    }
}

impl RestartResult {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.restarted_containers.is_empty()
    }

    /// Get the number of containers that were restarted
    #[must_use]
    pub fn container_count(&self) -> usize {
        self.restarted_containers.len()
    }

    /// Get the first restarted container ID (useful for single container operations)
    #[must_use]
    pub fn first_container(&self) -> Option<&String> {
        self.restarted_containers.first()
    }

    /// Check if a specific container was restarted
    #[must_use]
    pub fn contains_container(&self, container: &str) -> bool {
        self.restarted_containers.iter().any(|c| c == container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restart_command_new() {
        let cmd = RestartCommand::new("test-container");
        assert_eq!(cmd.containers, vec!["test-container"]);
        assert!(cmd.signal.is_none());
        assert!(cmd.timeout.is_none());
    }

    #[test]
    fn test_restart_command_new_multiple() {
        let cmd = RestartCommand::new_multiple(vec!["container1", "container2"]);
        assert_eq!(cmd.containers, vec!["container1", "container2"]);
    }

    #[test]
    fn test_restart_command_with_signal() {
        let cmd = RestartCommand::new("test-container").signal("SIGKILL");
        assert_eq!(cmd.signal, Some("SIGKILL".to_string()));
    }

    #[test]
    fn test_restart_command_with_timeout() {
        let cmd = RestartCommand::new("test-container").timeout(30);
        assert_eq!(cmd.timeout, Some(30));
    }

    #[test]
    fn test_restart_command_with_timeout_duration() {
        let cmd = RestartCommand::new("test-container").timeout_duration(Duration::from_secs(45));
        assert_eq!(cmd.timeout, Some(45));
    }

    #[test]
    fn test_restart_command_args_basic() {
        let cmd = RestartCommand::new("test-container");
        let args = cmd.args();
        assert_eq!(args, vec!["restart", "test-container"]);
    }

    #[test]
    fn test_restart_command_args_with_options() {
        let cmd = RestartCommand::new("test-container")
            .signal("SIGTERM")
            .timeout(30);
        let args = cmd.args();
        assert_eq!(
            args,
            vec![
                "restart",
                "--signal",
                "SIGTERM",
                "--timeout",
                "30",
                "test-container"
            ]
        );
    }

    #[test]
    fn test_restart_command_args_multiple_containers() {
        let cmd = RestartCommand::new_multiple(vec!["container1", "container2"]).timeout(10);
        let args = cmd.args();
        assert_eq!(
            args,
            vec!["restart", "--timeout", "10", "container1", "container2"]
        );
    }

    #[test]
    fn test_restart_result_is_success() {
        let result = RestartResult {
            stdout: "container1\n".to_string(),
            stderr: String::new(),
            restarted_containers: vec!["container1".to_string()],
        };
        assert!(result.is_success());

        let empty_result = RestartResult {
            stdout: String::new(),
            stderr: String::new(),
            restarted_containers: vec![],
        };
        assert!(!empty_result.is_success());
    }

    #[test]
    fn test_restart_result_container_count() {
        let result = RestartResult {
            stdout: String::new(),
            stderr: String::new(),
            restarted_containers: vec!["container1".to_string(), "container2".to_string()],
        };
        assert_eq!(result.container_count(), 2);
    }

    #[test]
    fn test_restart_result_first_container() {
        let result = RestartResult {
            stdout: String::new(),
            stderr: String::new(),
            restarted_containers: vec!["container1".to_string(), "container2".to_string()],
        };
        assert_eq!(result.first_container(), Some(&"container1".to_string()));

        let empty_result = RestartResult {
            stdout: String::new(),
            stderr: String::new(),
            restarted_containers: vec![],
        };
        assert_eq!(empty_result.first_container(), None);
    }

    #[test]
    fn test_restart_result_contains_container() {
        let result = RestartResult {
            stdout: String::new(),
            stderr: String::new(),
            restarted_containers: vec!["container1".to_string(), "container2".to_string()],
        };
        assert!(result.contains_container("container1"));
        assert!(result.contains_container("container2"));
        assert!(!result.contains_container("container3"));
    }

    #[test]
    fn test_command_name() {
        let cmd = RestartCommand::new("test");
        assert_eq!(cmd.command_name(), "restart");
    }
}
