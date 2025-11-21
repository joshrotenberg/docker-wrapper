//! Docker Compose kill command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;

/// Docker Compose kill command builder.
#[derive(Debug, Clone)]
pub struct ComposeKillCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Services to kill (empty for all).
    pub services: Vec<String>,
    /// Signal to send to containers.
    pub signal: Option<String>,
}

/// Result from [`ComposeKillCommand`].
#[derive(Debug, Clone)]
pub struct ComposeKillResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Services that were killed.
    pub services: Vec<String>,
}

impl ComposeKillCommand {
    /// Creates a new [`ComposeKillCommand`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            signal: None,
        }
    }

    /// Adds a service to kill.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to kill.
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Sets signal to send to containers.
    #[must_use]
    pub fn signal(mut self, signal: impl Into<String>) -> Self {
        self.signal = Some(signal.into());
        self
    }
}

impl Default for ComposeKillCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeKillCommand {
    type Output = ComposeKillResult;

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

        Ok(ComposeKillResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeKillCommand {
    fn subcommand_name() -> &'static str {
        "kill"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref signal) = self.signal {
            args.push("--signal".to_string());
            args.push(signal.clone());
        }

        // add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeKillResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the services that were killed.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_kill_basic() {
        let cmd = ComposeKillCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"kill".to_string()));
    }

    #[test]
    fn test_compose_kill_with_services() {
        let cmd = ComposeKillCommand::new().service("web").service("worker");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "worker"]);
    }

    #[test]
    fn test_compose_kill_with_signal() {
        let cmd = ComposeKillCommand::new().signal("SIGTERM").service("app");

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--signal", "SIGTERM", "app"]);
    }

    #[test]
    fn test_compose_kill_with_numeric_signal() {
        let cmd = ComposeKillCommand::new().signal("15").service("database");

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--signal", "15", "database"]);
    }

    #[test]
    fn test_compose_kill_multiple_services() {
        let cmd = ComposeKillCommand::new()
            .services(vec!["frontend", "backend"])
            .signal("SIGINT");

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--signal", "SIGINT", "frontend", "backend"]);
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeKillCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .signal("SIGTERM")
            .service("web");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"--signal".to_string()));
        assert!(args.contains(&"SIGTERM".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}
