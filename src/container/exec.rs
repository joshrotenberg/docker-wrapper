//! Container execution module for running commands inside containers.
//!
//! This module provides functionality to execute commands inside running containers
//! using `docker exec`, with support for streaming I/O, environment variables,
//! and various execution modes.

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};

use crate::types::ContainerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Options for stream attachment during command execution
#[derive(Debug, Clone)]
pub struct AttachmentOptions {
    /// Attach to stdin
    pub stdin: bool,
    /// Attach to stdout
    pub stdout: bool,
    /// Attach to stderr
    pub stderr: bool,
}

impl Default for AttachmentOptions {
    fn default() -> Self {
        Self {
            stdin: false,
            stdout: true,
            stderr: true,
        }
    }
}

/// Execution mode options
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ExecutionOptions {
    /// Allocate a pseudo-TTY
    pub tty: bool,
    /// Run in privileged mode
    pub privileged: bool,
    /// Run in detached mode
    pub detached: bool,
    /// Run interactively
    pub interactive: bool,
}

/// Configuration for executing commands in containers
#[derive(Debug, Clone)]
pub struct ExecConfig {
    /// Command to execute
    pub command: Vec<String>,
    /// Working directory for the command
    pub working_dir: Option<PathBuf>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// User to run as
    pub user: Option<String>,
    /// Stream attachment options
    pub attachment: AttachmentOptions,
    /// Execution mode options
    pub execution: ExecutionOptions,
}

#[allow(clippy::derivable_impls)]
impl Default for ExecConfig {
    fn default() -> Self {
        Self {
            command: Vec::new(),
            working_dir: None,
            environment: HashMap::new(),
            user: None,
            attachment: AttachmentOptions::default(),
            execution: ExecutionOptions::default(),
        }
    }
}

impl ExecConfig {
    /// Create a new exec configuration with the specified command
    #[must_use]
    pub fn new(command: Vec<String>) -> Self {
        Self {
            command,
            ..Default::default()
        }
    }

    /// Create from a command string (space-separated)
    #[must_use]
    pub fn from_command_str(command: impl Into<String>) -> Self {
        let cmd_str = command.into();
        let command_parts: Vec<String> = cmd_str
            .split_whitespace()
            .map(std::string::ToString::to_string)
            .collect();
        Self::new(command_parts)
    }

    /// Set the working directory
    #[must_use]
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add an environment variable
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Set multiple environment variables
    #[must_use]
    pub fn envs(mut self, envs: HashMap<String, String>) -> Self {
        self.environment.extend(envs);
        self
    }

    /// Set the user to run as
    #[must_use]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Enable stdin attachment
    #[must_use]
    pub fn stdin(mut self) -> Self {
        self.attachment.stdin = true;
        self
    }

    /// Enable TTY allocation
    #[must_use]
    pub fn tty(mut self) -> Self {
        self.execution.tty = true;
        self
    }

    /// Run in privileged mode
    #[must_use]
    pub fn privileged(mut self) -> Self {
        self.execution.privileged = true;
        self
    }

    /// Run in detached mode
    #[must_use]
    pub fn detached(mut self) -> Self {
        self.execution.detached = true;
        self
    }

    /// Run interactively
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.execution.interactive = true;
        self.attachment.stdin = true;
        self
    }
}

/// Result of command execution in a container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Captured stdout
    pub stdout: String,
    /// Captured stderr
    pub stderr: String,
    /// Execution time
    pub duration: Duration,
}

impl ExecResult {
    /// Check if the command executed successfully (exit code 0)
    #[must_use]
    pub fn is_success(&self) -> bool {
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
            format!("{}\n{}", &self.stdout, &self.stderr)
        }
    }
}

/// Container executor for running commands in containers
pub struct ContainerExecutor<'a> {
    client: &'a DockerClient,
}

impl<'a> ContainerExecutor<'a> {
    /// Create a new container executor
    #[must_use]
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Execute a command in a container and wait for completion
    pub async fn exec(
        &self,
        container_id: &ContainerId,
        config: ExecConfig,
    ) -> DockerResult<ExecResult> {
        debug!(
            "Executing command in container {}: {:?}",
            container_id, config.command
        );

        let start_time = std::time::Instant::now();

        if config.execution.detached {
            // For detached execution, we just start the command and return immediately
            self.exec_detached(container_id, &config).await?;
            return Ok(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                duration: start_time.elapsed(),
            });
        }

        // Build the exec command
        let args = Self::build_exec_args(container_id, &config);

        // Execute the command
        let output = self.client.execute_command(&args, None).await?;

        let result = ExecResult {
            exit_code: if output.exit_code == 0 {
                0
            } else {
                output.exit_code
            },
            stdout: output.stdout,
            stderr: output.stderr,
            duration: start_time.elapsed(),
        };

        info!(
            "Command executed in container {} with exit code: {}",
            container_id, result.exit_code
        );

