//! Docker exec command implementation.
//!
//! This module provides a comprehensive implementation of the `docker exec` command
//! with support for all native options and an extensible architecture for any additional options.

use super::{CommandExecutor, DockerCommand, EnvironmentBuilder};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Docker exec command builder with fluent API
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ExecCommand {
    /// The container to execute the command in
    container: String,
    /// The command to execute
    command: Vec<String>,
    /// Command executor for extensibility
    executor: CommandExecutor,
    /// Run in detached mode
    detach: bool,
    /// Override the key sequence for detaching a container
    detach_keys: Option<String>,
    /// Environment variables
    environment: EnvironmentBuilder,
    /// Environment files
    env_files: Vec<String>,
    /// Keep STDIN open even if not attached
    interactive: bool,
    /// Give extended privileges to the command
    privileged: bool,
    /// Allocate a pseudo-TTY
    tty: bool,
    /// Username or UID (format: "<name|uid>[:<group|gid>]")
    user: Option<String>,
    /// Working directory inside the container
    workdir: Option<PathBuf>,
}

/// Output from docker exec command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutput {
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code of the executed command
    pub exit_code: i32,
}

impl ExecOutput {
    /// Check if the command executed successfully
    #[must_use]
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get combined output (stdout + stderr)
    #[must_use]
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }

    /// Check if stdout is empty (ignoring whitespace)
    #[must_use]
    pub fn stdout_is_empty(&self) -> bool {
        self.stdout.trim().is_empty()
    }

    /// Check if stderr is empty (ignoring whitespace)
    #[must_use]
    pub fn stderr_is_empty(&self) -> bool {
        self.stderr.trim().is_empty()
    }
}

impl ExecCommand {
    /// Create a new exec command for the specified container and command
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["ls".to_string(), "-la".to_string()]);
    /// ```
    pub fn new(container: impl Into<String>, command: Vec<String>) -> Self {
        Self {
            container: container.into(),
            command,
            executor: CommandExecutor::new(),
            detach: false,
            detach_keys: None,
            environment: EnvironmentBuilder::new(),
            env_files: Vec::new(),
            interactive: false,
            privileged: false,
            tty: false,
            user: None,
            workdir: None,
        }
    }

    /// Run in detached mode (background)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["sleep".to_string(), "10".to_string()])
    ///     .detach();
    /// ```
    #[must_use]
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }

    /// Override the key sequence for detaching a container
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["bash".to_string()])
    ///     .detach_keys("ctrl-p,ctrl-q");
    /// ```
    #[must_use]
    pub fn detach_keys(mut self, keys: impl Into<String>) -> Self {
        self.detach_keys = Some(keys.into());
        self
    }

    /// Add an environment variable
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["env".to_string()])
    ///     .env("DEBUG", "1")
    ///     .env("LOG_LEVEL", "info");
    /// ```
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment = self.environment.var(key, value);
        self
    }

    /// Add multiple environment variables
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    /// use std::collections::HashMap;
    ///
    /// let mut env_vars = HashMap::new();
    /// env_vars.insert("DEBUG".to_string(), "1".to_string());
    /// env_vars.insert("LOG_LEVEL".to_string(), "info".to_string());
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["env".to_string()])
    ///     .envs(env_vars);
    /// ```
    #[must_use]
    pub fn envs(mut self, vars: std::collections::HashMap<String, String>) -> Self {
        self.environment = self.environment.vars(vars);
        self
    }

    /// Add an environment file
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["env".to_string()])
    ///     .env_file("/path/to/env.file");
    /// ```
    #[must_use]
    pub fn env_file(mut self, file: impl Into<String>) -> Self {
        self.env_files.push(file.into());
        self
    }

    /// Keep STDIN open even if not attached
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["bash".to_string()])
    ///     .interactive();
    /// ```
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// Give extended privileges to the command
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["mount".to_string()])
    ///     .privileged();
    /// ```
    #[must_use]
    pub fn privileged(mut self) -> Self {
        self.privileged = true;
        self
    }

    /// Allocate a pseudo-TTY
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["bash".to_string()])
    ///     .tty();
    /// ```
    #[must_use]
    pub fn tty(mut self) -> Self {
        self.tty = true;
        self
    }

    /// Set username or UID (format: "<name|uid>[:<group|gid>]")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["whoami".to_string()])
    ///     .user("root");
    ///
    /// let exec_cmd2 = ExecCommand::new("my-container", vec!["id".to_string()])
    ///     .user("1000:1000");
    /// ```
    #[must_use]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set working directory inside the container
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["pwd".to_string()])
    ///     .workdir("/app");
    /// ```
    #[must_use]
    pub fn workdir(mut self, workdir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(workdir.into());
        self
    }

    /// Convenience method for interactive TTY mode
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::exec::ExecCommand;
    ///
    /// let exec_cmd = ExecCommand::new("my-container", vec!["bash".to_string()])
    ///     .it();
    /// ```
    #[must_use]
    pub fn it(self) -> Self {
        self.interactive().tty()
    }
}

