//! Docker attach command implementation.
//!
//! This module provides the `docker attach` command for attaching to a running container.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker attach command builder
///
/// Attach local standard input, output, and error streams to a running container.
///
/// # Example
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
    executor: CommandExecutor,
}

impl AttachCommand {
    /// Create a new attach command
    ///
    /// # Example
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
    /// # Example
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
    /// # Example
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
    /// # Example
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
    /// # Example
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

    fn command_name(&self) -> &'static str {
        "attach"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

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

        args
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
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_attach_with_detach_keys() {
        let cmd = AttachCommand::new("test-container").detach_keys("ctrl-a,ctrl-d");
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["--detach-keys", "ctrl-a,ctrl-d", "test-container"]
        );
    }

    #[test]
    fn test_attach_no_stdin() {
        let cmd = AttachCommand::new("test-container").no_stdin();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--no-stdin", "test-container"]);
    }

    #[test]
    fn test_attach_no_sig_proxy() {
        let cmd = AttachCommand::new("test-container").no_sig_proxy();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--sig-proxy=false", "test-container"]);
    }

    #[test]
    fn test_attach_all_options() {
        let cmd = AttachCommand::new("test-container")
            .detach_keys("ctrl-x,ctrl-y")
            .no_stdin()
            .no_sig_proxy();
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "--detach-keys",
                "ctrl-x,ctrl-y",
                "--no-stdin",
                "--sig-proxy=false",
                "test-container"
            ]
        );
    }
}
