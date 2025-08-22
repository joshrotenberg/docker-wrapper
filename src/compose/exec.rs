//! Docker Compose exec command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose exec command builder
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Stub implementation
pub struct ComposeExecCommand {
    config: ComposeConfig,
    service: String,
    command: Vec<String>,
    detach: bool,
    user: Option<String>,
    workdir: Option<String>,
}

impl ComposeExecCommand {
    /// Create a new compose exec command
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            ..Self::default()
        }
    }

    /// Execute the exec command
    ///
    /// # Errors
    ///
    /// Returns an error if the docker compose exec command fails
    pub async fn run(&self) -> Result<ComposeOutput> {
        self.execute().await
    }
}

#[async_trait]
impl ComposeCommand for ComposeExecCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
        "exec"
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
    fn test_compose_exec_basic() {
        let cmd = ComposeExecCommand::new("web");
        assert_eq!(cmd.service, "web");
        assert_eq!(cmd.subcommand(), "exec");
        let args = cmd.build_args();
        assert_eq!(args, vec!["web"]);
    }

    #[test]
    fn test_compose_exec_with_command() {
        let mut cmd = ComposeExecCommand::new("db");
        cmd.command = vec!["psql".to_string(), "-U".to_string(), "postgres".to_string()];

        let args = cmd.build_args();
        assert_eq!(args, vec!["db", "psql", "-U", "postgres"]);
    }

    #[test]
    fn test_compose_exec_with_config() {
        let config = ComposeConfig::new()
            .file("docker-compose.yml")
            .project_name("myapp");

        let mut cmd = ComposeExecCommand::new("web");
        cmd.config = config;
        cmd.command = vec!["bash".to_string()];

        assert_eq!(cmd.config().project_name, Some("myapp".to_string()));
        assert_eq!(cmd.build_args(), vec!["web", "bash"]);
    }

    #[test]
    fn test_compose_exec_future_options() {
        // Test that fields exist for future implementation
        let cmd = ComposeExecCommand {
            config: ComposeConfig::new(),
            service: "app".to_string(),
            command: vec!["sh".to_string()],
            detach: true,
            user: Some("root".to_string()),
            workdir: Some("/app".to_string()),
        };

        // Currently only includes service and command
        let args = cmd.build_args();
        assert_eq!(args, vec!["app", "sh"]);

        // When fully implemented, it should include options:
        // assert!(args.contains(&"--detach".to_string()));
        // assert!(args.contains(&"--user".to_string()));
        // assert!(args.contains(&"root".to_string()));
        // assert!(args.contains(&"--workdir".to_string()));
        // assert!(args.contains(&"/app".to_string()));
    }
}
