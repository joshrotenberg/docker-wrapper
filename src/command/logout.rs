//! Docker logout command implementation
//!
//! This module provides functionality to log out from Docker registries.
//! It supports logging out from specific registries or using the daemon default.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;
use std::fmt;

/// Command for logging out from Docker registries
///
/// The `LogoutCommand` provides a builder pattern for constructing Docker logout commands
/// to remove stored authentication credentials for Docker registries.
///
/// # Examples
///
/// ```rust
/// use docker_wrapper::LogoutCommand;
///
/// // Logout from default registry (daemon-defined)
/// let logout = LogoutCommand::new();
///
/// // Logout from specific registry
/// let logout = LogoutCommand::new()
///     .server("my-registry.com");
/// ```
#[derive(Debug, Clone)]
pub struct LogoutCommand {
    /// Registry server URL (None for daemon default)
    server: Option<String>,
    /// Command executor for running the command
    executor: CommandExecutor,
}

/// Output from a logout command execution
///
/// Contains the raw output from the Docker logout command and provides
/// convenience methods for checking logout status.
#[derive(Debug, Clone)]
pub struct LogoutOutput {
    /// Raw output from the Docker command
    pub output: CommandOutput,
}

impl LogoutCommand {
    /// Creates a new logout command
    ///
    /// By default, logs out from the daemon-defined default registry
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::LogoutCommand;
    ///
    /// let logout = LogoutCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            server: None,
            executor: CommandExecutor::default(),
        }
    }

    /// Sets the registry server to logout from
    ///
    /// If not specified, uses the daemon-defined default registry
    ///
    /// # Arguments
    ///
    /// * `server` - The registry server URL
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::LogoutCommand;
    ///
    /// let logout = LogoutCommand::new()
    ///     .server("gcr.io");
    /// ```
    #[must_use]
    pub fn server(mut self, server: impl Into<String>) -> Self {
        self.server = Some(server.into());
        self
    }

    /// Sets a custom command executor
    ///
    /// # Arguments
    ///
    /// * `executor` - Custom command executor
    #[must_use]
    pub fn executor(mut self, executor: CommandExecutor) -> Self {
        self.executor = executor;
        self
    }

    /// Builds the command arguments for Docker logout
    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["logout".to_string()];

        // Add server if specified
        if let Some(ref server) = self.server {
            args.push(server.clone());
        }

        args
    }

    /// Gets the server (if set)
    #[must_use]
    pub fn get_server(&self) -> Option<&str> {
        self.server.as_deref()
    }
}

impl Default for LogoutCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl LogoutOutput {
    /// Returns true if the logout was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Returns true if the output indicates successful logout
    #[must_use]
    pub fn is_logged_out(&self) -> bool {
        self.success()
            && (self.output.stdout.contains("Removing login credentials")
                || self.output.stdout.contains("Not logged in")
                || self.output.stdout.is_empty() && self.output.stderr.is_empty())
    }

    /// Gets any warning messages from the logout output
    #[must_use]
    pub fn warnings(&self) -> Vec<&str> {
        self.output
            .stderr
            .lines()
            .filter(|line| line.to_lowercase().contains("warning"))
            .collect()
    }

    /// Gets any info messages from the logout output
    #[must_use]
    pub fn info_messages(&self) -> Vec<&str> {
        self.output
            .stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect()
    }
}

#[async_trait]
impl DockerCommand for LogoutCommand {
    type Output = LogoutOutput;

