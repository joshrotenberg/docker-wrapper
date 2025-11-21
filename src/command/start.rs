//! Docker start command implementation.
//!
//! This module provides a comprehensive implementation of the `docker start` command
//! with support for all native options and an extensible architecture.

use super::{CommandExecutor, DockerCommand};
use crate::error::{Error, Result};
use async_trait::async_trait;

/// Docker start command builder with fluent API
#[derive(Debug, Clone)]
pub struct StartCommand {
    /// Command executor for extensibility
    pub executor: CommandExecutor,
    /// Container IDs or names to start
    containers: Vec<String>,
    /// Attach STDOUT/STDERR and forward signals
    attach: bool,
    /// Restore from this checkpoint
    checkpoint: Option<String>,
    /// Use a custom checkpoint storage directory
    checkpoint_dir: Option<String>,
    /// Override the key sequence for detaching a container
    detach_keys: Option<String>,
    /// Attach container's STDIN
    interactive: bool,
}

/// Result of a start command execution
#[derive(Debug, Clone, PartialEq)]
pub struct StartResult {
    /// Raw stdout from the command
    pub stdout: String,
    /// Raw stderr from the command
    pub stderr: String,
    /// Container IDs that were started
    pub started_containers: Vec<String>,
}

impl StartCommand {
    /// Create a new start command for the specified container(s)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new("my-container");
    /// ```
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new_multiple(vec!["container1", "container2"]);
    /// ```
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            executor: CommandExecutor::new(),
            containers: vec![container.into()],
            attach: false,
            checkpoint: None,
            checkpoint_dir: None,
            detach_keys: None,
            interactive: false,
        }
    }

    /// Create a new start command for multiple containers
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new_multiple(vec!["container1", "container2"]);
    /// ```
    pub fn new_multiple<I, S>(containers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            executor: CommandExecutor::new(),
            containers: containers.into_iter().map(Into::into).collect(),
            attach: false,
            checkpoint: None,
            checkpoint_dir: None,
            detach_keys: None,
            interactive: false,
        }
    }

    /// Attach STDOUT/STDERR and forward signals
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new("my-container")
    ///     .attach();
    /// ```
    #[must_use]
    pub fn attach(mut self) -> Self {
        self.attach = true;
        self
    }

    /// Restore from this checkpoint
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new("my-container")
    ///     .checkpoint("checkpoint1");
    /// ```
    #[must_use]
    pub fn checkpoint(mut self, checkpoint: impl Into<String>) -> Self {
        self.checkpoint = Some(checkpoint.into());
        self
    }

    /// Use a custom checkpoint storage directory
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new("my-container")
    ///     .checkpoint_dir("/custom/checkpoint/dir");
    /// ```
    #[must_use]
    pub fn checkpoint_dir(mut self, dir: impl Into<String>) -> Self {
        self.checkpoint_dir = Some(dir.into());
        self
    }

    /// Override the key sequence for detaching a container
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new("my-container")
    ///     .detach_keys("ctrl-p,ctrl-q");
    /// ```
    #[must_use]
    pub fn detach_keys(mut self, keys: impl Into<String>) -> Self {
        self.detach_keys = Some(keys.into());
        self
    }

    /// Attach container's STDIN
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new("my-container")
    ///     .interactive();
    /// ```
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// Convenience method for interactive + attach mode
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::StartCommand;
    ///
    /// let cmd = StartCommand::new("my-container")
    ///     .ai(); // attach + interactive
    /// ```
    #[must_use]
    pub fn ai(self) -> Self {
        self.attach().interactive()
    }
}

#[async_trait]
impl DockerCommand for StartCommand {
    type Output = StartResult;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["start".to_string()];

        // Add attach option
        if self.attach {
            args.push("--attach".to_string());
        }

        // Add checkpoint option
        if let Some(checkpoint) = &self.checkpoint {
            args.push("--checkpoint".to_string());
            args.push(checkpoint.clone());
        }

        // Add checkpoint-dir option
        if let Some(checkpoint_dir) = &self.checkpoint_dir {
            args.push("--checkpoint-dir".to_string());
            args.push(checkpoint_dir.clone());
        }

        // Add detach-keys option
        if let Some(detach_keys) = &self.detach_keys {
            args.push("--detach-keys".to_string());
            args.push(detach_keys.clone());
        }

        // Add interactive option
        if self.interactive {
            args.push("--interactive".to_string());
        }

        // Add container names/IDs
        args.extend(self.containers.clone());

        // Add raw arguments from executor
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(Error::invalid_config("No containers specified"));
        }

        let args = self.build_command_args();
        let output = self.execute_command(args).await?;

        // Parse the output to extract started container IDs
        let started_containers = if output.stdout.trim().is_empty() {
            // If no stdout, assume the containers specified were started
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

        Ok(StartResult {
            stdout: output.stdout,
            stderr: output.stderr,
            started_containers,
        })
    }
}

impl StartCommand {
    /// Get the command arguments (for testing)
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        self.build_command_args()
    }
}

