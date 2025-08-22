//! Docker Compose run command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose run command builder
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Stub implementation
pub struct ComposeRunCommand {
    config: ComposeConfig,
    service: String,
    command: Vec<String>,
    detach: bool,
    rm: bool,
}

impl ComposeRunCommand {
    /// Create a new compose run command
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            ..Self::default()
        }
    }

    /// Execute the run command
    ///
    /// # Errors
    ///
    /// Returns an error if the docker compose run command fails
    pub async fn run(&self) -> Result<ComposeOutput> {
        self.execute().await
    }
}

#[async_trait]
impl ComposeCommand for ComposeRunCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
        "run"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec![self.service.clone()];
        args.extend(self.command.clone());
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
    fn test_compose_run_basic() {
        let cmd = ComposeRunCommand::new("web");
        assert_eq!(cmd.service, "web");
        assert_eq!(cmd.subcommand(), "run");
        let args = cmd.build_args();
        assert_eq!(args, vec!["web"]);
    }

    #[test]
    fn test_compose_run_with_command() {
        let mut cmd = ComposeRunCommand::new("worker");
        cmd.command = vec!["python".to_string(), "script.py".to_string()];

        let args = cmd.build_args();
        assert_eq!(args, vec!["worker", "python", "script.py"]);
    }

    #[test]
    fn test_compose_run_with_config() {
        let config = ComposeConfig::new()
            .file("docker-compose.yml")
            .project_name("myproject");

        let mut cmd = ComposeRunCommand::new("test");
        cmd.config = config;
        cmd.command = vec!["npm".to_string(), "test".to_string()];

        assert_eq!(cmd.config().project_name, Some("myproject".to_string()));
        assert_eq!(cmd.build_args(), vec!["test", "npm", "test"]);
    }

    #[test]
    fn test_compose_run_future_options() {
        // Test that fields exist for future implementation
        let cmd = ComposeRunCommand {
            config: ComposeConfig::new(),
            service: "app".to_string(),
            command: vec!["echo".to_string(), "hello".to_string()],
            detach: true,
            rm: true,
        };

        // Currently only includes service and command
        let args = cmd.build_args();
        assert_eq!(args, vec!["app", "echo", "hello"]);

        // When fully implemented, it should include options:
        // assert!(args.contains(&"--detach".to_string()));
        // assert!(args.contains(&"--rm".to_string()));
    }
}
