//! Docker Compose stop command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose stop command builder
#[derive(Debug, Clone, Default)]
pub struct ComposeStopCommand {
    config: ComposeConfig,
    services: Vec<String>,
    timeout: Option<Duration>,
}

impl ComposeStopCommand {
    /// Create a new compose stop command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a specific configuration
    #[must_use]
    pub fn with_config(config: ComposeConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Add a service to stop
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Set the timeout for stopping containers
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Execute the stop command
    ///
    /// # Errors
    ///
    /// Returns an error if the docker compose stop command fails
    pub async fn run(&self) -> Result<ComposeOutput> {
        self.execute().await
    }
}

#[async_trait]
impl ComposeCommand for ComposeStopCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
        "stop"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(timeout) = self.timeout {
            args.push("--timeout".to_string());
            args.push(timeout.as_secs().to_string());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_stop_basic() {
        let cmd = ComposeStopCommand::new();
        assert_eq!(cmd.subcommand(), "stop");
        assert_eq!(cmd.build_args(), Vec::<String>::new());
    }

    #[test]
    fn test_compose_stop_with_services() {
        let cmd = ComposeStopCommand::new().service("web").service("worker");

        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "worker"]);
    }

    #[test]
    fn test_compose_stop_with_timeout() {
        let cmd = ComposeStopCommand::new()
            .timeout(Duration::from_secs(10))
            .service("db");

        let args = cmd.build_args();
        assert_eq!(args, vec!["--timeout", "10", "db"]);
    }

    #[test]
    fn test_compose_stop_with_config() {
        let config = ComposeConfig::new()
            .file("compose.yml")
            .project_name("webapp");

        let cmd = ComposeStopCommand::with_config(config)
            .timeout(Duration::from_secs(5))
            .service("redis")
            .service("postgres");

        assert_eq!(cmd.config().project_name, Some("webapp".to_string()));
        let args = cmd.build_args();
        assert_eq!(args, vec!["--timeout", "5", "redis", "postgres"]);
    }

    #[test]
    fn test_compose_stop_zero_timeout() {
        let cmd = ComposeStopCommand::new()
            .timeout(Duration::from_secs(0))
            .service("app");

        let args = cmd.build_args();
        assert_eq!(args, vec!["--timeout", "0", "app"]);
    }

    #[test]
    fn test_compose_stop_long_timeout() {
        let cmd = ComposeStopCommand::new()
            .timeout(Duration::from_secs(300))
            .service("slow-service");

        let args = cmd.build_args();
        assert_eq!(args, vec!["--timeout", "300", "slow-service"]);
    }

    #[test]
    fn test_compose_stop_all_services() {
        // Stop all services when no specific services are specified
        let cmd = ComposeStopCommand::new().timeout(Duration::from_secs(30));

        let args = cmd.build_args();
        assert_eq!(args, vec!["--timeout", "30"]);
    }
}
