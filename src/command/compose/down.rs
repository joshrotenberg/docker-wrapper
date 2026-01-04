//! Docker Compose down command implementation using unified trait pattern.

use crate::command::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose down command builder
#[derive(Debug, Clone)]
pub struct ComposeDownCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Remove images
    pub remove_images: Option<RemoveImages>,
    /// Remove named volumes
    pub volumes: bool,
    /// Remove orphan containers
    pub remove_orphans: bool,
    /// Timeout for container shutdown
    pub timeout: Option<Duration>,
    /// Services to stop (empty for all)
    pub services: Vec<String>,
}

/// Image removal options for compose down
#[derive(Debug, Clone, Copy)]
pub enum RemoveImages {
    /// Remove all images used by services
    All,
    /// Remove only images that don't have a custom tag
    Local,
}

impl std::fmt::Display for RemoveImages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Local => write!(f, "local"),
        }
    }
}

/// Result from compose down command
#[derive(Debug, Clone)]
pub struct ComposeDownResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Whether volumes were removed
    pub removed_volumes: bool,
    /// Whether images were removed
    pub removed_images: bool,
}

impl ComposeDownCommand {
    /// Create a new compose down command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            remove_images: None,
            volumes: false,
            remove_orphans: false,
            timeout: None,
            services: Vec::new(),
        }
    }

    /// Remove images (all or local)
    #[must_use]
    pub fn remove_images(mut self, policy: RemoveImages) -> Self {
        self.remove_images = Some(policy);
        self
    }

    /// Remove named volumes declared in the volumes section
    #[must_use]
    pub fn volumes(mut self) -> Self {
        self.volumes = true;
        self
    }

    /// Remove containers for services not defined in the compose file
    #[must_use]
    pub fn remove_orphans(mut self) -> Self {
        self.remove_orphans = true;
        self
    }

    /// Set timeout for container shutdown
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add a service to stop
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services
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

impl Default for ComposeDownCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeDownCommand {
    type Output = ComposeDownResult;

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

        Ok(ComposeDownResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            removed_volumes: self.volumes,
            removed_images: self.remove_images.is_some(),
        })
    }
}

impl ComposeCommand for ComposeDownCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "down"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref remove) = self.remove_images {
            args.push("--rmi".to_string());
            args.push(remove.to_string());
        }

        if self.volumes {
            args.push("--volumes".to_string());
        }

        if self.remove_orphans {
            args.push("--remove-orphans".to_string());
        }

        if let Some(timeout) = self.timeout {
            args.push("--timeout".to_string());
            args.push(timeout.as_secs().to_string());
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeDownResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Check if volumes were removed
    #[must_use]
    pub fn volumes_removed(&self) -> bool {
        self.removed_volumes
    }

    /// Check if images were removed
    #[must_use]
    pub fn images_removed(&self) -> bool {
        self.removed_images
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_down_basic() {
        let cmd = ComposeDownCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"down".to_string()));
    }

    #[test]
    fn test_compose_down_with_volumes() {
        let cmd = ComposeDownCommand::new().volumes();
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--volumes"]);
    }

    #[test]
    fn test_compose_down_remove_images() {
        let cmd = ComposeDownCommand::new().remove_images(RemoveImages::All);
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--rmi", "all"]);
    }

    #[test]
    fn test_compose_down_all_options() {
        let cmd = ComposeDownCommand::new()
            .remove_images(RemoveImages::Local)
            .volumes()
            .remove_orphans()
            .timeout(Duration::from_secs(30))
            .service("web")
            .service("db");

        let args = cmd.build_subcommand_args();
        assert_eq!(
            args,
            vec![
                "--rmi",
                "local",
                "--volumes",
                "--remove-orphans",
                "--timeout",
                "30",
                "web",
                "db"
            ]
        );
    }

    #[test]
    fn test_remove_images_display() {
        assert_eq!(RemoveImages::All.to_string(), "all");
        assert_eq!(RemoveImages::Local.to_string(), "local");
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeDownCommand::new()
            .file("docker-compose.yml")
            .project_name("my-project")
            .volumes()
            .remove_orphans();

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--volumes".to_string()));
        assert!(args.contains(&"--remove-orphans".to_string()));
    }
}
