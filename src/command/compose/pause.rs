//! Docker Compose pause command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;

/// Docker Compose pause command builder.
#[derive(Debug, Clone)]
pub struct ComposePauseCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Services to pause (empty for all).
    pub services: Vec<String>,
}

/// Result from compose pause command.
#[derive(Debug, Clone)]
pub struct ComposePauseResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Services that were paused.
    pub services: Vec<String>,
}

impl ComposePauseCommand {
    /// Creates a new compose pause command.
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
        }
    }

    /// Adds a service to pause.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to pause.
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

impl Default for ComposePauseCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposePauseCommand {
    type Output = ComposePauseResult;

    fn command_name() -> &'static str {
        <Self as ComposeCommand>::command_name()
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposePauseResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposePauseCommand {
    fn subcommand_name() -> &'static str {
        "pause"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        self.services.clone()
    }
}

impl ComposePauseResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the services that were paused.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_pause_basic() {
        let cmd = ComposePauseCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"pause".to_string()));
    }

    #[test]
    fn test_compose_pause_with_services() {
        let cmd = ComposePauseCommand::new().service("web").service("worker");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "worker"]);
    }

    #[test]
    fn test_compose_pause_single_service() {
        let cmd = ComposePauseCommand::new().service("database");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["database"]);
    }

    #[test]
    fn test_compose_pause_with_services_method() {
        let cmd = ComposePauseCommand::new().services(vec!["cache", "queue"]);
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["cache", "queue"]);
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposePauseCommand::new()
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
