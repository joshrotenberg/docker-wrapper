//! Docker export command implementation.
//!
//! This module provides the `docker export` command for exporting containers to tarballs.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker export command builder
///
/// Export a container's filesystem as a tar archive.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::ExportCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Export container to file
/// let result = ExportCommand::new("my-container")
///     .output("container.tar")
///     .run()
///     .await?;
///
/// if result.success() {
///     println!("Container exported to container.tar");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ExportCommand {
    /// Container name or ID to export
    container: String,
    /// Output file path
    output: Option<String>,
    /// Command executor
    executor: CommandExecutor,
}

impl ExportCommand {
    /// Create a new export command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::ExportCommand;
    ///
    /// let cmd = ExportCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            output: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Set output file for the export
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::ExportCommand;
    ///
    /// let cmd = ExportCommand::new("my-container")
    ///     .output("backup.tar");
    /// ```
    #[must_use]
    pub fn output(mut self, output: impl Into<String>) -> Self {
        self.output = Some(output.into());
        self
    }

    /// Execute the export command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    /// - File I/O errors occur during export
    /// - Insufficient disk space
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::ExportCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = ExportCommand::new("web-server")
    ///     .output("web-backup.tar")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Container '{}' exported to '{}'",
    ///              result.container(), result.output_file().unwrap_or("stdout"));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<ExportResult> {
        let output = self.execute().await?;

        Ok(ExportResult {
            output,
            container: self.container.clone(),
            output_file: self.output.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for ExportCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "export"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref output) = self.output {
            args.push("--output".to_string());
            args.push(output.clone());
        }

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

/// Result from the export command
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Container that was exported
    pub container: String,
    /// Output file path (if specified)
    pub output_file: Option<String>,
}

impl ExportResult {
    /// Check if the export was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the container name
    #[must_use]
    pub fn container(&self) -> &str {
        &self.container
    }

    /// Get the output file path
    #[must_use]
    pub fn output_file(&self) -> Option<&str> {
        self.output_file.as_deref()
    }

    /// Get the raw command output
    #[must_use]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }

    /// Check if export was written to a file
    #[must_use]
    pub fn exported_to_file(&self) -> bool {
        self.output_file.is_some()
    }

    /// Check if export was written to stdout
    #[must_use]
    pub fn exported_to_stdout(&self) -> bool {
        self.output_file.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_basic() {
        let cmd = ExportCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_export_with_output() {
        let cmd = ExportCommand::new("test-container").output("backup.tar");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--output", "backup.tar", "test-container"]);
    }

    #[test]
    fn test_export_with_path() {
        let cmd = ExportCommand::new("web-server").output("/tmp/exports/web.tar");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--output", "/tmp/exports/web.tar", "web-server"]);
    }

    #[test]
    fn test_export_result() {
        let result = ExportResult {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            container: "my-container".to_string(),
            output_file: Some("backup.tar".to_string()),
        };

        assert!(result.success());
        assert_eq!(result.container(), "my-container");
        assert_eq!(result.output_file(), Some("backup.tar"));
        assert!(result.exported_to_file());
        assert!(!result.exported_to_stdout());
    }

    #[test]
    fn test_export_result_stdout() {
        let result = ExportResult {
            output: CommandOutput {
                stdout: "tar data...".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            container: "my-container".to_string(),
            output_file: None,
        };

        assert!(result.success());
        assert_eq!(result.container(), "my-container");
        assert_eq!(result.output_file(), None);
        assert!(!result.exported_to_file());
        assert!(result.exported_to_stdout());
    }

    #[test]
    fn test_command_name() {
        let cmd = ExportCommand::new("container");
        assert_eq!(cmd.command_name(), "export");
    }
}
