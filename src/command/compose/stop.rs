//! Docker Compose stop command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose stop command builder.
#[derive(Debug, Clone)]
pub struct ComposeStopCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Services to stop (empty for all).
    pub services: Vec<String>,
    /// Timeout for stopping containers.
    pub timeout: Option<Duration>,
}

/// Result from [`ComposeStopCommand`].
#[derive(Debug, Clone)]
pub struct ComposeStopResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Services that were stopped.
    pub services: Vec<String>,
}

impl ComposeStopCommand {
    /// Creates a new [`ComposeStopCommand`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            timeout: None,
        }
    }

    /// Adds a service to stop.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to stop.
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Sets the timeout for stopping containers.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl Default for ComposeStopCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeStopCommand {
    type Output = ComposeStopResult;

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

        Ok(ComposeStopResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeStopCommand {
    fn subcommand_name() -> &'static str {
        "stop"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(timeout) = self.timeout {
            args.push("--timeout".to_string());
            args.push(timeout.as_secs().to_string());
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeStopResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the services that were stopped.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_stop_basic() {
        let cmd = ComposeStopCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"stop".to_string()));
    }

    #[test]
    fn test_compose_stop_with_services() {
        let cmd = ComposeStopCommand::new().service("web").service("worker");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "worker"]);
    }

    #[test]
    fn test_compose_stop_with_timeout() {
        let cmd = ComposeStopCommand::new()
            .timeout(Duration::from_secs(10))
            .service("db");

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--timeout", "10", "db"]);
    }

    #[test]
    fn test_compose_stop_with_services_method() {
        let cmd = ComposeStopCommand::new().services(vec!["redis", "postgres"]);
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["redis", "postgres"]);
    }

    #[test]
    fn test_compose_stop_zero_timeout() {
        let cmd = ComposeStopCommand::new()
            .timeout(Duration::from_secs(0))
            .service("app");

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--timeout", "0", "app"]);
    }

    #[test]
    fn test_compose_stop_all_services_with_timeout() {
        let cmd = ComposeStopCommand::new().timeout(Duration::from_secs(30));
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--timeout", "30"]);
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeStopCommand::new()
            .file("compose.yml")
            .project_name("webapp")
            .timeout(Duration::from_secs(5))
            .service("redis");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"webapp".to_string()));
        assert!(args.contains(&"--timeout".to_string()));
        assert!(args.contains(&"5".to_string()));
        assert!(args.contains(&"redis".to_string()));
    }
}
