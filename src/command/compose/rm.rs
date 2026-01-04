//! Docker Compose rm command implementation using unified trait pattern.

use crate::command::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose rm command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are appropriate for rm command
pub struct ComposeRmCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to remove (empty for all)
    pub services: Vec<String>,
    /// Force removal without confirmation
    pub force: bool,
    /// Stop containers if running
    pub stop: bool,
    /// Remove volumes associated with containers
    pub volumes: bool,
}

/// Result from compose rm command
#[derive(Debug, Clone)]
pub struct ComposeRmResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were removed
    pub services: Vec<String>,
    /// Whether volumes were removed
    pub volumes_removed: bool,
}

impl ComposeRmCommand {
    /// Create a new compose rm command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            force: false,
            stop: false,
            volumes: false,
        }
    }

    /// Add a service to remove
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to remove
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Force removal without confirmation
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Stop containers if running
    #[must_use]
    pub fn stop(mut self) -> Self {
        self.stop = true;
        self
    }

    /// Remove volumes associated with containers
    #[must_use]
    pub fn volumes(mut self) -> Self {
        self.volumes = true;
        self
    }
}

impl Default for ComposeRmCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeRmCommand {
    type Output = ComposeRmResult;

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

        Ok(ComposeRmResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
            volumes_removed: self.volumes,
        })
    }
}

impl ComposeCommand for ComposeRmCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "rm"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.force {
            args.push("--force".to_string());
        }

        if self.stop {
            args.push("--stop".to_string());
        }

        if self.volumes {
            args.push("--volumes".to_string());
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeRmResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were removed
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }

    /// Check if volumes were removed
    #[must_use]
    pub fn volumes_removed(&self) -> bool {
        self.volumes_removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_rm_basic() {
        let cmd = ComposeRmCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"rm".to_string()));
    }

    #[test]
    fn test_compose_rm_with_services() {
        let cmd = ComposeRmCommand::new().service("web").service("worker");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "worker"]);
    }

    #[test]
    fn test_compose_rm_with_flags() {
        let cmd = ComposeRmCommand::new()
            .force()
            .stop()
            .volumes()
            .service("app");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--stop".to_string()));
        assert!(args.contains(&"--volumes".to_string()));
        assert!(args.contains(&"app".to_string()));
    }

    #[test]
    fn test_compose_rm_force_only() {
        let cmd = ComposeRmCommand::new().force().service("database");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--force", "database"]);
    }

    #[test]
    fn test_compose_rm_with_volumes() {
        let cmd = ComposeRmCommand::new()
            .volumes()
            .stop()
            .services(vec!["cache", "queue"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--volumes".to_string()));
        assert!(args.contains(&"--stop".to_string()));
        assert!(args.contains(&"cache".to_string()));
        assert!(args.contains(&"queue".to_string()));
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeRmCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .force()
            .volumes()
            .service("web");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--volumes".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}