        Ok(result)
    }

    /// Execute a command in detached mode
    pub async fn exec_detached(
        &self,
        container_id: &ContainerId,
        config: &ExecConfig,
    ) -> DockerResult<String> {
        debug!(
            "Executing detached command in container {}: {:?}",
            container_id, config.command
        );

        let mut args = vec!["exec".to_string(), "--detach".to_string()];
        Self::add_exec_options(&mut args, config);
        args.push(container_id.to_string());
        args.extend(config.command.clone());

        let output = self.client.execute_command_stdout(&args).await?;
        let exec_id = output.trim().to_string();

        info!(
            "Started detached exec in container {}: {}",
            container_id, exec_id
        );
        Ok(exec_id)
    }

    /// Execute a command with streaming output
    ///
    /// # Panics
    ///
    /// Panics if the spawned process doesn't have stdout or stderr streams available.
    /// This should not happen under normal circumstances when executing Docker commands.
    pub async fn exec_streaming<F>(
        &self,
        container_id: &ContainerId,
        config: ExecConfig,
        mut output_handler: F,
    ) -> DockerResult<ExecResult>
    where
        F: FnMut(ExecOutput) -> DockerResult<()> + Send,
    {
        debug!(
            "Executing streaming command in container {}: {:?}",
            container_id, config.command
        );

        let start_time = std::time::Instant::now();
        let args = Self::build_exec_args(container_id, &config);

        // Create the command
        let mut cmd = Command::new(self.client.docker_path());
        cmd.args(&args); // Include all args - docker_path() is the binary, args start with subcommand

        // Configure stdio
        if config.attachment.stdin {
            cmd.stdin(std::process::Stdio::piped());
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| DockerError::process_spawn(format!("Failed to spawn docker exec: {e}")))?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Create channels for output
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn stdout reader
        let tx_stdout = tx.clone();
        let stdout_task = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if tx_stdout.send(ExecOutput::Stdout(line)).is_err() {
                    break;
                }
            }
        });

        // Spawn stderr reader
        let tx_stderr = tx.clone();
        let stderr_task = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if tx_stderr.send(ExecOutput::Stderr(line)).is_err() {
                    break;
                }
            }
        });

        // Drop the sender to close the channel when readers finish
        drop(tx);

        // Collect output and forward to handler
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        while let Some(output) = rx.recv().await {
            match &output {
                ExecOutput::Stdout(line) => stdout_lines.push(line.clone()),
                ExecOutput::Stderr(line) => stderr_lines.push(line.clone()),
            }

            // Forward to the handler
            if let Err(e) = output_handler(output) {
                warn!("Output handler error: {}", e);
            }
        }

        // Wait for the process to complete
        let exit_status = child.wait().await.map_err(|e| {
            DockerError::process_wait(format!("Failed to wait for docker exec: {e}"))
        })?;

        // Wait for readers to finish
        let _ = tokio::join!(stdout_task, stderr_task);

        let result = ExecResult {
            exit_code: exit_status.code().unwrap_or(-1),
            stdout: stdout_lines.join("\n"),
            stderr: stderr_lines.join("\n"),
            duration: start_time.elapsed(),
        };

        info!(
            "Streaming command executed in container {} with exit code: {}",
            container_id, result.exit_code
        );

        Ok(result)
    }

    /// Execute a simple command and return only stdout
    pub async fn exec_simple(
        &self,
        container_id: &ContainerId,
        command: Vec<String>,
    ) -> DockerResult<String> {
        let config = ExecConfig::new(command);
        let result = self.exec(container_id, config).await?;

        if result.is_success() {
            Ok(result.stdout)
        } else {
            Err(DockerError::generic(
                "exec_command".to_string(),
                format!(
                    "Command failed with exit code {}: {}",
                    result.exit_code, result.stderr
                ),
            ))
        }
    }

    /// Build the docker exec command arguments
    fn build_exec_args(container_id: &ContainerId, config: &ExecConfig) -> Vec<String> {
        let mut args = vec!["exec".to_string()];
        Self::add_exec_options(&mut args, config);
        args.push(container_id.to_string());
        args.extend(config.command.clone());
        args
    }

    /// Add exec options to the command arguments
    fn add_exec_options(args: &mut Vec<String>, config: &ExecConfig) {
        // Interactive and TTY options
        if config.execution.interactive {
            args.push("--interactive".to_string());
        }
        if config.execution.tty {
            args.push("--tty".to_string());
        }

        // User option
        if let Some(user) = &config.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        // Working directory
        if let Some(workdir) = &config.working_dir {
            args.push("--workdir".to_string());
            args.push(workdir.to_string_lossy().to_string());
        }

        // Environment variables
        for (key, value) in &config.environment {
            args.push("--env".to_string());
            args.push(format!("{key}={value}"));
        }

        // Privileged mode
        if config.execution.privileged {
            args.push("--privileged".to_string());
        }
    }
}

/// Output from exec command execution
#[derive(Debug, Clone)]
pub enum ExecOutput {
    /// Line from stdout
    Stdout(String),
    /// Line from stderr
    Stderr(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_config_builder() {
        let config = ExecConfig::from_command_str("echo hello world")
            .user("root")
            .working_dir("/tmp")
            .env("VAR", "value")
            .tty()
            .privileged();

        assert_eq!(config.command, vec!["echo", "hello", "world"]);
        assert_eq!(config.user, Some("root".to_string()));
        assert_eq!(config.working_dir, Some(PathBuf::from("/tmp")));
        assert_eq!(config.environment.get("VAR"), Some(&"value".to_string()));
        assert!(config.execution.tty);
        assert!(config.execution.privileged);
    }

    #[test]
    fn test_exec_result() {
        let result = ExecResult {
            exit_code: 0,
            stdout: "Hello".to_string(),
            stderr: "Warning".to_string(),
            duration: Duration::from_millis(100),
        };

        assert!(result.is_success());
        assert_eq!(result.combined_output(), "Hello\nWarning");
    }

    #[test]
    fn test_exec_result_failed() {
        let result = ExecResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: "Error occurred".to_string(),
            duration: Duration::from_millis(50),
        };

        assert!(!result.is_success());
        assert_eq!(result.combined_output(), "Error occurred");
    }
}
