//! Docker stats command implementation.
//!
//! This module provides the `docker stats` command for displaying real-time
//! resource usage statistics of containers.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;

/// Docker stats command builder
///
/// Display a live stream of container(s) resource usage statistics.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::StatsCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get stats for all running containers
/// let stats = StatsCommand::new()
///     .run()
///     .await?;
///
/// // Get stats for specific containers
/// let stats = StatsCommand::new()
///     .container("my-container")
///     .container("another-container")
///     .no_stream()
///     .run()
///     .await?;
///
/// // Parse as JSON for programmatic use
/// let json_stats = StatsCommand::new()
///     .format("json")
///     .no_stream()
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct StatsCommand {
    /// Container names or IDs to monitor (empty = all running containers)
    containers: Vec<String>,
    /// Show all containers (default shows only running)
    all: bool,
    /// Pretty print images (default true)
    format: Option<String>,
    /// Disable streaming stats and only pull the first result
    no_stream: bool,
    /// Only display numeric IDs
    no_trunc: bool,
    /// Command executor
    executor: CommandExecutor,
}

impl StatsCommand {
    /// Create a new stats command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::StatsCommand;
    ///
    /// let cmd = StatsCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
            all: false,
            format: None,
            no_stream: false,
            no_trunc: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add a container to monitor
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::StatsCommand;
    ///
    /// let cmd = StatsCommand::new()
    ///     .container("web-server")
    ///     .container("database");
    /// ```
    #[must_use]
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Add multiple containers to monitor
    #[must_use]
    pub fn containers(mut self, containers: Vec<impl Into<String>>) -> Self {
        self.containers
            .extend(containers.into_iter().map(Into::into));
        self
    }

    /// Show stats for all containers (default shows only running)
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::StatsCommand;
    ///
    /// let cmd = StatsCommand::new().all();
    /// ```
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Set output format
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::StatsCommand;
    ///
    /// // JSON format for programmatic parsing
    /// let cmd = StatsCommand::new().format("json");
    ///
    /// // Table format (default)
    /// let cmd = StatsCommand::new().format("table");
    /// ```
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Disable streaming stats and only pull the first result
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::StatsCommand;
    ///
    /// let cmd = StatsCommand::new().no_stream();
    /// ```
    #[must_use]
    pub fn no_stream(mut self) -> Self {
        self.no_stream = true;
        self
    }

    /// Do not truncate output (show full container IDs)
    #[must_use]
    pub fn no_trunc(mut self) -> Self {
        self.no_trunc = true;
        self
    }

    /// Execute the stats command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Any specified container doesn't exist
    /// - No containers are running (when no specific containers are specified)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::StatsCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = StatsCommand::new()
    ///     .container("my-container")
    ///     .no_stream()
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Container stats:\n{}", result.output.stdout);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<StatsResult> {
        let output = self.execute().await?;

        // Parse stats if JSON format was used
        let parsed_stats = if self.format.as_deref() == Some("json") {
            Self::parse_json_stats(&output.stdout)
        } else {
            Vec::new()
        };

        Ok(StatsResult {
            output,
            containers: self.containers.clone(),
            parsed_stats,
        })
    }

    /// Parse JSON stats output into structured data
    fn parse_json_stats(stdout: &str) -> Vec<ContainerStats> {
        let mut stats = Vec::new();

        // Docker stats JSON output can be either a single object or multiple lines of JSON
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(stat) = serde_json::from_str::<ContainerStats>(line) {
                stats.push(stat);
            }
        }

        stats
    }
}

impl Default for StatsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for StatsCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "stats"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.all {
            args.push("--all".to_string());
        }

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if self.no_stream {
            args.push("--no-stream".to_string());
        }

        if self.no_trunc {
            args.push("--no-trunc".to_string());
        }

        // Add container names/IDs
        args.extend(self.containers.clone());

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

/// Result from the stats command
#[derive(Debug, Clone)]
pub struct StatsResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Containers that were monitored
    pub containers: Vec<String>,
    /// Parsed stats (when JSON format is used)
    pub parsed_stats: Vec<ContainerStats>,
}

impl StatsResult {
    /// Check if the stats command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the monitored container names
    #[must_use]
    pub fn containers(&self) -> &[String] {
        &self.containers
    }

    /// Get parsed stats (available when JSON format is used)
    #[must_use]
    pub fn parsed_stats(&self) -> &[ContainerStats] {
        &self.parsed_stats
    }

    /// Get the raw stats output
    #[must_use]
    pub fn raw_output(&self) -> &str {
        &self.output.stdout
    }
}

