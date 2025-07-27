//! Docker ps command implementation.
//!
//! This module provides a comprehensive implementation of the `docker ps` command
//! with support for all native options and an extensible architecture for any additional options.

use super::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::ffi::OsStr;

/// Docker ps command builder with fluent API
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct PsCommand {
    /// Command executor for extensibility
    executor: CommandExecutor,
    /// Show all containers (default shows just running)
    all: bool,
    /// Filter output based on conditions provided
    filters: Vec<String>,
    /// Format output using a custom template
    format: Option<String>,
    /// Show n last created containers (includes all states)
    last: Option<i32>,
    /// Show the latest created container (includes all states)
    latest: bool,
    /// Don't truncate output
    no_trunc: bool,
    /// Only display container IDs
    quiet: bool,
    /// Display total file sizes
    size: bool,
}

/// Container information returned by docker ps
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerInfo {
    /// Container ID
    pub id: String,
    /// Container image
    pub image: String,
    /// Command being run
    pub command: String,
    /// Creation time
    pub created: String,
    /// Container status
    pub status: String,
    /// Port mappings
    pub ports: String,
    /// Container names
    pub names: String,
}

/// Output format for ps command
#[derive(Debug, Clone)]
pub enum PsFormat {
    /// Default table format
    Table,
    /// JSON format
    Json,
    /// Custom Go template
    Template(String),
    /// Raw output (when using quiet mode)
    Raw,
}

/// Output from docker ps command
#[derive(Debug, Clone)]
pub struct PsOutput {
    /// The raw stdout from the command
    pub stdout: String,
    /// The raw stderr from the command
    pub stderr: String,
    /// Exit code from the command
    pub exit_code: i32,
    /// Parsed container information (when possible)
    pub containers: Vec<ContainerInfo>,
}

impl PsOutput {
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

    /// Get container IDs only (useful when using quiet mode)
    #[must_use]
    pub fn container_ids(&self) -> Vec<String> {
        self.stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    }

    /// Get number of containers found
    #[must_use]
    pub fn container_count(&self) -> usize {
        self.containers.len()
    }
}

impl PsCommand {
    /// Create a new ps command
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            all: false,
            filters: Vec::new(),
            format: None,
            last: None,
            latest: false,
            no_trunc: false,
            quiet: false,
            size: false,
        }
    }

    /// Show all containers (default shows just running)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().all();
    /// ```
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Filter output based on conditions provided
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new()
    ///     .filter("status=running")
    ///     .filter("name=my-container");
    /// ```
    #[must_use]
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filters.push(filter.into());
        self
    }

    /// Add multiple filters
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let filters = vec!["status=running".to_string(), "name=web".to_string()];
    /// let ps_cmd = PsCommand::new().filters(filters);
    /// ```
    #[must_use]
    pub fn filters(mut self, filters: Vec<String>) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Format output using table format
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().format_table();
    /// ```
    #[must_use]
    pub fn format_table(mut self) -> Self {
        self.format = Some("table".to_string());
        self
    }

    /// Format output using JSON format
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().format_json();
    /// ```
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some("json".to_string());
        self
    }

    /// Format output using a custom Go template
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new()
    ///     .format_template("table {{.ID}}\\t{{.Names}}\\t{{.Status}}");
    /// ```
    #[must_use]
    pub fn format_template(mut self, template: impl Into<String>) -> Self {
        self.format = Some(template.into());
        self
    }

    /// Show n last created containers (includes all states)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().last(5);
    /// ```
    #[must_use]
    pub fn last(mut self, n: i32) -> Self {
        self.last = Some(n);
        self
    }

    /// Show the latest created container (includes all states)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().latest();
    /// ```
    #[must_use]
    pub fn latest(mut self) -> Self {
        self.latest = true;
        self
    }

    /// Don't truncate output
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().no_trunc();
    /// ```
    #[must_use]
    pub fn no_trunc(mut self) -> Self {
        self.no_trunc = true;
        self
    }

    /// Only display container IDs
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().quiet();
    /// ```
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Display total file sizes
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ps::PsCommand;
    ///
    /// let ps_cmd = PsCommand::new().size();
    /// ```
    #[must_use]
    pub fn size(mut self) -> Self {
        self.size = true;
        self
    }

    /// Parse container info from table output (best effort)
    fn parse_table_output(output: &str) -> Vec<ContainerInfo> {
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() < 2 {
            return Vec::new(); // No header or data
        }

        let mut containers = Vec::new();

        // Skip header line
        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            // Basic parsing - this is best effort since docker ps output can vary
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                containers.push(ContainerInfo {
                    id: parts[0].to_string(),
                    image: parts[1].to_string(),
                    command: (*parts.get(2).unwrap_or(&"")).to_string(),
                    created: (*parts.get(3).unwrap_or(&"")).to_string(),
                    status: (*parts.get(4).unwrap_or(&"")).to_string(),
                    ports: (*parts.get(5).unwrap_or(&"")).to_string(),
                    names: (*parts.get(6).unwrap_or(&"")).to_string(),
                });
            }
        }

        containers
    }

    /// Parse container info from JSON output
    fn parse_json_output(output: &str) -> Vec<ContainerInfo> {
        // Try to parse as JSON array
        if let Ok(containers) = serde_json::from_str::<Vec<serde_json::Value>>(output) {
            return containers
                .into_iter()
                .filter_map(|container| {
                    Some(ContainerInfo {
                        id: container.get("ID")?.as_str()?.to_string(),
                        image: container.get("Image")?.as_str()?.to_string(),
                        command: container.get("Command")?.as_str()?.to_string(),
                        created: container.get("CreatedAt")?.as_str()?.to_string(),
                        status: container.get("Status")?.as_str()?.to_string(),
                        ports: container.get("Ports")?.as_str().unwrap_or("").to_string(),
                        names: container.get("Names")?.as_str()?.to_string(),
                    })
                })
                .collect();
        }

        Vec::new()
    }
}