#[async_trait]
impl DockerCommand for ExecCommand {
    type Output = ExecOutput;

    fn command_name(&self) -> &'static str {
        "exec"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add flags
        if self.detach {
            args.push("--detach".to_string());
        }

        if let Some(ref keys) = self.detach_keys {
            args.push("--detach-keys".to_string());
            args.push(keys.clone());
        }

        // Add environment variables
        for (key, value) in self.environment.as_map() {
            args.push("--env".to_string());
            args.push(format!("{key}={value}"));
        }

        // Add environment files
        for env_file in &self.env_files {
            args.push("--env-file".to_string());
            args.push(env_file.clone());
        }

        if self.interactive {
            args.push("--interactive".to_string());
        }

        if self.privileged {
            args.push("--privileged".to_string());
        }

        if self.tty {
            args.push("--tty".to_string());
        }

        if let Some(ref user) = self.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        if let Some(ref workdir) = self.workdir {
            args.push("--workdir".to_string());
            args.push(workdir.to_string_lossy().to_string());
        }

        // Add any additional raw arguments
        args.extend(self.executor.raw_args.clone());

        // Add container
        args.push(self.container.clone());

        // Add command
        args.extend(self.command.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self
            .executor
            .execute_command(self.command_name(), args)
            .await?;

        Ok(ExecOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_command_builder() {
        let cmd = ExecCommand::new("test-container", vec!["ls".to_string(), "-la".to_string()])
            .interactive()
            .tty()
            .env("DEBUG", "1")
            .user("root")
            .workdir("/app");

        let args = cmd.build_args();

        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--tty".to_string()));
        assert!(args.contains(&"--env".to_string()));
        assert!(args.contains(&"DEBUG=1".to_string()));
        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"root".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/app".to_string()));
        assert!(args.contains(&"test-container".to_string()));
        assert!(args.contains(&"ls".to_string()));
        assert!(args.contains(&"-la".to_string()));
    }

    #[test]
    fn test_exec_command_detach() {
        let cmd = ExecCommand::new(
            "test-container",
            vec!["sleep".to_string(), "10".to_string()],
        )
        .detach()
        .detach_keys("ctrl-p,ctrl-q");

        let args = cmd.build_args();

        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--detach-keys".to_string()));
        assert!(args.contains(&"ctrl-p,ctrl-q".to_string()));
    }

    #[test]
    fn test_exec_command_privileged() {
        let cmd = ExecCommand::new("test-container", vec!["mount".to_string()]).privileged();

        let args = cmd.build_args();

        assert!(args.contains(&"--privileged".to_string()));
    }

    #[test]
    fn test_exec_command_env_file() {
        let cmd = ExecCommand::new("test-container", vec!["env".to_string()])
            .env_file("/path/to/env.file")
            .env_file("/another/env.file");

        let args = cmd.build_args();

        assert!(args.contains(&"--env-file".to_string()));
        assert!(args.contains(&"/path/to/env.file".to_string()));
        assert!(args.contains(&"/another/env.file".to_string()));
    }

    #[test]
    fn test_it_convenience_method() {
        let cmd = ExecCommand::new("test-container", vec!["bash".to_string()]).it();

        let args = cmd.build_args();

        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--tty".to_string()));
    }

    #[test]
    fn test_exec_output_helpers() {
        let output_success = ExecOutput {
            stdout: "Hello World".to_string(),
            stderr: String::new(),
            exit_code: 0,
        };

        assert!(output_success.success());
        assert!(!output_success.stdout_is_empty());
        assert!(output_success.stderr_is_empty());
        assert_eq!(output_success.combined_output(), "Hello World");

        let output_error = ExecOutput {
            stdout: String::new(),
            stderr: "Error occurred".to_string(),
            exit_code: 1,
        };

        assert!(!output_error.success());
        assert!(output_error.stdout_is_empty());
        assert!(!output_error.stderr_is_empty());
        assert_eq!(output_error.combined_output(), "Error occurred");

        let output_combined = ExecOutput {
            stdout: "Output".to_string(),
            stderr: "Warning".to_string(),
            exit_code: 0,
        };

        assert_eq!(output_combined.combined_output(), "Output\nWarning");
    }

    #[test]
    fn test_exec_command_extensibility() {
        let mut cmd = ExecCommand::new("test-container", vec!["test".to_string()]);

        // Test extensibility methods
        cmd.flag("--some-flag");
        cmd.option("--some-option", "value");
        cmd.arg("extra-arg");

        let args = cmd.build_args();

        assert!(args.contains(&"--some-flag".to_string()));
        assert!(args.contains(&"--some-option".to_string()));
        assert!(args.contains(&"value".to_string()));
        assert!(args.contains(&"extra-arg".to_string()));
    }
}
