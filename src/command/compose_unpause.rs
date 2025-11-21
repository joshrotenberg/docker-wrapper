//! Docker Compose unpause command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose unpause command builder
#[derive(Debug, Clone)]
pub struct ComposeUnpauseCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to unpause (empty for all)
    pub services: Vec<String>,
}

/// Result from compose unpause command
#[derive(Debug, Clone)]
pub struct ComposeUnpauseResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were unpaused
    pub services: Vec<String>,
}

impl ComposeUnpauseCommand {
    /// Create a new compose unpause command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
        }
    }

    /// Add a service to unpause
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to unpause
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

impl Default for ComposeUnpauseCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeUnpauseCommand {
    type Output = ComposeUnpauseResult;

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

        Ok(ComposeUnpauseResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeUnpauseCommand {
    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "unpause"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        // Unpause command just takes service names
        self.services.clone()
    }
}

impl ComposeUnpauseResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were unpaused
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_unpause_basic() {
        let cmd = ComposeUnpauseCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"unpause".to_string()));
    }

    #[test]
    fn test_compose_unpause_with_services() {
        let cmd = ComposeUnpauseCommand::new()
            .service("web")
            .service("worker");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "worker"]);
    }

    #[test]
    fn test_compose_unpause_single_service() {
        let cmd = ComposeUnpauseCommand::new().service("database");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["database"]);
    }

    #[test]
    fn test_compose_unpause_with_services_method() {
        let cmd = ComposeUnpauseCommand::new().services(vec!["cache", "queue"]);
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["cache", "queue"]);
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeUnpauseCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .service("web");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}