impl StartResult {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.started_containers.is_empty()
    }

    /// Get the number of containers that were started
    #[must_use]
    pub fn container_count(&self) -> usize {
        self.started_containers.len()
    }

    /// Get the first started container ID (useful for single container operations)
    #[must_use]
    pub fn first_container(&self) -> Option<&String> {
        self.started_containers.first()
    }

    /// Check if a specific container was started
    #[must_use]
    pub fn contains_container(&self, container: &str) -> bool {
        self.started_containers.iter().any(|c| c == container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_command_new() {
        let cmd = StartCommand::new("test-container");
        assert_eq!(cmd.containers, vec!["test-container"]);
        assert!(!cmd.attach);
        assert!(cmd.checkpoint.is_none());
        assert!(cmd.checkpoint_dir.is_none());
        assert!(cmd.detach_keys.is_none());
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_start_command_new_multiple() {
        let cmd = StartCommand::new_multiple(vec!["container1", "container2"]);
        assert_eq!(cmd.containers, vec!["container1", "container2"]);
    }

    #[test]
    fn test_start_command_with_attach() {
        let cmd = StartCommand::new("test-container").attach();
        assert!(cmd.attach);
    }

    #[test]
    fn test_start_command_with_checkpoint() {
        let cmd = StartCommand::new("test-container").checkpoint("checkpoint1");
        assert_eq!(cmd.checkpoint, Some("checkpoint1".to_string()));
    }

    #[test]
    fn test_start_command_with_checkpoint_dir() {
        let cmd = StartCommand::new("test-container").checkpoint_dir("/custom/dir");
        assert_eq!(cmd.checkpoint_dir, Some("/custom/dir".to_string()));
    }

    #[test]
    fn test_start_command_with_detach_keys() {
        let cmd = StartCommand::new("test-container").detach_keys("ctrl-p,ctrl-q");
        assert_eq!(cmd.detach_keys, Some("ctrl-p,ctrl-q".to_string()));
    }

    #[test]
    fn test_start_command_with_interactive() {
        let cmd = StartCommand::new("test-container").interactive();
        assert!(cmd.interactive);
    }

    #[test]
    fn test_start_command_ai_convenience() {
        let cmd = StartCommand::new("test-container").ai();
        assert!(cmd.attach);
        assert!(cmd.interactive);
    }

    #[test]
    fn test_start_command_args_basic() {
        let cmd = StartCommand::new("test-container");
        let args = cmd.args();
        assert_eq!(args, vec!["start", "test-container"]);
    }

    #[test]
    fn test_start_command_args_with_options() {
        let cmd = StartCommand::new("test-container")
            .attach()
            .interactive()
            .checkpoint("checkpoint1");
        let args = cmd.args();
        assert_eq!(
            args,
            vec![
                "start",
                "--attach",
                "--checkpoint",
                "checkpoint1",
                "--interactive",
                "test-container"
            ]
        );
    }

    #[test]
    fn test_start_command_args_multiple_containers() {
        let cmd =
            StartCommand::new_multiple(vec!["container1", "container2"]).detach_keys("ctrl-c");
        let args = cmd.args();
        assert_eq!(
            args,
            vec![
                "start",
                "--detach-keys",
                "ctrl-c",
                "container1",
                "container2"
            ]
        );
    }

    #[test]
    fn test_start_result_is_success() {
        let result = StartResult {
            stdout: "container1\n".to_string(),
            stderr: String::new(),
            started_containers: vec!["container1".to_string()],
        };
        assert!(result.is_success());

        let empty_result = StartResult {
            stdout: String::new(),
            stderr: String::new(),
            started_containers: vec![],
        };
        assert!(!empty_result.is_success());
    }

    #[test]
    fn test_start_result_container_count() {
        let result = StartResult {
            stdout: String::new(),
            stderr: String::new(),
            started_containers: vec!["container1".to_string(), "container2".to_string()],
        };
        assert_eq!(result.container_count(), 2);
    }

    #[test]
    fn test_start_result_first_container() {
        let result = StartResult {
            stdout: String::new(),
            stderr: String::new(),
            started_containers: vec!["container1".to_string(), "container2".to_string()],
        };
        assert_eq!(result.first_container(), Some(&"container1".to_string()));

        let empty_result = StartResult {
            stdout: String::new(),
            stderr: String::new(),
            started_containers: vec![],
        };
        assert_eq!(empty_result.first_container(), None);
    }

    #[test]
    fn test_start_result_contains_container() {
        let result = StartResult {
            stdout: String::new(),
            stderr: String::new(),
            started_containers: vec!["container1".to_string(), "container2".to_string()],
        };
        assert!(result.contains_container("container1"));
        assert!(result.contains_container("container2"));
        assert!(!result.contains_container("container3"));
    }

    #[test]
    fn test_command_name() {
        let cmd = StartCommand::new("test");
        let args = cmd.build_command_args();
        assert_eq!(args[0], "start");
    }
}
