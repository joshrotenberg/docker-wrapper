//! Docker Compose rm command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;

/// Docker Compose rm command builder.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // multiple boolean flags are appropriate for rm command
pub struct ComposeRmCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Services to remove (empty for all).
    pub services: Vec<String>,
    /// Forces removal without confirmation.
    pub force: bool,
    /// Stops containers if running.
    pub stop: bool,
    /// Removes volumes associated with containers.
    pub volumes: bool,
}

/// Result from [`ComposeRmCommand`].
#[derive(Debug, Clone)]
pub struct ComposeRmResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Services that were removed.
    pub services: Vec<String>,
    /// Whether volumes were removed.
    pub volumes_removed: bool,
}

impl ComposeRmCommand {
    /// Creates a new [`ComposeRmCommand`].
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

    /// Adds a service to remove.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to remove.
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Forces removal without confirmation.
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Stops containers if running.
    #[must_use]
    pub fn stop(mut self) -> Self {
        self.stop = true;
        self
    }

    /// Removes volumes associated with containers.
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
    fn subcommand_name() -> &'static str {
        "rm"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
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

        // add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeRmResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the services that were removed.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }

    /// Checks if volumes were removed.
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
