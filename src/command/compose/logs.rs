//! Docker Compose logs command implementation using unified trait pattern.

use crate::command::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose logs command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are appropriate for logs command
pub struct ComposeLogsCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to show logs for (empty for all)
    pub services: Vec<String>,
    /// Follow log output
    pub follow: bool,
    /// Show timestamps
    pub timestamps: bool,
    /// Number of lines to show from the end
    pub tail: Option<String>,
    /// Show logs since timestamp
    pub since: Option<String>,
    /// Show logs until timestamp
    pub until: Option<String>,
    /// Don't print prefix
    pub no_log_prefix: bool,
    /// Don't use colors
    pub no_color: bool,
}

/// Result from compose logs command
#[derive(Debug, Clone)]
pub struct ComposeLogsResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services logs were fetched for
    pub services: Vec<String>,
}

impl ComposeLogsCommand {
    /// Create a new compose logs command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            follow: false,
            timestamps: false,
            tail: None,
            since: None,
            until: None,
            no_log_prefix: false,
            no_color: false,
        }
    }

    /// Add a service to show logs for
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Follow log output
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

    /// Number of lines to show from the end
    #[must_use]
    pub fn tail(mut self, lines: impl Into<String>) -> Self {
        self.tail = Some(lines.into());
        self
    }

    /// Show logs since timestamp
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

    /// Don't print prefix
    #[must_use]
    pub fn no_log_prefix(mut self) -> Self {
        self.no_log_prefix = true;
        self
    }

    /// Don't use colors
    #[must_use]
    pub fn no_color(mut self) -> Self {
        self.no_color = true;
        self
    }
}

impl Default for ComposeLogsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeLogsCommand {
    type Output = ComposeLogsResult;

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

        Ok(ComposeLogsResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeLogsCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "logs"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
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

        if self.no_log_prefix {
            args.push("--no-log-prefix".to_string());
        }

        if self.no_color {
            args.push("--no-color".to_string());
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeLogsResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services logs were fetched for
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_logs_basic() {
        let cmd = ComposeLogsCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"logs".to_string()));
    }

    #[test]
    fn test_compose_logs_follow() {
        let cmd = ComposeLogsCommand::new().follow().timestamps();
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--follow", "--timestamps"]);
    }

    #[test]
    fn test_compose_logs_with_tail() {
        let cmd = ComposeLogsCommand::new().tail("100").service("web");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--tail", "100", "web"]);
    }

    #[test]
    fn test_compose_logs_with_services() {
        let cmd = ComposeLogsCommand::new()
            .services(vec!["web", "db"])
            .follow();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--follow".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_compose_logs_all_options() {
        let cmd = ComposeLogsCommand::new()
            .follow()
            .timestamps()
            .tail("50")
            .since("2024-01-01T00:00:00")
            .until("2024-01-02T00:00:00")
            .no_color()
            .no_log_prefix()
            .service("web")
            .service("db");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--follow".to_string()));
        assert!(args.contains(&"--timestamps".to_string()));
        assert!(args.contains(&"--tail".to_string()));
        assert!(args.contains(&"50".to_string()));
        assert!(args.contains(&"--since".to_string()));
        assert!(args.contains(&"2024-01-01T00:00:00".to_string()));
        assert!(args.contains(&"--until".to_string()));
        assert!(args.contains(&"2024-01-02T00:00:00".to_string()));
        assert!(args.contains(&"--no-color".to_string()));
        assert!(args.contains(&"--no-log-prefix".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeLogsCommand::new()
            .file("docker-compose.yml")
            .project_name("my-project")
            .follow()
            .service("api");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--follow".to_string()));
        assert!(args.contains(&"api".to_string()));
    }
}
