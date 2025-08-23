//! Docker login command implementation
//!
//! This module provides functionality to authenticate with Docker registries.
//! It supports both Docker Hub and private registries with various authentication methods.

use super::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;
use std::fmt;

/// Command for authenticating with Docker registries
///
/// The `LoginCommand` provides a builder pattern for constructing Docker login commands
/// with various authentication options including username/password, token-based auth,
/// and different registry endpoints.
///
/// # Examples
///
/// ```rust
/// use docker_wrapper::LoginCommand;
///
/// // Login to Docker Hub
/// let login = LoginCommand::new("myusername", "mypassword");
///
/// // Login to private registry
/// let login = LoginCommand::new("user", "pass")
///     .registry("my-registry.com");
///
/// // Login with stdin password (more secure)
/// let login = LoginCommand::new("user", "")
///     .password_stdin();
/// ```
#[derive(Debug, Clone)]
pub struct LoginCommand {
    /// Username for authentication
    username: String,
    /// Password for authentication (empty if using stdin)
    password: String,
    /// Registry server URL (defaults to Docker Hub if None)
    registry: Option<String>,
    /// Whether to read password from stdin
    password_stdin: bool,
    /// Command executor for running the command
    pub executor: CommandExecutor,
}

/// Output from a login command execution
///
/// Contains the raw output from the Docker login command and provides
/// convenience methods for checking authentication status.
#[derive(Debug, Clone)]
pub struct LoginOutput {
    /// Raw output from the Docker command
    pub output: CommandOutput,
}

impl LoginCommand {
    /// Creates a new login command with username and password
    ///
    /// # Arguments
    ///
    /// * `username` - The username for authentication
    /// * `password` - The password for authentication (can be empty if using stdin)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::LoginCommand;
    ///
    /// let login = LoginCommand::new("myuser", "mypass");
    /// ```
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            registry: None,
            password_stdin: false,
            executor: CommandExecutor::default(),
        }
    }

    /// Sets the registry server for authentication
    ///
    /// If not specified, defaults to Docker Hub (index.docker.io)
    ///
    /// # Arguments
    ///
    /// * `registry` - The registry server URL
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::LoginCommand;
    ///
    /// let login = LoginCommand::new("user", "pass")
    ///     .registry("gcr.io");
    /// ```
    #[must_use]
    pub fn registry(mut self, registry: impl Into<String>) -> Self {
        self.registry = Some(registry.into());
        self
    }

    /// Enables reading password from stdin for security
    ///
    /// When enabled, the password field is ignored and Docker will
    /// prompt for password input via stdin.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use docker_wrapper::LoginCommand;
    ///
    /// let login = LoginCommand::new("user", "")
    ///     .password_stdin();
    /// ```
    #[must_use]
    pub fn password_stdin(mut self) -> Self {
        self.password_stdin = true;
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

    /// Gets the username
    #[must_use]
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Gets the registry (if set)
    #[must_use]
    pub fn get_registry(&self) -> Option<&str> {
        self.registry.as_deref()
    }

    /// Returns true if password will be read from stdin
    #[must_use]
    pub fn is_password_stdin(&self) -> bool {
        self.password_stdin
    }

    /// Get a reference to the command executor
    #[must_use]
    pub fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    /// Get a mutable reference to the command executor
    #[must_use]
    pub fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
}

impl Default for LoginCommand {
    fn default() -> Self {
        Self::new("", "")
    }
}

impl LoginOutput {
    /// Returns true if the login was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Returns true if the output indicates successful authentication
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.success()
            && (self.output.stdout.contains("Login Succeeded")
                || self.output.stdout.contains("login succeeded"))
    }

    /// Gets any warning messages from the login output
    #[must_use]
    pub fn warnings(&self) -> Vec<&str> {
        self.output
            .stderr
            .lines()
            .filter(|line| line.to_lowercase().contains("warning"))
            .collect()
    }
}

#[async_trait]
impl DockerCommandV2 for LoginCommand {
    type Output = LoginOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["login".to_string()];

        // Add username
        args.push("--username".to_string());
        args.push(self.username.clone());

        // Add password option
        if self.password_stdin {
            args.push("--password-stdin".to_string());
        } else {
            args.push("--password".to_string());
            args.push(self.password.clone());
        }