impl Default for PsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for PsCommand {
    type Output = PsOutput;

    fn command_name(&self) -> &'static str {
        "ps"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.all {
            args.push("--all".to_string());
        }

        for filter in &self.filters {
            args.push("--filter".to_string());
            args.push(filter.clone());
        }

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if let Some(last) = self.last {
            args.push("--last".to_string());
            args.push(last.to_string());
        }

        if self.latest {
            args.push("--latest".to_string());
        }

        if self.no_trunc {
            args.push("--no-trunc".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        if self.size {
            args.push("--size".to_string());
        }

        // Add any additional raw arguments
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self
            .executor
            .execute_command(self.command_name(), args)
            .await?;

        // Parse containers based on format
        let containers = if self.quiet {
            // In quiet mode, we just get container IDs
            Vec::new()
        } else if let Some(ref format) = self.format {
            if format == "json" {
                Self::parse_json_output(&output.stdout)
            } else {
                Self::parse_table_output(&output.stdout)
            }
        } else {
            // Default table format
            Self::parse_table_output(&output.stdout)
        };

        Ok(PsOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
            containers,
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
    fn test_ps_command_builder() {
        let cmd = PsCommand::new()
            .all()
            .filter("status=running")
            .filter("name=web")
            .format_json()
            .no_trunc()
            .size();

        let args = cmd.build_args();

        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.contains(&"status=running".to_string()));
        assert!(args.contains(&"name=web".to_string()));
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
        assert!(args.contains(&"--no-trunc".to_string()));
        assert!(args.contains(&"--size".to_string()));
    }

    #[test]
    fn test_ps_command_quiet() {
        let cmd = PsCommand::new().quiet().all();

        let args = cmd.build_args();

        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--all".to_string()));
    }

    #[test]
    fn test_ps_command_latest() {
        let cmd = PsCommand::new().latest();

        let args = cmd.build_args();

        assert!(args.contains(&"--latest".to_string()));
    }

    #[test]
    fn test_ps_command_last() {
        let cmd = PsCommand::new().last(5);

        let args = cmd.build_args();

        assert!(args.contains(&"--last".to_string()));
        assert!(args.contains(&"5".to_string()));
    }

    #[test]
    fn test_ps_command_multiple_filters() {
        let filters = vec!["status=running".to_string(), "name=web".to_string()];
        let cmd = PsCommand::new().filters(filters);

        let args = cmd.build_args();

        // Should have two --filter entries
        let filter_count = args.iter().filter(|&arg| arg == "--filter").count();
        assert_eq!(filter_count, 2);
        assert!(args.contains(&"status=running".to_string()));
        assert!(args.contains(&"name=web".to_string()));
    }

    #[test]
    fn test_ps_command_format_variants() {
        let cmd1 = PsCommand::new().format_table();
        assert!(cmd1.build_args().contains(&"table".to_string()));

        let cmd2 = PsCommand::new().format_json();
        assert!(cmd2.build_args().contains(&"json".to_string()));

        let cmd3 = PsCommand::new().format_template("{{.ID}}");
        assert!(cmd3.build_args().contains(&"{{.ID}}".to_string()));
    }

    #[test]
    fn test_ps_output_helpers() {
        let output = PsOutput {
            stdout: "container1\ncontainer2\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            containers: Vec::new(),
        };

        assert!(output.success());
        assert!(!output.stdout_is_empty());
        assert!(output.stderr_is_empty());

        let ids = output.container_ids();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "container1");
        assert_eq!(ids[1], "container2");
    }

    #[test]
    fn test_ps_command_extensibility() {
        let mut cmd = PsCommand::new();

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

    #[test]
    fn test_container_info_creation() {
        let info = ContainerInfo {
            id: "abc123".to_string(),
            image: "nginx:latest".to_string(),
            command: "\"/docker-entrypoint.sh nginx -g 'daemon off;'\"".to_string(),
            created: "2 minutes ago".to_string(),
            status: "Up 2 minutes".to_string(),
            ports: "0.0.0.0:8080->80/tcp".to_string(),
            names: "web-server".to_string(),
        };

        assert_eq!(info.id, "abc123");
        assert_eq!(info.image, "nginx:latest");
        assert_eq!(info.names, "web-server");
    }
}
