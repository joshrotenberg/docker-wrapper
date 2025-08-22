//! Docker Compose restart command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose restart command builder
#[derive(Debug, Clone, Default)]
pub struct ComposeRestartCommand {
    config: ComposeConfig,
    services: Vec<String>,
    timeout: Option<Duration>,
}

impl ComposeRestartCommand {
    /// Create a new compose restart command
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

    /// Add a service to restart
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

    /// Execute the restart command
    ///
    /// # Errors
    ///
    /// Returns an error if the docker compose restart command fails
    pub async fn run(&self) -> Result<ComposeOutput> {
        self.execute().await
    }
}

#[async_trait]
impl ComposeCommand for ComposeRestartCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
        "restart"
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
    fn test_compose_restart_basic() {
        let cmd = ComposeRestartCommand::new();
        assert_eq!(cmd.subcommand(), "restart");
        assert_eq!(cmd.build_args(), Vec::<String>::new());
    }

    #[test]
    fn test_compose_restart_with_services() {
        let cmd = ComposeRestartCommand::new().service("web").service("db");

        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "db"]);
    }

    #[test]
    fn test_compose_restart_with_timeout() {
        let cmd = ComposeRestartCommand::new()
            .timeout(Duration::from_secs(30))
            .service("app");

        let args = cmd.build_args();
        assert_eq!(args, vec!["--timeout", "30", "app"]);
    }

    #[test]
    fn test_compose_restart_with_config() {
        let config = ComposeConfig::new()
            .file("docker-compose.yml")
            .project_name("myapp");

        let cmd = ComposeRestartCommand::with_config(config)
            .service("web")
            .timeout(Duration::from_secs(10));

        assert_eq!(cmd.config().project_name, Some("myapp".to_string()));
        let args = cmd.build_args();
        assert_eq!(args, vec!["--timeout", "10", "web"]);
    }

    #[test]
    fn test_compose_restart_builder_pattern() {
        let cmd = ComposeRestartCommand::new()
            .service("service1")
            .service("service2")
            .service("service3")
            .timeout(Duration::from_secs(60));

        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["--timeout", "60", "service1", "service2", "service3"]
        );
    }

    #[test]
    fn test_compose_restart_no_timeout() {
        let cmd = ComposeRestartCommand::new().service("nginx");

        let args = cmd.build_args();
        assert_eq!(args, vec!["nginx"]);
    }
}