        // Add registry if specified
        if let Some(ref registry) = self.registry {
            args.push(registry.clone());
        }

        // Add raw args from executor
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let output = self.executor.execute_command("docker", args).await?;

        Ok(LoginOutput { output })
    }
}

impl fmt::Display for LoginCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "docker login")?;

        if let Some(ref registry) = self.registry {
            write!(f, " {registry}")?;
        }

        write!(f, " --username {}", self.username)?;

        if self.password_stdin {
            write!(f, " --password-stdin")?;
        } else {
            write!(f, " --password [HIDDEN]")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_command_basic() {
        let login = LoginCommand::new("testuser", "testpass");

        assert_eq!(login.get_username(), "testuser");
        assert_eq!(login.get_registry(), None);
        assert!(!login.is_password_stdin());

        let args = login.build_command_args();
        assert_eq!(
            args,
            vec!["login", "--username", "testuser", "--password", "testpass"]
        );
    }

    #[test]
    fn test_login_command_with_registry() {
        let login = LoginCommand::new("user", "pass").registry("gcr.io");

        assert_eq!(login.get_registry(), Some("gcr.io"));

        let args = login.build_command_args();
        assert_eq!(
            args,
            vec![
                "login",
                "--username",
                "user",
                "--password",
                "pass",
                "gcr.io"
            ]
        );
    }

    #[test]
    fn test_login_command_password_stdin() {
        let login = LoginCommand::new("user", "ignored").password_stdin();

        assert!(login.is_password_stdin());

        let args = login.build_command_args();
        assert_eq!(
            args,
            vec!["login", "--username", "user", "--password-stdin"]
        );
    }

    #[test]
    fn test_login_command_with_private_registry() {
        let login = LoginCommand::new("admin", "secret").registry("my-registry.example.com:5000");

        let args = login.build_command_args();
        assert_eq!(
            args,
            vec![
                "login",
                "--username",
                "admin",
                "--password",
                "secret",
                "my-registry.example.com:5000"
            ]
        );
    }

    #[test]
    fn test_login_command_docker_hub_default() {
        let login = LoginCommand::new("dockeruser", "dockerpass");

        // No registry specified should default to Docker Hub
        assert_eq!(login.get_registry(), None);

        let args = login.build_command_args();
        assert!(!args.contains(&"index.docker.io".to_string()));
    }

    #[test]
    fn test_login_command_display() {
        let login = LoginCommand::new("testuser", "testpass").registry("example.com");

        let display = format!("{login}");
        assert!(display.contains("docker login"));
        assert!(display.contains("example.com"));
        assert!(display.contains("--username testuser"));
        assert!(display.contains("--password [HIDDEN]"));
        assert!(!display.contains("testpass"));
    }

    #[test]
    fn test_login_command_display_stdin() {
        let login = LoginCommand::new("testuser", "").password_stdin();

        let display = format!("{login}");
        assert!(display.contains("--password-stdin"));
        assert!(!display.contains("[HIDDEN]"));
    }

    #[test]
    fn test_login_command_default() {
        let login = LoginCommand::default();

        assert_eq!(login.get_username(), "");
        assert_eq!(login.get_registry(), None);
        assert!(!login.is_password_stdin());
    }

    #[test]
    fn test_login_output_success_detection() {
        let output = CommandOutput {
            stdout: "Login Succeeded".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let login_output = LoginOutput { output };

        assert!(login_output.success());
        assert!(login_output.is_authenticated());
    }

    #[test]
    fn test_login_output_alternative_success_message() {
        let output = CommandOutput {
            stdout: "login succeeded for user@registry".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let login_output = LoginOutput { output };

        assert!(login_output.is_authenticated());
    }

    #[test]
    fn test_login_output_warnings() {
        let output = CommandOutput {
            stdout: "Login Succeeded".to_string(),
            stderr: "WARNING: login credentials saved in plaintext\ninfo: using default registry"
                .to_string(),
            exit_code: 0,
            success: true,
        };
        let login_output = LoginOutput { output };

        let warnings = login_output.warnings();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("WARNING"));
    }

    #[test]
    fn test_login_output_failure() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: "Error: authentication failed".to_string(),
            exit_code: 1,
            success: false,
        };
        let login_output = LoginOutput { output };

        assert!(!login_output.success());
        assert!(!login_output.is_authenticated());
    }
}
