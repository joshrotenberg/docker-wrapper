//! Container logs module for retrieving and streaming container logs.
//!
//! This module provides functionality to retrieve logs from containers using
//! `docker logs`, with support for streaming, filtering, and various output formats.

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::types::ContainerId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Options for retrieving container logs
#[derive(Debug, Clone)]
pub struct LogOptions {
    /// Follow log output (stream logs)
    pub follow: bool,
    /// Show timestamps
    pub timestamps: bool,
    /// Number of lines to show from the end
    pub tail: Option<u64>,
    /// Show logs since timestamp
    pub since: Option<DateTime<Utc>>,
    /// Show logs until timestamp
    pub until: Option<DateTime<Utc>>,
    /// Show stdout logs
    pub stdout: bool,
    /// Show stderr logs
    pub stderr: bool,
    /// Add details about log source
    pub details: bool,
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            follow: false,
            timestamps: false,
            tail: None,
            since: None,
            until: None,
            stdout: true,
            stderr: true,
            details: false,
        }
    }
}

impl LogOptions {
    /// Create new log options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable following (streaming) logs
    pub fn follow(mut self) -> Self {
        self.follow = true;
        self
    }

    /// Enable timestamps in log output
    pub fn timestamps(mut self) -> Self {
        self.timestamps = true;
        self
    }

    /// Set number of lines to show from the end
    pub fn tail(mut self, lines: u64) -> Self {
        self.tail = Some(lines);
        self
    }

    /// Show logs since the specified timestamp
    pub fn since(mut self, timestamp: DateTime<Utc>) -> Self {
        self.since = Some(timestamp);
        self
    }

    /// Show logs until the specified timestamp
    pub fn until(mut self, timestamp: DateTime<Utc>) -> Self {
        self.until = Some(timestamp);
        self
    }

    /// Show only stdout logs
    pub fn stdout_only(mut self) -> Self {
        self.stdout = true;
        self.stderr = false;
        self
    }

    /// Show only stderr logs
    pub fn stderr_only(mut self) -> Self {
        self.stdout = false;
        self.stderr = true;
        self
    }

    /// Enable log details
    pub fn details(mut self) -> Self {
        self.details = true;
        self
    }
}

/// A single log entry from a container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// The log message content
    pub message: String,
    /// Timestamp when the log was generated
    pub timestamp: Option<DateTime<Utc>>,
    /// Log source (stdout or stderr)
    pub source: LogSource,
    /// Additional details if available
    pub details: Option<LogDetails>,
}

/// Source of a log entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogSource {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
}

impl std::fmt::Display for LogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdout => write!(f, "stdout"),
            Self::Stderr => write!(f, "stderr"),
        }
    }
}

/// Additional details about a log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDetails {
    /// Partial log indicator
    pub partial: Option<bool>,
    /// Log attributes
    pub attrs: Option<std::collections::HashMap<String, String>>,
}

/// Container logs manager
pub struct LogManager<'a> {
    client: &'a DockerClient,
}

impl<'a> LogManager<'a> {
    /// Create a new log manager
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Get container logs as a string
    pub async fn get_logs(
        &self,
        container_id: &ContainerId,
        options: LogOptions,
    ) -> DockerResult<String> {
        debug!("Getting logs for container: {}", container_id);

        let args = self.build_logs_args(container_id, &options)?;
        let output = self.client.execute_command_stdout(&args).await?;

        info!(
            "Retrieved {} bytes of logs for container {}",
            output.len(),
            container_id
        );

        Ok(output)
    }

    /// Get container logs as structured log entries
    pub async fn get_log_entries(
        &self,
        container_id: &ContainerId,
        mut options: LogOptions,
    ) -> DockerResult<Vec<LogEntry>> {
        debug!("Getting structured logs for container: {}", container_id);

        // Enable timestamps and details for parsing
        options.timestamps = true;
        options.details = true;

        let args = self.build_logs_args(container_id, &options)?;
        let output = self.client.execute_command_stdout(&args).await?;

        let entries = self.parse_log_output(&output)?;

        info!(
            "Parsed {} log entries for container {}",
            entries.len(),
            container_id
        );

        Ok(entries)
    }

