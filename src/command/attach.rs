//! Docker attach command implementation.
//!
//! This module provides the `docker attach` command for attaching to a running container.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker attach command builder
///
/// Attach local standard input, output, and error streams to a running container.
///
/// # Examples
///
/// ```no_run
/// use docker_wrapper::AttachCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Attach to a running container
/// AttachCommand::new("my-container")
///     .run()
///     .await?;
///
/// // Attach without stdin
/// AttachCommand::new("my-container")
///     .no_stdin()
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AttachCommand {
    /// Container name or ID
    container: String,
    /// Override the key sequence for detaching
    detach_keys: Option<String>,
    /// Do not attach STDIN
    no_stdin: bool,
    /// Proxy all received signals to the process
    sig_proxy: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl AttachCommand {
    /// Create a new attach command
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::AttachCommand;
    ///
    /// let cmd = AttachCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            detach_keys: None,
            no_stdin: false,
            sig_proxy: true, // Docker default is true
            executor: CommandExecutor::new(),
        }
    }

    /// Override the key sequence for detaching a container
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::AttachCommand;
    ///
    /// let cmd = AttachCommand::new("my-container")
    ///     .detach_keys("ctrl-a,ctrl-d");
    /// ```
    #[must_use]
    pub fn detach_keys(mut self, keys: impl Into<String>) -> Self {
        self.detach_keys = Some(keys.into());
        self
    }

    /// Do not attach STDIN
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::AttachCommand;
    ///
    /// let cmd = AttachCommand::new("my-container")
    ///     .no_stdin();
    /// ```
    #[must_use]
    pub fn no_stdin(mut self) -> Self {
        self.no_stdin = true;
        self
    }

    /// Do not proxy signals
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::AttachCommand;
    ///
    /// let cmd = AttachCommand::new("my-container")
    ///     .no_sig_proxy();
    /// ```
    #[must_use]
    pub fn no_sig_proxy(mut self) -> Self {
        self.sig_proxy = false;
        self
    }

    /// Execute the attach command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    /// - The container is not running
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use docker_wrapper::AttachCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = AttachCommand::new("my-container")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Successfully attached to container");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<AttachResult> {
        let output = self.execute().await?;
        Ok(AttachResult {
            output,
            container: self.container.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for AttachCommand {
    type Output = CommandOutput;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["attach".to_string()];

        if let Some(ref keys) = self.detach_keys {
            args.push("--detach-keys".to_string());
            args.push(keys.clone());
        }

        if self.no_stdin {
            args.push("--no-stdin".to_string());
        }

        if !self.sig_proxy {
            args.push("--sig-proxy=false".to_string());
        }

        // Add container name/ID
        args.push(self.container.clone());

        // Add raw arguments from executor
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        self.execute_command(args).await
    }
}

/// Result from the attach command
#[derive(Debug, Clone)]
pub struct AttachResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Container that was attached to
    pub container: String,
}

impl AttachResult {
    /// Check if the attach was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the container name
    #[must_use]
    pub fn container(&self) -> &str {
        &self.container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_basic() {
        let cmd = AttachCommand::new("test-container");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["attach", "test-container"]);
    }

    #[test]
    fn test_attach_with_detach_keys() {
        let cmd = AttachCommand::new("test-container").detach_keys("ctrl-a,ctrl-d");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["attach", "--detach-keys", "ctrl-a,ctrl-d", "test-container"]
        );
    }

    #[test]
    fn test_attach_no_stdin() {
        let cmd = AttachCommand::new("test-container").no_stdin();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["attach", "--no-stdin", "test-container"]);
    }

    #[test]
    fn test_attach_no_sig_proxy() {
        let cmd = AttachCommand::new("test-container").no_sig_proxy();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["attach", "--sig-proxy=false", "test-container"]);
    }

    #[test]
    fn test_attach_all_options() {
        let cmd = AttachCommand::new("test-container")
            .detach_keys("ctrl-x,ctrl-y")
            .no_stdin()
            .no_sig_proxy();
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "attach",
                "--detach-keys",
                "ctrl-x,ctrl-y",
                "--no-stdin",
                "--sig-proxy=false",
                "test-container"
            ]
        );
    }
}
