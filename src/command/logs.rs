//! Docker logs command implementation.
//!
//! This module provides the `docker logs` command for viewing container logs.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use crate::stream::{OutputLine, StreamResult, StreamableCommand};
use async_trait::async_trait;
use std::ffi::OsStr;
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

/// Docker logs command builder
#[derive(Debug, Clone)]
pub struct LogsCommand {
    /// Container name or ID
    container: String,
    /// Follow log output
    follow: bool,
    /// Show timestamps
    timestamps: bool,
    /// Number of lines to show from the end
    tail: Option<String>,
    /// Show logs since timestamp
    since: Option<String>,
    /// Show logs until timestamp
    until: Option<String>,
    /// Show extra details
    details: bool,
    /// Command executor
    executor: CommandExecutor,
}

impl LogsCommand {
    /// Create a new logs command
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            follow: false,
            timestamps: false,
            tail: None,
            since: None,
            until: None,
            details: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Follow log output (like tail -f)
    #[must_use]
    pub fn follow(mut self) -> Self {
        self.follow = true;
        self
    }

    /// Show timestamps
    #[must_use]
    pub fn timestamps(mut self) -> Self {
        self.timestamps = true;
        self
    }

    /// Number of lines to show from the end of the logs
    #[must_use]
    pub fn tail(mut self, lines: impl Into<String>) -> Self {
        self.tail = Some(lines.into());
        self
    }

    /// Show all logs (equivalent to tail("all"))
    #[must_use]
    pub fn all(mut self) -> Self {
        self.tail = Some("all".to_string());
        self
    }

    /// Show logs since timestamp (e.g., 2013-01-02T13:23:37Z) or relative (e.g., 42m)
    #[must_use]
    pub fn since(mut self, timestamp: impl Into<String>) -> Self {
        self.since = Some(timestamp.into());
        self
    }

    /// Show logs until timestamp
    #[must_use]
    pub fn until(mut self, timestamp: impl Into<String>) -> Self {
        self.until = Some(timestamp.into());
        self
    }

    /// Show extra details provided to logs
    #[must_use]
    pub fn details(mut self) -> Self {
        self.details = true;
        self
    }

    /// Execute the logs command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The specified container doesn't exist
    /// - The container has been removed
    pub async fn run(&self) -> Result<CommandOutput> {
        self.execute().await
    }
}

#[async_trait]
impl DockerCommand for LogsCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "logs"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.follow {
            args.push("--follow".to_string());
        }

        if self.timestamps {
            args.push("--timestamps".to_string());
        }

        if let Some(ref tail) = self.tail {
            args.push("--tail".to_string());
            args.push(tail.clone());
        }

        if let Some(ref since) = self.since {
            args.push("--since".to_string());
            args.push(since.clone());
        }

        if let Some(ref until) = self.until {
            args.push("--until".to_string());
            args.push(until.clone());
        }

        if self.details {
            args.push("--details".to_string());
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

// Streaming support for LogsCommand
#[async_trait]
impl StreamableCommand for LogsCommand {
    async fn stream<F>(&self, handler: F) -> Result<StreamResult>
    where
        F: FnMut(OutputLine) + Send + 'static,
    {
        let mut cmd = TokioCommand::new("docker");
        cmd.arg(self.command_name());

        for arg in self.build_args() {
            cmd.arg(arg);
        }

        crate::stream::stream_command(cmd, handler).await
    }

    async fn stream_channel(&self) -> Result<(mpsc::Receiver<OutputLine>, StreamResult)> {
        let mut cmd = TokioCommand::new("docker");
        cmd.arg(self.command_name());

        for arg in self.build_args() {
            cmd.arg(arg);
        }

        crate::stream::stream_command_channel(cmd).await
    }
}

impl LogsCommand {
    /// Stream container logs with a custom handler
    ///
    /// This is particularly useful with the `follow` option to stream logs in real-time.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use docker_wrapper::LogsCommand;
    /// use docker_wrapper::StreamHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Follow logs in real-time
    /// let result = LogsCommand::new("mycontainer")
    ///     .follow()
    ///     .timestamps()
    ///     .stream(StreamHandler::print())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails or encounters an I/O error
    pub async fn stream<F>(&self, handler: F) -> Result<StreamResult>
    where
        F: FnMut(OutputLine) + Send + 'static,
    {
        <Self as StreamableCommand>::stream(self, handler).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_basic() {
        let cmd = LogsCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_logs_follow() {
        let cmd = LogsCommand::new("test-container").follow();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--follow", "test-container"]);
    }

    #[test]
    fn test_logs_with_tail() {
        let cmd = LogsCommand::new("test-container").tail("100");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--tail", "100", "test-container"]);
    }

    #[test]
    fn test_logs_with_timestamps() {
        let cmd = LogsCommand::new("test-container").timestamps();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--timestamps", "test-container"]);
    }

    #[test]
    fn test_logs_with_since() {
        let cmd = LogsCommand::new("test-container").since("10m");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--since", "10m", "test-container"]);
    }

    #[test]
    fn test_logs_all_options() {
        let cmd = LogsCommand::new("test-container")
            .follow()
            .timestamps()
            .tail("50")
            .since("1h")
            .details();
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "--follow",
                "--timestamps",
                "--tail",
                "50",
                "--since",
                "1h",
                "--details",
                "test-container"
            ]
        );
    }
}
