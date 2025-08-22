//! Streaming support for Docker command output.
//!
//! This module provides functionality to stream output from long-running Docker
//! commands in real-time, rather than waiting for completion.

use crate::error::Result;
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

/// Represents a line of output from a streaming command
#[derive(Debug, Clone)]
pub enum OutputLine {
    /// Standard output line
    Stdout(String),
    /// Standard error line  
    Stderr(String),
}

/// Result returned from streaming commands
#[derive(Debug, Clone)]
pub struct StreamResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Whether the command succeeded (exit code 0)
    pub success: bool,
    /// Accumulated stdout if captured
    pub stdout: Option<String>,
    /// Accumulated stderr if captured
    pub stderr: Option<String>,
}

impl StreamResult {
    /// Check if the command was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// Trait for commands that support streaming output
#[async_trait]
pub trait StreamableCommand: Send + Sync {
    /// Run the command with streaming output
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to spawn or encounters an I/O error
    async fn stream<F>(&self, handler: F) -> Result<StreamResult>
    where
        F: FnMut(OutputLine) + Send + 'static;

    /// Run the command with streaming output via a channel
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to spawn or encounters an I/O error
    async fn stream_channel(&self) -> Result<(mpsc::Receiver<OutputLine>, StreamResult)>;
}

/// Stream handler utilities
pub struct StreamHandler;

impl StreamHandler {
    /// Print output lines to stdout/stderr
    pub fn print() -> impl FnMut(OutputLine) {
        move |line| match line {
            OutputLine::Stdout(s) => println!("{s}"),
            OutputLine::Stderr(s) => eprintln!("{s}"),
        }
    }

    /// Collect output while also calling another handler
    pub fn tee<F>(mut handler: F) -> impl FnMut(OutputLine) -> (Vec<String>, Vec<String>)
    where
        F: FnMut(&OutputLine),
    {
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        move |line| {
            handler(&line);
            match line {
                OutputLine::Stdout(s) => stdout_lines.push(s),
                OutputLine::Stderr(s) => stderr_lines.push(s),
            }
            (stdout_lines.clone(), stderr_lines.clone())
        }
    }

    /// Filter lines by pattern
    pub fn filter(pattern: String) -> impl FnMut(OutputLine) -> Option<String> {
        move |line| {
            let text = match &line {
                OutputLine::Stdout(s) | OutputLine::Stderr(s) => s,
            };
            if text.contains(&pattern) {
                Some(text.clone())
            } else {
                None
            }
        }
    }

    /// Log output lines with a prefix
    pub fn with_prefix(prefix: String) -> impl FnMut(OutputLine) {
        move |line| match line {
            OutputLine::Stdout(s) => println!("{prefix}: {s}"),
            OutputLine::Stderr(s) => eprintln!("{prefix} (error): {s}"),
        }
    }
}

/// Internal helper to spawn a streaming command
pub(crate) async fn stream_command(
    mut cmd: TokioCommand,
    mut handler: impl FnMut(OutputLine) + Send + 'static,
) -> Result<StreamResult> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| crate::error::Error::custom(format!("Failed to spawn command: {e}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| crate::error::Error::custom("Failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| crate::error::Error::custom("Failed to capture stderr"))?;

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);
    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();

    let mut stdout_accumulator = Vec::new();
    let mut stderr_accumulator = Vec::new();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        stdout_accumulator.push(text.clone());
                        handler(OutputLine::Stdout(text));
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(crate::error::Error::custom(
                            format!("Error reading stdout: {e}")
                        ));
                    }
                }
            }
            line = stderr_lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        stderr_accumulator.push(text.clone());
                        handler(OutputLine::Stderr(text));
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(crate::error::Error::custom(
                            format!("Error reading stderr: {e}")
                        ));
                    }
                }
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| crate::error::Error::custom(format!("Failed to wait for command: {e}")))?;

    Ok(StreamResult {
        exit_code: status.code().unwrap_or(-1),
        success: status.success(),
        stdout: Some(stdout_accumulator.join("\n")),
        stderr: Some(stderr_accumulator.join("\n")),
    })
}

/// Internal helper to spawn a streaming command with channel output
pub(crate) async fn stream_command_channel(
    mut cmd: TokioCommand,
) -> Result<(mpsc::Receiver<OutputLine>, StreamResult)> {
    let (tx, rx) = mpsc::channel(100);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| crate::error::Error::custom(format!("Failed to spawn command: {e}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| crate::error::Error::custom("Failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| crate::error::Error::custom("Failed to capture stderr"))?;

    let tx_clone = tx.clone();

    // Spawn task to read stdout
    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut reader_lines = reader.lines();
        let mut lines = Vec::new();
        while let Ok(Some(line)) = reader_lines.next_line().await {
            lines.push(line.clone());
            let _ = tx.send(OutputLine::Stdout(line)).await;
        }
        lines
    });

    // Spawn task to read stderr
    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut reader_lines = reader.lines();
        let mut lines = Vec::new();
        while let Ok(Some(line)) = reader_lines.next_line().await {
            lines.push(line.clone());
            let _ = tx_clone.send(OutputLine::Stderr(line)).await;
        }
        lines
    });

    // Wait for both tasks and the process
    let status_future = child.wait();
    let (stdout_lines, stderr_lines, status) =
        tokio::join!(stdout_task, stderr_task, status_future);

    let stdout_lines = stdout_lines.unwrap_or_default();
    let stderr_lines = stderr_lines.unwrap_or_default();
    let status = status
        .map_err(|e| crate::error::Error::custom(format!("Failed to wait for command: {e}")))?;

    Ok((
        rx,
        StreamResult {
            exit_code: status.code().unwrap_or(-1),
            success: status.success(),
            stdout: Some(stdout_lines.join("\n")),
            stderr: Some(stderr_lines.join("\n")),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_line() {
        let stdout = OutputLine::Stdout("test".to_string());
        let stderr = OutputLine::Stderr("error".to_string());

        match stdout {
            OutputLine::Stdout(s) => assert_eq!(s, "test"),
            _ => panic!("Wrong variant"),
        }

        match stderr {
            OutputLine::Stderr(s) => assert_eq!(s, "error"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_stream_result() {
        let result = StreamResult {
            exit_code: 0,
            success: true,
            stdout: Some("output".to_string()),
            stderr: None,
        };

        assert!(result.is_success());
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, Some("output".to_string()));
        assert!(result.stderr.is_none());
    }

    #[test]
    fn test_stream_handler_filter() {
        let mut filter = StreamHandler::filter("error".to_string());

        let result1 = filter(OutputLine::Stdout(
            "this contains error message".to_string(),
        ));
        assert_eq!(result1, Some("this contains error message".to_string()));

        let result2 = filter(OutputLine::Stdout("normal message".to_string()));
        assert!(result2.is_none());
    }
}
