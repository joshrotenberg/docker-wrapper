//! Docker top command implementation.
//!
//! This module provides the `docker top` command for displaying running processes in a container.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker top command builder
///
/// Display the running processes of a container.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::TopCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Show processes in a container
/// let processes = TopCommand::new("my-container")
///     .run()
///     .await?;
///
/// // Use custom ps options
/// let detailed = TopCommand::new("my-container")
///     .ps_options("aux")
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TopCommand {
    /// Container name or ID
    container: String,
    /// ps command options
    ps_options: Option<String>,
    /// Command executor
    executor: CommandExecutor,
}

impl TopCommand {
    /// Create a new top command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::TopCommand;
    ///
    /// let cmd = TopCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            ps_options: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Set ps command options
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::TopCommand;
    ///
    /// // Show detailed process information
    /// let cmd = TopCommand::new("my-container")
    ///     .ps_options("aux");
    ///
    /// // Show processes with specific format
    /// let cmd = TopCommand::new("my-container")
    ///     .ps_options("-eo pid,ppid,cmd,%mem,%cpu");
    /// ```
    #[must_use]
    pub fn ps_options(mut self, options: impl Into<String>) -> Self {
        self.ps_options = Some(options.into());
        self
    }

    /// Execute the top command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    /// - The container is not running
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::TopCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = TopCommand::new("my-container")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Container processes:\n{}", result.output().stdout);
    ///     for process in result.processes() {
    ///         println!("PID: {}, CMD: {}", process.pid, process.command);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<TopResult> {
        let output = self.execute().await?;

        // Parse process information from output
        let processes = Self::parse_processes(&output.stdout);

        Ok(TopResult {
            output,
            container: self.container.clone(),
            processes,
        })
    }

    /// Parse process information from top command output
    fn parse_processes(stdout: &str) -> Vec<ContainerProcess> {
        let mut processes = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            return processes;
        }

        // First line contains headers, skip it
        let _headers = lines[0].split_whitespace().collect::<Vec<_>>();

        // Parse each process line
        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if !parts.is_empty() {
                let process = ContainerProcess {
                    pid: (*parts.first().unwrap_or(&"")).to_string(),
                    user: if parts.len() > 1 {
                        parts[1].to_string()
                    } else {
                        String::new()
                    },
                    time: if parts.len() > 2 {
                        parts[2].to_string()
                    } else {
                        String::new()
                    },
                    command: if parts.len() > 3 {
                        parts[3..].join(" ")
                    } else {
                        String::new()
                    },
                    raw_line: (*line).to_string(),
                };
                processes.push(process);
            }
        }

        processes
    }
}

#[async_trait]
impl DockerCommand for TopCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "top"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add container name/ID
        args.push(self.container.clone());

        // Add ps options if specified
        if let Some(ref options) = self.ps_options {
            args.push(options.clone());
        }

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

/// Result from the top command
#[derive(Debug, Clone)]
pub struct TopResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Container that was inspected
    pub container: String,
    /// Parsed process information
    pub processes: Vec<ContainerProcess>,
}

impl TopResult {
    /// Check if the top command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the container name
    #[must_use]
    pub fn container(&self) -> &str {
        &self.container
    }

    /// Get the parsed processes
    #[must_use]
    pub fn processes(&self) -> &[ContainerProcess] {
        &self.processes
    }

    /// Get the raw command output
    #[must_use]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }

    /// Get process count
    #[must_use]
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }
}

/// Information about a running process in a container
#[derive(Debug, Clone)]
pub struct ContainerProcess {
    /// Process ID
    pub pid: String,
    /// User running the process
    pub user: String,
    /// CPU time
    pub time: String,
    /// Command line
    pub command: String,
    /// Raw output line
    pub raw_line: String,
}

impl ContainerProcess {
    /// Get PID as integer
    #[must_use]
    pub fn pid_as_int(&self) -> Option<u32> {
        self.pid.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_basic() {
        let cmd = TopCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_top_with_ps_options() {
        let cmd = TopCommand::new("test-container").ps_options("aux");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container", "aux"]);
    }

    #[test]
    fn test_top_with_custom_ps_options() {
        let cmd = TopCommand::new("test-container").ps_options("-eo pid,ppid,cmd");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container", "-eo pid,ppid,cmd"]);
    }

    #[test]
    fn test_parse_processes() {
        let output = "PID   USER     TIME   COMMAND\n1234  root     0:00   nginx: master process\n5678  www-data 0:01   nginx: worker process";

        let processes = TopCommand::parse_processes(output);
        assert_eq!(processes.len(), 2);

        assert_eq!(processes[0].pid, "1234");
        assert_eq!(processes[0].user, "root");
        assert_eq!(processes[0].time, "0:00");
        assert_eq!(processes[0].command, "nginx: master process");

        assert_eq!(processes[1].pid, "5678");
        assert_eq!(processes[1].user, "www-data");
        assert_eq!(processes[1].time, "0:01");
        assert_eq!(processes[1].command, "nginx: worker process");
    }

    #[test]
    fn test_parse_processes_empty() {
        let processes = TopCommand::parse_processes("");
        assert!(processes.is_empty());
    }

    #[test]
    fn test_parse_processes_headers_only() {
        let output = "PID   USER     TIME   COMMAND";
        let processes = TopCommand::parse_processes(output);
        assert!(processes.is_empty());
    }

    #[test]
    fn test_container_process_pid_as_int() {
        let process = ContainerProcess {
            pid: "1234".to_string(),
            user: "root".to_string(),
            time: "0:00".to_string(),
            command: "nginx".to_string(),
            raw_line: "1234 root 0:00 nginx".to_string(),
        };

        assert_eq!(process.pid_as_int(), Some(1234));
    }

    #[test]
    fn test_container_process_invalid_pid() {
        let process = ContainerProcess {
            pid: "invalid".to_string(),
            user: "root".to_string(),
            time: "0:00".to_string(),
            command: "nginx".to_string(),
            raw_line: "invalid root 0:00 nginx".to_string(),
        };

        assert_eq!(process.pid_as_int(), None);
    }
}