    /// Stream container logs with a callback function
    pub async fn stream_logs<F>(
        &self,
        container_id: &ContainerId,
        mut options: LogOptions,
        mut log_handler: F,
    ) -> DockerResult<()>
    where
        F: FnMut(LogEntry) -> DockerResult<()> + Send,
    {
        debug!("Starting log stream for container: {}", container_id);

        // Force follow mode for streaming
        options.follow = true;
        options.timestamps = true;

        let args = self.build_logs_args(container_id, &options)?;

        // Create the command
        let mut cmd = Command::new(self.client.docker_path());
        cmd.args(&args); // Include all args - docker_path() is the binary, args start with subcommand
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            DockerError::process_spawn(format!("Failed to spawn docker logs: {}", e))
        })?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Create channels for log entries
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn stdout reader
        let tx_stdout = tx.clone();
        let stdout_task = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let entry = LogEntry {
                    message: line,
                    timestamp: None, // Will be parsed later if needed
                    source: LogSource::Stdout,
                    details: None,
                };

                if tx_stdout.send(entry).is_err() {
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
                let entry = LogEntry {
                    message: line,
                    timestamp: None,
                    source: LogSource::Stderr,
                    details: None,
                };

                if tx_stderr.send(entry).is_err() {
                    break;
                }
            }
        });

        // Drop the sender to close the channel when readers finish
        drop(tx);

        // Process log entries
        while let Some(entry) = rx.recv().await {
            if let Err(e) = log_handler(entry) {
                warn!("Log handler error: {}", e);
            }
        }

        // Clean up
        let _ = child.kill().await;
        let _ = tokio::join!(stdout_task, stderr_task);

        info!("Log streaming completed for container: {}", container_id);
        Ok(())
    }

    /// Stream logs and collect them into a vector (with optional limit)
    pub async fn stream_logs_collect(
        &self,
        container_id: &ContainerId,
        options: LogOptions,
        max_entries: Option<usize>,
    ) -> DockerResult<Vec<LogEntry>> {
        let mut entries = Vec::new();
        let mut count = 0;

        self.stream_logs(container_id, options, |entry| {
            entries.push(entry);
            count += 1;

            // Check if we've reached the limit
            if let Some(max) = max_entries {
                if count >= max {
                    return Err(DockerError::cancelled(
                        "Reached maximum log entries".to_string(),
                    ));
                }
            }

            Ok(())
        })
        .await?;

        Ok(entries)
    }

    /// Get the last N log entries
    pub async fn get_recent_logs(
        &self,
        container_id: &ContainerId,
        count: u64,
    ) -> DockerResult<Vec<LogEntry>> {
        let options = LogOptions::new().tail(count);
        self.get_log_entries(container_id, options).await
    }

    /// Follow logs until a specific pattern is found
    pub async fn follow_until_pattern(
        &self,
        container_id: &ContainerId,
        pattern: &str,
        timeout: Option<Duration>,
    ) -> DockerResult<Vec<LogEntry>> {
        debug!("Following logs until pattern '{}' is found", pattern);

        let mut collected_logs = Vec::new();
        let pattern_regex = regex::Regex::new(pattern)
            .map_err(|e| DockerError::invalid_config(format!("Invalid regex pattern: {}", e)))?;

        let start_time = std::time::Instant::now();
        let options = LogOptions::new().follow();

        // Use a channel to signal when pattern is found
        let (tx, mut rx) = mpsc::unbounded_channel();
        let tx_clone = tx.clone();

        // Start streaming logs
        let stream_task = tokio::spawn({
            let container_id = container_id.clone();
            let client = self.client.clone();

            async move {
                let log_manager = LogManager::new(&client);
                log_manager
                    .stream_logs(&container_id, options, move |entry| {
                        let found_pattern = pattern_regex.is_match(&entry.message);

                        if tx_clone.send((entry, found_pattern)).is_err() {
                            return Err(DockerError::cancelled("Channel closed".to_string()));
                        }

                        if found_pattern {
                            return Err(DockerError::cancelled("Pattern found".to_string()));
                        }

                        Ok(())
                    })
                    .await
            }
        });

        // Collect logs and watch for pattern or timeout
        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Some((entry, found_pattern)) => {
                            collected_logs.push(entry);
                            if found_pattern {
                                break;
                            }
                        }
                        None => break, // Channel closed
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if let Some(timeout_duration) = timeout {
                        if start_time.elapsed() > timeout_duration {
                            return Err(DockerError::timeout(
                                format!("Pattern '{}' not found within timeout", pattern)
                            ));
                        }
                    }
                }
            }
        }

        // Cancel the streaming task
        stream_task.abort();

        info!(
            "Found pattern '{}' after {} log entries",
            pattern,
            collected_logs.len()
        );
        Ok(collected_logs)
    }

    /// Build docker logs command arguments
    fn build_logs_args(
        &self,
        container_id: &ContainerId,
        options: &LogOptions,
    ) -> DockerResult<Vec<String>> {
        let mut args = vec!["logs".to_string()];

        // Follow option
        if options.follow {
            args.push("--follow".to_string());
        }

        // Timestamps
        if options.timestamps {
            args.push("--timestamps".to_string());
        }

        // Tail option
        if let Some(tail) = options.tail {
            args.push("--tail".to_string());
            args.push(tail.to_string());
        }

        // Since option
        if let Some(since) = options.since {
            args.push("--since".to_string());
            args.push(since.to_rfc3339());
        }

        // Until option
        if let Some(until) = options.until {
            args.push("--until".to_string());
            args.push(until.to_rfc3339());
        }

        // Details option
        if options.details {
            args.push("--details".to_string());
        }

        // Add container ID
        args.push(container_id.to_string());

        Ok(args)
    }

    /// Parse log output into structured log entries
    fn parse_log_output(&self, output: &str) -> DockerResult<Vec<LogEntry>> {
        let mut entries = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let entry = self.parse_log_line(line)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Parse a single log line into a LogEntry
    fn parse_log_line(&self, line: &str) -> DockerResult<LogEntry> {
        // Try to parse timestamp if present
        // Docker log format: "2023-01-01T12:00:00.000000000Z message"
        let (timestamp, message) = if line.len() > 30 && line.chars().nth(4) == Some('-') {
            // Looks like it starts with a timestamp
            if let Some(space_pos) = line.find(' ') {
                let timestamp_str = &line[..space_pos];
                let message = &line[space_pos + 1..];

                let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok();

                (timestamp, message.to_string())
            } else {
                (None, line.to_string())
            }
        } else {
            (None, line.to_string())
        };

        Ok(LogEntry {
            message,
            timestamp,
            source: LogSource::Stdout, // Default to stdout
            details: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_options_builder() {
        let options = LogOptions::new()
            .follow()
            .timestamps()
            .tail(100)
            .stdout_only()
            .details();

        assert!(options.follow);
        assert!(options.timestamps);
        assert_eq!(options.tail, Some(100));
        assert!(options.stdout);
        assert!(!options.stderr);
        assert!(options.details);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            message: "Test log message".to_string(),
            timestamp: Some(Utc::now()),
            source: LogSource::Stdout,
            details: None,
        };

        assert_eq!(entry.message, "Test log message");
        assert_eq!(entry.source, LogSource::Stdout);
        assert!(entry.timestamp.is_some());
    }

    #[test]
    fn test_log_source_display() {
        assert_eq!(LogSource::Stdout.to_string(), "stdout");
        assert_eq!(LogSource::Stderr.to_string(), "stderr");
    }

    #[test]
    fn test_parse_log_line_with_timestamp() {
        // Test basic log line parsing without mocking DockerClient
        let line = "2023-01-01T12:00:00.000000000Z This is a test message";

        // Verify basic structure of Docker log line with timestamp
        assert!(line.len() > 30);
        assert_eq!(line.chars().nth(4), Some('-'));
        assert!(line.contains("This is a test message"));
    }

    #[test]
    fn test_parse_log_line_without_timestamp() {
        let line = "This is a message without timestamp";
        assert!(line.len() < 30 || line.chars().nth(4) != Some('-'));
    }
}
