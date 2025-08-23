//! Docker Compose ps command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Docker Compose ps command builder
#[derive(Debug, Clone)]
pub struct ComposePsCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to list (empty for all)
    pub services: Vec<String>,
    /// Show all containers (including stopped)
    pub all: bool,
    /// Only display container IDs
    pub quiet: bool,
    /// Show services
    pub show_services: bool,
    /// Filter containers
    pub filter: Vec<String>,
    /// Output format
    pub format: Option<String>,
    /// Only show running containers
    pub status: Option<Vec<ContainerStatus>>,
}

/// Container status filter
#[derive(Debug, Clone, Copy)]
pub enum ContainerStatus {
    /// Paused containers
    Paused,
    /// Restarting containers
    Restarting,
    /// Running containers
    Running,
    /// Stopped containers
    Stopped,
}

impl std::fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Paused => write!(f, "paused"),
            Self::Restarting => write!(f, "restarting"),
            Self::Running => write!(f, "running"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// Container information from compose ps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeContainerInfo {
    /// Container ID
    #[serde(rename = "ID")]
    pub id: String,
    /// Container name
    #[serde(rename = "Name")]
    pub name: String,
    /// Service name
    #[serde(rename = "Service")]
    pub service: String,
    /// Container state
    #[serde(rename = "State")]
    pub state: String,
    /// Health status
    #[serde(rename = "Health")]
    pub health: Option<String>,
    /// Exit code
    #[serde(rename = "ExitCode")]
    pub exit_code: Option<i32>,
    /// Published ports
    #[serde(rename = "Publishers")]
    pub publishers: Option<Vec<PortPublisher>>,
}

/// Port publishing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortPublisher {
    /// Target port
    #[serde(rename = "TargetPort")]
    pub target_port: u16,
    /// Published port
    #[serde(rename = "PublishedPort")]
    pub published_port: Option<u16>,
    /// Protocol
    #[serde(rename = "Protocol")]
    pub protocol: String,
}

/// Result from compose ps command
#[derive(Debug, Clone)]
pub struct ComposePsResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Parsed container information (if JSON format)
    pub containers: Vec<ComposeContainerInfo>,
}

impl ComposePsCommand {
    /// Create a new compose ps command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            all: false,
            quiet: false,
            show_services: false,
            filter: Vec::new(),
            format: None,
            status: None,
        }
    }

    /// Add a service to list
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Show all containers (default shows only running)
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Only display container IDs
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Display services
    #[must_use]
    pub fn services(mut self) -> Self {
        self.show_services = true;
        self
    }

    /// Add a filter
    #[must_use]
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter.push(filter.into());
        self
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Filter by status
    #[must_use]
    pub fn status(mut self, status: ContainerStatus) -> Self {
        self.status.get_or_insert_with(Vec::new).push(status);
        self
    }

    /// Use JSON output format
    #[must_use]
    pub fn json(mut self) -> Self {
        self.format = Some("json".to_string());
        self
    }

    /// Parse JSON output into container info
    fn parse_json_output(stdout: &str) -> Vec<ComposeContainerInfo> {
        stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }
}

impl Default for ComposePsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommandV2 for ComposePsCommand {
    type Output = ComposePsResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        // Use the ComposeCommand implementation explicitly
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        // Parse JSON output if format is json
        let containers = if self.format.as_deref() == Some("json") {
            Self::parse_json_output(&output.stdout)
        } else {
            Vec::new()
        };

        Ok(ComposePsResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            containers,
        })
    }
}

impl ComposeCommand for ComposePsCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "ps"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.all {
            args.push("--all".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        if self.show_services {
            args.push("--services".to_string());
        }

        for filter in &self.filter {
            args.push("--filter".to_string());
            args.push(filter.clone());
        }

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if let Some(ref statuses) = self.status {
            for status in statuses {
                args.push("--status".to_string());
                args.push(status.to_string());
            }
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposePsResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get container information
    #[must_use]
    pub fn containers(&self) -> &[ComposeContainerInfo] {
        &self.containers
    }

    /// Get container IDs from output
    #[must_use]
    pub fn container_ids(&self) -> Vec<String> {
        if self.containers.is_empty() {
            // Parse from text output if not JSON
            self.stdout
                .lines()
                .skip(1) // Skip header
                .filter_map(|line| line.split_whitespace().next())
                .map(String::from)
                .collect()
        } else {
            self.containers.iter().map(|c| c.id.clone()).collect()
        }
    }

    /// Get stdout lines
    #[must_use]
    pub fn stdout_lines(&self) -> Vec<&str> {
        self.stdout.lines().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_ps_basic() {
        let cmd = ComposePsCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"ps".to_string()));
    }

    #[test]
    fn test_compose_ps_all() {
        let cmd = ComposePsCommand::new().all();
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--all"]);
    }

    #[test]
    fn test_compose_ps_with_format() {
        let cmd = ComposePsCommand::new().format("json").all();
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--all", "--format", "json"]);
    }

    #[test]
    fn test_compose_ps_with_filters() {
        let cmd = ComposePsCommand::new()
            .filter("status=running")
            .quiet()
            .service("web");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--quiet", "--filter", "status=running", "web"]);
    }

    #[test]
    fn test_container_status_display() {
        assert_eq!(ContainerStatus::Running.to_string(), "running");
        assert_eq!(ContainerStatus::Stopped.to_string(), "stopped");
        assert_eq!(ContainerStatus::Paused.to_string(), "paused");
        assert_eq!(ContainerStatus::Restarting.to_string(), "restarting");
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposePsCommand::new()
            .file("docker-compose.yml")
            .project_name("my-project")
            .all();

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--all".to_string()));
    }
}
