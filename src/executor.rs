//! Async process executor for Docker command execution.
//!
//! This module provides the core process execution functionality for running
//! Docker commands asynchronously with proper timeout handling, streaming I/O,
//! and error management.

use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

use crate::errors::{DockerError, DockerResult};

/// Configuration for command execution
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Command timeout (None for no timeout)
    pub timeout: Option<Duration>,
    /// Environment variables to set
    pub environment: HashMap<String, String>,
    /// Working directory for the command
    pub working_dir: Option<std::path::PathBuf>,
    /// Whether to capture stdout
    pub capture_stdout: bool,
    /// Whether to capture stderr
    pub capture_stderr: bool,
    /// Input data to send to stdin
    pub stdin_data: Option<Vec<u8>>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            environment: HashMap::new(),
            working_dir: None,
            capture_stdout: true,
            capture_stderr: true,
            stdin_data: None,
        }
    }
}

/// Result of command execution
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Exit code of the process
    pub exit_code: i32,
    /// Captured stdout (if enabled)
    pub stdout: String,
    /// Captured stderr (if enabled)
    pub stderr: String,
    /// Whether the command was successful (exit code 0)
    pub success: bool,
}

impl CommandOutput {
    /// Create a new command output
    pub fn new(exit_code: i32, stdout: String, stderr: String) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
            success: exit_code == 0,
        }
    }

    /// Get the combined output (stdout + stderr)
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }
}

/// Streaming command output
pub struct StreamingOutput {
    /// Receiver for stdout lines
    pub stdout: mpsc::Receiver<DockerResult<String>>,
    /// Receiver for stderr lines
    pub stderr: mpsc::Receiver<DockerResult<String>>,
    /// Handle to the child process
    pub child: Child,
}

/// Async process executor for Docker commands
#[derive(Debug, Clone)]
pub struct ProcessExecutor {
    /// Path to the Docker binary
    pub docker_path: std::path::PathBuf,
    /// Default execution configuration
    default_config: ExecutionConfig,
}

impl ProcessExecutor {
    /// Create a new process executor with the given Docker binary path
    pub fn new(docker_path: std::path::PathBuf) -> Self {
        Self {
            docker_path,
            default_config: ExecutionConfig::default(),
        }
    }

