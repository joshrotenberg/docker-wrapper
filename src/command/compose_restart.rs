//! Docker Compose restart command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose restart command builder
#[derive(Debug, Clone)]
pub struct ComposeRestartCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to restart (empty for all)
    pub services: Vec<String>,
    /// Timeout for stopping containers before restarting
    pub timeout: Option<Duration>,
}

/// Result from compose restart command
#[derive(Debug, Clone)]
pub struct ComposeRestartResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were restarted
    pub services: Vec<String>,
}

impl ComposeRestartCommand {
    /// Create a new compose restart command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            timeout: None,
        }
    }

    /// Add a service to restart
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to restart
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Set the timeout for stopping containers before restarting
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl Default for ComposeRestartCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommandV2 for ComposeRestartCommand {
    type Output = ComposeRestartResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        // Use the ComposeCommand implementation explicitly
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposeRestartResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeRestartCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "restart"
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

impl ComposeRestartResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were restarted
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_restart_basic() {
        let cmd = ComposeRestartCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"restart".to_string()));
    }

    #[test]
    fn test_compose_restart_with_services() {
        let cmd = ComposeRestartCommand::new().service("web").service("db");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "db"]);
    }

    #[test]
    fn test_compose_restart_with_timeout() {
        let cmd = ComposeRestartCommand::new()
            .timeout(Duration::from_secs(30))
            .service("app");

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--timeout", "30", "app"]);
    }

    #[test]
    fn test_compose_restart_with_services_method() {
        let cmd = ComposeRestartCommand::new().services(vec!["service1", "service2"]);
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["service1", "service2"]);
    }

    #[test]
    fn test_compose_restart_builder_pattern() {
        let cmd = ComposeRestartCommand::new()
            .service("service1")
            .service("service2")
            .timeout(Duration::from_secs(60));

        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--timeout", "60", "service1", "service2"]);
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeRestartCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .timeout(Duration::from_secs(10))
            .service("web");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"--timeout".to_string()));
        assert!(args.contains(&"10".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}