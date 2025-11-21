//! Docker Compose create command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose create command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are appropriate for create command
pub struct ComposeCreateCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Build images before creating containers
    pub build: bool,
    /// Don't build images, even if missing
    pub no_build: bool,
    /// Force recreate containers
    pub force_recreate: bool,
    /// Don't recreate containers if they exist
    pub no_recreate: bool,
    /// Pull images before creating
    pub pull: Option<PullPolicy>,
    /// Remove orphaned containers
    pub remove_orphans: bool,
    /// Services to create (empty for all)
    pub services: Vec<String>,
}

/// Pull policy for images
#[derive(Debug, Clone, Copy)]
pub enum PullPolicy {
    /// Always pull images
    Always,
    /// Never pull images
    Never,
    /// Pull missing images (default)
    Missing,
    /// Pull images if local is older
    Build,
}

impl std::fmt::Display for PullPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
            Self::Missing => write!(f, "missing"),
            Self::Build => write!(f, "build"),
        }
    }
}

/// Result from compose create command
#[derive(Debug, Clone)]
pub struct ComposeCreateResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were created
    pub services: Vec<String>,
}

impl ComposeCreateCommand {
    /// Create a new compose create command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            build: false,
            no_build: false,
            force_recreate: false,
            no_recreate: false,
            pull: None,
            remove_orphans: false,
            services: Vec::new(),
        }
    }

    /// Build images before creating containers
    #[must_use]
    pub fn build(mut self) -> Self {
        self.build = true;
        self
    }

    /// Don't build images, even if missing
    #[must_use]
    pub fn no_build(mut self) -> Self {
        self.no_build = true;
        self
    }

    /// Force recreate containers
    #[must_use]
    pub fn force_recreate(mut self) -> Self {
        self.force_recreate = true;
        self
    }

    /// Don't recreate containers if they exist
    #[must_use]
    pub fn no_recreate(mut self) -> Self {
        self.no_recreate = true;
        self
    }

    /// Set pull policy
    #[must_use]
    pub fn pull(mut self, policy: PullPolicy) -> Self {
        self.pull = Some(policy);
        self
    }

    /// Remove orphaned containers
    #[must_use]
    pub fn remove_orphans(mut self) -> Self {
        self.remove_orphans = true;
        self
    }

    /// Add a service to create
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to create
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

impl Default for ComposeCreateCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeCreateCommand {
    type Output = ComposeCreateResult;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        // Use the ComposeCommand implementation explicitly
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposeCreateResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeCreateCommand {
    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "create"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.build {
            args.push("--build".to_string());
        }

        if self.no_build {
            args.push("--no-build".to_string());
        }

        if self.force_recreate {
            args.push("--force-recreate".to_string());
        }

        if self.no_recreate {
            args.push("--no-recreate".to_string());
        }

        if self.remove_orphans {
            args.push("--remove-orphans".to_string());
        }

        // Add pull policy
        if let Some(pull) = self.pull {
            args.push("--pull".to_string());
            args.push(pull.to_string());
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeCreateResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were created
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_create_basic() {
        let cmd = ComposeCreateCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"create".to_string()));
    }

    #[test]
    fn test_compose_create_with_build() {
        let cmd = ComposeCreateCommand::new().build().force_recreate();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--build".to_string()));
        assert!(args.contains(&"--force-recreate".to_string()));
    }

    #[test]
    fn test_compose_create_with_pull() {
        let cmd = ComposeCreateCommand::new()
            .pull(PullPolicy::Always)
            .no_recreate();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--pull".to_string()));
        assert!(args.contains(&"always".to_string()));
        assert!(args.contains(&"--no-recreate".to_string()));
    }

    #[test]
    fn test_compose_create_with_services() {
        let cmd = ComposeCreateCommand::new()
            .service("web")
            .service("db")
            .remove_orphans();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
        assert!(args.contains(&"--remove-orphans".to_string()));
    }

    #[test]
    fn test_pull_policy_display() {
        assert_eq!(PullPolicy::Always.to_string(), "always");
        assert_eq!(PullPolicy::Never.to_string(), "never");
        assert_eq!(PullPolicy::Missing.to_string(), "missing");
        assert_eq!(PullPolicy::Build.to_string(), "build");
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeCreateCommand::new()
            .file("docker-compose.yml")
            .project_name("my-project")
            .build()
            .service("web");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--build".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}