    /// Set the default timeout for all commands
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_config.timeout = Some(timeout);
        self
    }

    /// Execute a Docker command and return the output
    pub async fn execute(
        &self,
        args: &[String],
        config: Option<ExecutionConfig>,
    ) -> DockerResult<CommandOutput> {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        let command_str = format!("docker {}", args.join(" "));

        debug!("Executing command: {}", command_str);

        let mut command = Command::new(&self.docker_path);
        command.args(args);

        // Set up stdio
        command.stdin(if config.stdin_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        command.stdout(if config.capture_stdout {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        command.stderr(if config.capture_stderr {
            Stdio::piped()
        } else {
            Stdio::null()
        });

        // Set environment variables
        for (key, value) in &config.environment {
            command.env(key, value);
        }

        // Set working directory
        if let Some(working_dir) = &config.working_dir {
            command.current_dir(working_dir);
        }

        // Spawn the process
        let mut child = command
            .spawn()
            .map_err(|e| DockerError::io(format!("Failed to spawn command: {}", command_str), e))?;

        // Write stdin data if provided
        if let Some(stdin_data) = config.stdin_data {
            if let Some(mut stdin) = child.stdin.take() {
                tokio::spawn(async move {
                    if let Err(e) = stdin.write_all(&stdin_data).await {
                        warn!("Failed to write to stdin: {}", e);
                    }
                    if let Err(e) = stdin.shutdown().await {
                        warn!("Failed to close stdin: {}", e);
                    }
                });
            }
        }

        // Wait for the process to complete with optional timeout
        let output = if let Some(timeout_duration) = config.timeout {
            match timeout(timeout_duration, self.wait_for_output(child)).await {
                Ok(result) => result?,
                Err(_) => {
                    return Err(DockerError::command_timeout(command_str, timeout_duration));
                }
            }
        } else {
            self.wait_for_output(child).await?
        };

        trace!("Command completed with exit code: {}", output.exit_code);

        if !output.success {
            return Err(DockerError::command_failed(
                command_str,
                output.exit_code,
                output.stdout.clone(),
                output.stderr.clone(),
            ));
        }

        Ok(output)
    }

    /// Execute a Docker command with streaming output
    pub async fn execute_streaming(
        &self,
        args: &[String],
        config: Option<ExecutionConfig>,
    ) -> DockerResult<StreamingOutput> {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        let command_str = format!("docker {}", args.join(" "));

        debug!("Executing streaming command: {}", command_str);

        let mut command = Command::new(&self.docker_path);
        command.args(args);

        // Always use pipes for streaming
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &config.environment {
            command.env(key, value);
        }

        // Set working directory
        if let Some(working_dir) = &config.working_dir {
            command.current_dir(working_dir);
        }

        // Spawn the process
        let mut child = command.spawn().map_err(|e| {
            DockerError::io(
                format!("Failed to spawn streaming command: {}", command_str),
                e,
            )
        })?;

        // Set up stdout streaming
        let stdout = child.stdout.take().unwrap();
        let (stdout_tx, stdout_rx) = mpsc::channel(100);
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stdout_tx.send(Ok(line)).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        // Set up stderr streaming
        let stderr = child.stderr.take().unwrap();
        let (stderr_tx, stderr_rx) = mpsc::channel(100);
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stderr_tx.send(Ok(line)).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        // Write stdin data if provided
        if let Some(stdin_data) = config.stdin_data {
            if let Some(mut stdin) = child.stdin.take() {
                tokio::spawn(async move {
                    if let Err(e) = stdin.write_all(&stdin_data).await {
                        warn!("Failed to write to stdin: {}", e);
                    }
                    if let Err(e) = stdin.shutdown().await {
                        warn!("Failed to close stdin: {}", e);
                    }
                });
            }
        }

        Ok(StreamingOutput {
            stdout: stdout_rx,
            stderr: stderr_rx,
            child,
        })
    }

    /// Wait for process output and collect it
    async fn wait_for_output(&self, mut child: Child) -> DockerResult<CommandOutput> {
        let stdout_handle = if let Some(stdout) = child.stdout.take() {
            let handle = tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                let mut output = String::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !output.is_empty() {
                        output.push('\n');
                    }
                    output.push_str(&line);
                }
                output
            });
            Some(handle)
        } else {
            None
        };

        let stderr_handle = if let Some(stderr) = child.stderr.take() {
            let handle = tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                let mut output = String::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !output.is_empty() {
                        output.push('\n');
                    }
                    output.push_str(&line);
                }
                output
            });
            Some(handle)
        } else {
            None
        };

        // Wait for the process to complete
        let status = child
            .wait()
            .await
            .map_err(|e| DockerError::io("Failed to wait for process completion".to_string(), e))?;

        // Collect output
        let stdout = if let Some(handle) = stdout_handle {
            handle.await.map_err(|e| {
                DockerError::generic("stdout collection", format!("Join error: {}", e))
            })?
        } else {
            String::new()
        };

        let stderr = if let Some(handle) = stderr_handle {
            handle.await.map_err(|e| {
                DockerError::generic("stderr collection", format!("Join error: {}", e))
            })?
        } else {
            String::new()
        };

        let exit_code = status.code().unwrap_or(-1);

        Ok(CommandOutput::new(exit_code, stdout, stderr))
    }

    /// Check if the Docker binary is available and executable
    pub async fn check_docker_available(&self) -> DockerResult<()> {
        debug!("Checking Docker availability at: {:?}", self.docker_path);

        let output = self
            .execute(&["--version".to_string()], None)
            .await
            .map_err(|_| DockerError::docker_not_found(self.docker_path.display().to_string()))?;

        if !output.success {
            return Err(DockerError::docker_not_found(format!(
                "{} (not executable)",
                self.docker_path.display()
            )));
        }

        debug!("Docker version check successful: {}", output.stdout.trim());
        Ok(())
    }

    /// Get the Docker version
    pub async fn get_docker_version(&self) -> DockerResult<String> {
        let output = self.execute(&["--version".to_string()], None).await?;

        // Parse version from output like "Docker version 20.10.21, build baeda1f"
        let version_line = output.stdout.trim();
        if let Some(version_part) = version_line.split(',').next() {
            if let Some(version) = version_part.strip_prefix("Docker version ") {
                return Ok(version.to_string());
            }
        }

        // Fallback: return the full line
        Ok(version_line.to_string())
    }

    /// Get Docker system information
    pub async fn get_docker_info(&self) -> DockerResult<String> {
        let output = self.execute(&["info".to_string()], None).await?;
        Ok(output.stdout)
    }
}

