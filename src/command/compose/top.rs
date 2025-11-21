//! Docker Compose top command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;

/// Docker Compose top command builder.
#[derive(Debug, Clone)]
pub struct ComposeTopCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Services to show processes for (empty for all).
    pub services: Vec<String>,
}

/// Result from compose top command.
#[derive(Debug, Clone)]
pub struct ComposeTopResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Services that were queried.
    pub services: Vec<String>,
}

impl ComposeTopCommand {
    /// Creates a new compose top command.
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
        }
    }

    /// Adds a service to show processes for.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to show processes for.
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

impl Default for ComposeTopCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeTopCommand {
    type Output = ComposeTopResult;

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

        Ok(ComposeTopResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeTopCommand {
    fn subcommand_name() -> &'static str {
        "top"
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

impl ComposeTopResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the services that were queried.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_top_basic() {
        let cmd = ComposeTopCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"top".to_string()));
    }

    #[test]
    fn test_compose_top_with_services() {
        let cmd = ComposeTopCommand::new().services(vec!["web", "db"]);
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "db"]);
    }
}
