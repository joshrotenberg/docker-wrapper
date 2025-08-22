//! Docker Compose logs command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose logs command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are appropriate for logs command
pub struct ComposeLogsCommand {
    /// Base compose configuration
    config: ComposeConfig,
    /// Services to show logs for (empty for all)
    services: Vec<String>,
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
    /// Don't print prefix
    no_log_prefix: bool,
    /// Don't use colors
    no_color: bool,
}

impl ComposeLogsCommand {
    /// Create a new compose logs command
    #[must_use]
    pub fn new() -> Self {
        Self {
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

    /// Create with a specific compose configuration
    #[must_use]
    pub fn with_config(config: ComposeConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Add a service to show logs for
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
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

    /// Set compose file
    #[must_use]
    pub fn file(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config = self.config.file(path);
        self
    }

    /// Set project name
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config = self.config.project_name(name);
        self
    }

    /// Execute the compose logs command
    ///
    /// # Errors
    /// Returns an error if:
    /// - Docker Compose is not installed
    /// - Compose file is not found
    /// - Service doesn't exist
    pub async fn run(&self) -> Result<ComposeLogsResult> {
        let output = self.execute().await?;

        Ok(ComposeLogsResult {
            output,
            services: self.services.clone(),
        })
    }
}

impl Default for ComposeLogsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ComposeCommand for ComposeLogsCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
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

    async fn execute(&self) -> Result<Self::Output> {
        execute_compose_command(&self.config, self.subcommand(), self.build_args()).await
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }
}

/// Result from compose logs command
#[derive(Debug, Clone)]
pub struct ComposeLogsResult {
    /// Raw command output
    pub output: ComposeOutput,
    /// Services logs were fetched for
    pub services: Vec<String>,
}

impl ComposeLogsResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the services
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
        let args = cmd.build_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_compose_logs_follow() {
        let cmd = ComposeLogsCommand::new().follow().timestamps();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--follow", "--timestamps"]);
    }

    #[test]
    fn test_compose_logs_with_tail() {
        let cmd = ComposeLogsCommand::new().tail("100").service("web");
        let args = cmd.build_args();
        assert_eq!(args, vec!["--tail", "100", "web"]);
    }

    #[test]
    fn test_compose_logs_all_options() {
        let cmd = ComposeLogsCommand::new()
            .follow()
            .timestamps()
            .tail("50")
            .since("2024-01-01T00:00:00")
            .no_color()
            .service("web")
            .service("db");

        let args = cmd.build_args();
        assert!(args.contains(&"--follow".to_string()));
        assert!(args.contains(&"--timestamps".to_string()));
        assert!(args.contains(&"--tail".to_string()));
        assert!(args.contains(&"50".to_string()));
        assert!(args.contains(&"--since".to_string()));
        assert!(args.contains(&"2024-01-01T00:00:00".to_string()));
        assert!(args.contains(&"--no-color".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }
}