/// Find the Docker binary in the system PATH
pub fn find_docker_binary() -> DockerResult<std::path::PathBuf> {
    // Try common locations
    let possible_paths = [
        "docker",
        "/usr/bin/docker",
        "/usr/local/bin/docker",
        "/opt/docker/bin/docker",
    ];

    for path in &possible_paths {
        let path_buf = std::path::PathBuf::from(path);
        if path_buf.exists() || which::which(path).is_ok() {
            return Ok(path_buf);
        }
    }

    // Use which crate to find in PATH
    which::which("docker")
        .map_err(|_| DockerError::docker_not_found("docker not found in PATH".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_docker_binary() {
        // This test might fail in environments without Docker
        match find_docker_binary() {
            Ok(path) => {
                assert!(path.file_name().unwrap() == "docker");
            }
            Err(e) => {
                println!("Docker not found (expected in some environments): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_command_output() {
        let output = CommandOutput::new(0, "stdout".to_string(), "stderr".to_string());
        assert!(output.success);
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.combined_output(), "stdout\nstderr");

        let output_no_stderr = CommandOutput::new(0, "stdout".to_string(), "".to_string());
        assert_eq!(output_no_stderr.combined_output(), "stdout");
    }

    #[tokio::test]
    async fn test_execution_config() {
        let config = ExecutionConfig::default();
        assert!(config.timeout.is_some());
        assert!(config.capture_stdout);
        assert!(config.capture_stderr);
        assert!(config.stdin_data.is_none());
    }

    #[tokio::test]
    async fn test_process_executor_creation() {
        let executor = ProcessExecutor::new("/usr/bin/docker".into())
            .with_default_timeout(Duration::from_secs(60));

        assert_eq!(
            executor.docker_path,
            std::path::PathBuf::from("/usr/bin/docker")
        );
        assert_eq!(
            executor.default_config.timeout,
            Some(Duration::from_secs(60))
        );
    }

    // Integration tests (require Docker to be installed)
    #[tokio::test]
    #[ignore = "Requires Docker daemon running"]
    async fn test_docker_version_integration() {
        let docker_path = match find_docker_binary() {
            Ok(path) => path,
            Err(_) => return, // Skip if Docker not available
        };

        let executor = ProcessExecutor::new(docker_path);

        match executor.check_docker_available().await {
            Ok(()) => {
                let version = executor.get_docker_version().await.unwrap();
                assert!(!version.is_empty());
                println!("Docker version: {}", version);
            }
            Err(e) => {
                println!("Docker not available: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires Docker daemon running"]
    async fn test_docker_info_integration() {
        let docker_path = match find_docker_binary() {
            Ok(path) => path,
            Err(_) => return, // Skip if Docker not available
        };

        let executor = ProcessExecutor::new(docker_path);

        match executor.get_docker_info().await {
            Ok(info) => {
                assert!(!info.is_empty());
                assert!(info.contains("Server Version") || info.contains("ERROR"));
                // ERROR if daemon not running
            }
            Err(_) => {
                println!("Docker info failed (daemon might not be running)");
            }
        }
    }
}