/// Container resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    /// Container ID
    #[serde(alias = "Container")]
    pub container_id: String,

    /// Container name
    #[serde(alias = "Name")]
    pub name: String,

    /// CPU usage percentage
    #[serde(alias = "CPUPerc")]
    pub cpu_percent: String,

    /// Memory usage
    #[serde(alias = "MemUsage")]
    pub memory_usage: String,

    /// Memory usage percentage
    #[serde(alias = "MemPerc")]
    pub memory_percent: String,

    /// Network I/O
    #[serde(alias = "NetIO")]
    pub network_io: String,

    /// Block I/O
    #[serde(alias = "BlockIO")]
    pub block_io: String,

    /// Number of process IDs (PIDs)
    #[serde(alias = "PIDs")]
    pub pids: String,
}

impl ContainerStats {
    /// Get CPU percentage as a float (removes % sign)
    #[must_use]
    pub fn cpu_percentage(&self) -> Option<f64> {
        self.cpu_percent.trim_end_matches('%').parse().ok()
    }

    /// Get memory percentage as a float (removes % sign)
    #[must_use]
    pub fn memory_percentage(&self) -> Option<f64> {
        self.memory_percent.trim_end_matches('%').parse().ok()
    }

    /// Get number of PIDs as integer
    #[must_use]
    pub fn pid_count(&self) -> Option<u32> {
        self.pids.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_basic() {
        let cmd = StatsCommand::new();
        let args = cmd.build_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_stats_with_containers() {
        let cmd = StatsCommand::new().container("web").container("db");
        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "db"]);
    }

    #[test]
    fn test_stats_with_all_flag() {
        let cmd = StatsCommand::new().all();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--all"]);
    }

    #[test]
    fn test_stats_with_format() {
        let cmd = StatsCommand::new().format("json");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--format", "json"]);
    }

    #[test]
    fn test_stats_no_stream() {
        let cmd = StatsCommand::new().no_stream();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--no-stream"]);
    }

    #[test]
    fn test_stats_no_trunc() {
        let cmd = StatsCommand::new().no_trunc();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--no-trunc"]);
    }

    #[test]
    fn test_stats_all_options() {
        let cmd = StatsCommand::new()
            .all()
            .format("table")
            .no_stream()
            .no_trunc()
            .container("test-container");
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "--all",
                "--format",
                "table",
                "--no-stream",
                "--no-trunc",
                "test-container"
            ]
        );
    }

    #[test]
    fn test_container_stats_parsing() {
        let stats = ContainerStats {
            container_id: "abc123".to_string(),
            name: "test-container".to_string(),
            cpu_percent: "1.23%".to_string(),
            memory_usage: "512MiB / 2GiB".to_string(),
            memory_percent: "25.00%".to_string(),
            network_io: "1.2kB / 3.4kB".to_string(),
            block_io: "4.5MB / 6.7MB".to_string(),
            pids: "42".to_string(),
        };

        assert_eq!(stats.cpu_percentage(), Some(1.23));
        assert_eq!(stats.memory_percentage(), Some(25.0));
        assert_eq!(stats.pid_count(), Some(42));
    }

    #[test]
    fn test_parse_json_stats() {
        let json_output = r#"{"Container":"abc123","Name":"test","CPUPerc":"1.23%","MemUsage":"512MiB / 2GiB","MemPerc":"25.00%","NetIO":"1.2kB / 3.4kB","BlockIO":"4.5MB / 6.7MB","PIDs":"42"}"#;

        let stats = StatsCommand::parse_json_stats(json_output);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "test");
        assert_eq!(stats[0].container_id, "abc123");
    }

    #[test]
    fn test_parse_json_stats_multiple_lines() {
        let json_output = r#"{"Container":"abc123","Name":"test1","CPUPerc":"1.23%","MemUsage":"512MiB / 2GiB","MemPerc":"25.00%","NetIO":"1.2kB / 3.4kB","BlockIO":"4.5MB / 6.7MB","PIDs":"42"}
{"Container":"def456","Name":"test2","CPUPerc":"2.34%","MemUsage":"1GiB / 4GiB","MemPerc":"25.00%","NetIO":"2.3kB / 4.5kB","BlockIO":"5.6MB / 7.8MB","PIDs":"24"}"#;

        let stats = StatsCommand::parse_json_stats(json_output);
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].name, "test1");
        assert_eq!(stats[1].name, "test2");
    }

    #[test]
    fn test_parse_json_stats_empty() {
        let stats = StatsCommand::parse_json_stats("");
        assert!(stats.is_empty());
    }
}