    fn command_name(&self) -> &'static str {
        "logout"
    }

    fn build_args(&self) -> Vec<String> {
        self.build_command_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let output = self
            .executor
            .execute_command(self.command_name(), self.build_args())
            .await?;

        Ok(LogoutOutput { output })
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

impl fmt::Display for LogoutCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "docker logout")?;

        if let Some(ref server) = self.server {
            write!(f, " {server}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logout_command_basic() {
        let logout = LogoutCommand::new();

        assert_eq!(logout.get_server(), None);

        let args = logout.build_command_args();
        assert_eq!(args, vec!["logout"]);
    }

    #[test]
    fn test_logout_command_with_server() {
        let logout = LogoutCommand::new().server("gcr.io");

        assert_eq!(logout.get_server(), Some("gcr.io"));

        let args = logout.build_command_args();
        assert_eq!(args, vec!["logout", "gcr.io"]);
    }

    #[test]
    fn test_logout_command_with_private_registry() {
        let logout = LogoutCommand::new().server("my-registry.example.com:5000");

        let args = logout.build_command_args();
        assert_eq!(args, vec!["logout", "my-registry.example.com:5000"]);
    }

    #[test]
    fn test_logout_command_daemon_default() {
        let logout = LogoutCommand::new();

        // No server specified should use daemon default
        assert_eq!(logout.get_server(), None);

        let args = logout.build_command_args();
        assert_eq!(args, vec!["logout"]);
    }

    #[test]
    fn test_logout_command_display() {
        let logout = LogoutCommand::new().server("example.com");

        let display = format!("{logout}");
        assert_eq!(display, "docker logout example.com");
    }

    #[test]
    fn test_logout_command_display_no_server() {
        let logout = LogoutCommand::new();

        let display = format!("{logout}");
        assert_eq!(display, "docker logout");
    }

    #[test]
    fn test_logout_command_default() {
        let logout = LogoutCommand::default();

        assert_eq!(logout.get_server(), None);
        let args = logout.build_command_args();
        assert_eq!(args, vec!["logout"]);
    }

    #[test]
    fn test_logout_output_success_with_credentials_removal() {
        let output = CommandOutput {
            stdout: "Removing login credentials for https://index.docker.io/v1/".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let logout_output = LogoutOutput { output };

        assert!(logout_output.success());
        assert!(logout_output.is_logged_out());
    }

    #[test]
    fn test_logout_output_success_not_logged_in() {
        let output = CommandOutput {
            stdout: "Not logged in to https://index.docker.io/v1/".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let logout_output = LogoutOutput { output };

        assert!(logout_output.success());
        assert!(logout_output.is_logged_out());
    }

    #[test]
    fn test_logout_output_success_empty() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let logout_output = LogoutOutput { output };

        assert!(logout_output.success());
        assert!(logout_output.is_logged_out());
    }

    #[test]
    fn test_logout_output_warnings() {
        let output = CommandOutput {
            stdout: "Removing login credentials for registry".to_string(),
            stderr: "WARNING: credentials may still be cached\ninfo: using default registry"
                .to_string(),
            exit_code: 0,
            success: true,
        };
        let logout_output = LogoutOutput { output };

        let warnings = logout_output.warnings();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("WARNING"));
    }

    #[test]
    fn test_logout_output_info_messages() {
        let output = CommandOutput {
            stdout: "Removing login credentials for https://registry.example.com\nLogout completed"
                .to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let logout_output = LogoutOutput { output };

        let info = logout_output.info_messages();
        assert_eq!(info.len(), 2);
        assert!(info[0].contains("Removing login credentials"));
        assert!(info[1].contains("Logout completed"));
    }

    #[test]
    fn test_logout_output_failure() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: "Error: unable to logout".to_string(),
            exit_code: 1,
            success: false,
        };
        let logout_output = LogoutOutput { output };

        assert!(!logout_output.success());
        assert!(!logout_output.is_logged_out());
    }

    #[test]
    fn test_logout_command_name() {
        let logout = LogoutCommand::new();
        assert_eq!(logout.command_name(), "logout");
    }

    #[test]
    fn test_logout_command_extensibility() {
        let mut logout = LogoutCommand::new();

        // Test the extension methods for future compatibility
        logout
            .arg("extra")
            .args(vec!["more", "args"])
            .flag("--verbose")
            .option("key", "value");

        // Command should still work normally
        assert_eq!(logout.command_name(), "logout");
    }

    #[test]
    fn test_logout_multiple_servers_concept() {
        // Test that we can create logout commands for different servers
        let daemon_default_logout = LogoutCommand::new();
        let gcr_logout = LogoutCommand::new().server("gcr.io");
        let private_logout = LogoutCommand::new().server("my-registry.com");

        assert_eq!(daemon_default_logout.get_server(), None);
        assert_eq!(gcr_logout.get_server(), Some("gcr.io"));
        assert_eq!(private_logout.get_server(), Some("my-registry.com"));
    }

    #[test]
    fn test_logout_builder_pattern() {
        let logout = LogoutCommand::new().server("registry.example.com");

        assert_eq!(logout.get_server(), Some("registry.example.com"));
        assert_eq!(logout.command_name(), "logout");
    }

    #[test]
    fn test_logout_various_server_formats() {
        let test_cases = vec![
            "gcr.io",
            "registry-1.docker.io",
            "localhost:5000",
            "my-registry.com:443",
            "registry.example.com/path",
        ];

        for server in test_cases {
            let logout = LogoutCommand::new().server(server);
            assert_eq!(logout.get_server(), Some(server));

            let args = logout.build_command_args();
            assert!(args.contains(&server.to_string()));
        }
    }
}
