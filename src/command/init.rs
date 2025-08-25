//! Docker init command implementation
//!
//! This module provides the `docker init` command for initializing projects with
//! Docker-related starter files. The init command creates a Dockerfile, compose.yaml,
//! .dockerignore, and README.Docker.md files based on a selected template.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::fmt;

/// Docker init command builder
///
/// Initialize a project with files necessary to run the project in a container.
/// Creates a Dockerfile, compose.yaml, .dockerignore, and README.Docker.md.
///
/// # Examples
///
/// ```no_run
/// use docker_wrapper::{DockerCommand, InitCommand};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Initialize with default interactive mode
/// InitCommand::new()
///     .execute()
///     .await?;
///
/// // Show version information
/// InitCommand::new()
///     .version()
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct InitCommand {
    /// Show version information
    show_version: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

/// Available template types for Docker init
#[derive(Debug, Clone, PartialEq)]
pub enum InitTemplate {
    /// ASP.NET Core application
    AspNetCore,
    /// Go application  
    Go,
    /// Java application
    Java,
    /// Node.js application
    Node,
    /// PHP with Apache application
    Php,
    /// Python application
    Python,
    /// Rust application
    Rust,
    /// General purpose / other
    Other,
}

/// Result from the init command execution
#[derive(Debug, Clone)]
pub struct InitOutput {
    /// Raw command output
    pub output: CommandOutput,
    /// Whether version was requested
    pub version_requested: bool,
}

impl InitCommand {
    /// Create a new init command
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::InitCommand;
    ///
    /// let cmd = InitCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            show_version: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Show version information instead of initializing
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::InitCommand;
    ///
    /// let cmd = InitCommand::new().version();
    /// ```
    #[must_use]
    pub fn version(mut self) -> Self {
        self.show_version = true;
        self
    }

    /// Check if version flag is set
    #[must_use]
    pub fn is_version(&self) -> bool {
        self.show_version
    }
}

impl Default for InitCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for InitCommand {
    type Output = InitOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["init".to_string()];

        if self.show_version {
            args.push("--version".to_string());
        }

        // Add any additional raw arguments
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let output = self.execute_command(args).await?;

        Ok(InitOutput {
            output,
            version_requested: self.show_version,
        })
    }
}

impl InitOutput {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the version string (if version was requested)
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        if self.version_requested && self.success() {
            Some(self.output.stdout.trim())
        } else {
            None
        }
    }

    /// Check if this was a version request
    #[must_use]
    pub fn is_version_output(&self) -> bool {
        self.version_requested
    }

    /// Check if files were likely created (interactive mode completed successfully)
    #[must_use]
    pub fn files_created(&self) -> bool {
        !self.version_requested && self.success()
    }

    /// Get raw stdout output
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.output.stdout
    }

    /// Get raw stderr output  
    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.output.stderr
    }
}

impl fmt::Display for InitCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "docker init")?;

        if self.show_version {
            write!(f, " --version")?;
        }

        Ok(())
    }
}

impl fmt::Display for InitTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::AspNetCore => "ASP.NET Core",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::Node => "Node",
            Self::Php => "PHP with Apache",
            Self::Python => "Python",
            Self::Rust => "Rust",
            Self::Other => "Other",
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_command_basic() {
        let cmd = InitCommand::new();
        assert!(!cmd.is_version());

        let args = cmd.build_command_args();
        assert_eq!(args, vec!["init"]);
    }

    #[test]
    fn test_init_command_version() {
        let cmd = InitCommand::new().version();
        assert!(cmd.is_version());

        let args = cmd.build_command_args();
        assert_eq!(args, vec!["init", "--version"]);
    }

    #[test]
    fn test_init_command_default() {
        let cmd = InitCommand::default();
        assert!(!cmd.is_version());

        let args = cmd.build_command_args();
        assert_eq!(args, vec!["init"]);
    }

    #[test]
    fn test_init_command_display() {
        let cmd = InitCommand::new();
        assert_eq!(format!("{cmd}"), "docker init");

        let cmd_version = InitCommand::new().version();
        assert_eq!(format!("{cmd_version}"), "docker init --version");
    }

    #[test]
    fn test_init_template_display() {
        assert_eq!(format!("{}", InitTemplate::AspNetCore), "ASP.NET Core");
        assert_eq!(format!("{}", InitTemplate::Go), "Go");
        assert_eq!(format!("{}", InitTemplate::Java), "Java");
        assert_eq!(format!("{}", InitTemplate::Node), "Node");
        assert_eq!(format!("{}", InitTemplate::Php), "PHP with Apache");
        assert_eq!(format!("{}", InitTemplate::Python), "Python");
        assert_eq!(format!("{}", InitTemplate::Rust), "Rust");
        assert_eq!(format!("{}", InitTemplate::Other), "Other");
    }

    #[test]
    fn test_init_output_helpers() {
        let output = InitOutput {
            output: CommandOutput {
                stdout: "Version: v1.4.0".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            version_requested: true,
        };

        assert!(output.success());
        assert!(output.is_version_output());
        assert!(!output.files_created());
        assert_eq!(output.version(), Some("Version: v1.4.0"));
        assert_eq!(output.stdout(), "Version: v1.4.0");
    }

    #[test]
    fn test_init_output_files_created() {
        let output = InitOutput {
            output: CommandOutput {
                stdout: "Files created successfully".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            version_requested: false,
        };

        assert!(output.success());
        assert!(!output.is_version_output());
        assert!(output.files_created());
        assert_eq!(output.version(), None);
    }

    #[test]
    fn test_init_command_extensibility() {
        let mut cmd = InitCommand::new();

        // Test that we can add custom raw arguments
        cmd.get_executor_mut()
            .raw_args
            .push("--custom-flag".to_string());

        let args = cmd.build_command_args();
        assert!(args.contains(&"--custom-flag".to_string()));
    }
}
