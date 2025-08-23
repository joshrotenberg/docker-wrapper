//! Docker Compose create command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose create command
///
/// Create services without starting them.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ComposeCreateCommand {
    /// Base configuration
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
    /// Services to create
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

impl PullPolicy {
    /// Convert to command line argument
    #[must_use]
    pub fn as_arg(&self) -> &str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::Missing => "missing",
            Self::Build => "build",
        }
    }
}

/// Result from create command
#[derive(Debug, Clone)]
pub struct CreateResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeCreateCommand {
    /// Create a new create command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a compose file
    #[must_use]
    pub fn file<P: Into<std::path::PathBuf>>(mut self, file: P) -> Self {
        self.config.files.push(file.into());
        self
    }

    /// Set project name
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config.project_name = Some(name.into());
        self
    }

    /// Build images before creating
    #[must_use]
    pub fn build(mut self) -> Self {
        self.build = true;
        self
    }

    /// Don't build images
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

    /// Don't recreate containers
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

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["create".to_string()];

        // Add flags
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
        if let Some(pull) = &self.pull {
            args.push("--pull".to_string());
            args.push(pull.as_arg().to_string());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeCreateCommand {
    type Output = CreateResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(CreateResult {
            output: output.stdout,
            success: output.success,
        })
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.execute_compose(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_command_basic() {
        let cmd = ComposeCreateCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "create");
    }

    #[test]
    fn test_create_command_with_build() {
        let cmd = ComposeCreateCommand::new().build().force_recreate();
        let args = cmd.build_args();
        assert!(args.contains(&"--build".to_string()));
        assert!(args.contains(&"--force-recreate".to_string()));
    }

    #[test]
    fn test_create_command_with_pull() {
        let cmd = ComposeCreateCommand::new()
            .pull(PullPolicy::Always)
            .no_recreate();
        let args = cmd.build_args();
        assert!(args.contains(&"--pull".to_string()));
        assert!(args.contains(&"always".to_string()));
        assert!(args.contains(&"--no-recreate".to_string()));
    }

    #[test]
    fn test_create_command_with_services() {
        let cmd = ComposeCreateCommand::new()
            .service("web")
            .service("db")
            .remove_orphans();
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
        assert!(args.contains(&"--remove-orphans".to_string()));
    }
}
