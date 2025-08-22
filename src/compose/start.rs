//! Docker Compose start command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose start command builder
#[derive(Debug, Clone, Default)]
pub struct ComposeStartCommand {
    config: ComposeConfig,
    services: Vec<String>,
}

impl ComposeStartCommand {
    /// Create a new compose start command
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

    /// Add a service to start
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Execute the start command
    ///
    /// # Errors
    ///
    /// Returns an error if the docker compose start command fails
    pub async fn run(&self) -> Result<ComposeOutput> {
        self.execute().await
    }
}

#[async_trait]
impl ComposeCommand for ComposeStartCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
        "start"
    }

    fn build_args(&self) -> Vec<String> {
        self.services.clone()
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
    fn test_compose_start_basic() {
        let cmd = ComposeStartCommand::new();
        assert_eq!(cmd.subcommand(), "start");
        assert_eq!(cmd.build_args(), Vec::<String>::new());
    }

    #[test]
    fn test_compose_start_with_services() {
        let cmd = ComposeStartCommand::new()
            .service("web")
            .service("db")
            .service("cache");

        let args = cmd.build_args();
        assert_eq!(args, vec!["web", "db", "cache"]);
    }

    #[test]
    fn test_compose_start_with_config() {
        let config = ComposeConfig::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .profile("production");

        let cmd = ComposeStartCommand::with_config(config).service("api");

        assert_eq!(cmd.config().project_name, Some("myapp".to_string()));
        assert_eq!(cmd.config().profiles, vec!["production"]);
        assert_eq!(cmd.build_args(), vec!["api"]);
    }

    #[test]
    fn test_compose_start_single_service() {
        let cmd = ComposeStartCommand::new().service("postgres");

        let args = cmd.build_args();
        assert_eq!(args, vec!["postgres"]);
    }

    #[test]
    fn test_compose_start_no_services() {
        // Starting all services when no specific services are specified
        let cmd = ComposeStartCommand::new();

        let args = cmd.build_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_compose_start_builder_pattern() {
        let cmd = ComposeStartCommand::with_config(ComposeConfig::new().project_name("test"))
            .service("frontend")
            .service("backend");

        assert_eq!(cmd.services.len(), 2);
        assert!(cmd.services.contains(&"frontend".to_string()));
        assert!(cmd.services.contains(&"backend".to_string()));
    }
}
