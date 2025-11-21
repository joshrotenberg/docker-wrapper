//! Docker Compose start command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose start command builder
#[derive(Debug, Clone)]
pub struct ComposeStartCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to start (empty for all)
    pub services: Vec<String>,
}

/// Result from compose start command
#[derive(Debug, Clone)]
pub struct ComposeStartResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were started
    pub services: Vec<String>,
}

impl ComposeStartCommand {
    /// Create a new compose start command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
        }
    }

    /// Add a service to start
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to start
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }
}

impl Default for ComposeStartCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeStartCommand {
    type Output = ComposeStartResult;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        // Use the ComposeCommand implementation explicitly
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposeStartResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeStartCommand {
    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "start"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        // Start command just takes service names
        self.services.clone()
    }
}

impl ComposeStartResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were started
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_start_basic() {
        let cmd = ComposeStartCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"start".to_string()));
    }

    #[test]
    fn test_compose_start_with_services() {
        let cmd = ComposeStartCommand::new()
            .service("web")
            .service("db")
            .service("cache");

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "db", "cache"]);
    }

    #[test]
    fn test_compose_start_single_service() {
        let cmd = ComposeStartCommand::new().service("postgres");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["postgres"]);
    }

    #[test]
    fn test_compose_start_with_services_method() {
        let cmd = ComposeStartCommand::new().services(vec!["frontend", "backend"]);
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["frontend", "backend"]);
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeStartCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .service("api");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"api".to_string()));
    }
}
