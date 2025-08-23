//! Docker Compose rm command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose rm command
///
/// Remove stopped service containers.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ComposeRmCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Force removal without confirmation
    pub force: bool,
    /// Stop containers if running
    pub stop: bool,
    /// Remove volumes
    pub volumes: bool,
    /// Remove all containers (not just stopped)
    pub all: bool,
    /// Services to remove
    pub services: Vec<String>,
}

/// Result from rm command
#[derive(Debug, Clone)]
pub struct RmResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeRmCommand {
    /// Create a new rm command
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

    /// Remove volumes
    #[must_use]
    pub fn volumes(mut self) -> Self {
        self.volumes = true;
        self
    }

    /// Remove all containers
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
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

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["rm".to_string()];

        // Add flags
        if self.force {
            args.push("--force".to_string());
        }
        if self.stop {
            args.push("--stop".to_string());
        }
        if self.volumes {
            args.push("--volumes".to_string());
        }
        if self.all {
            args.push("--all".to_string());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeRmCommand {
    type Output = RmResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(RmResult {
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
    fn test_rm_command_basic() {
        let cmd = ComposeRmCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "rm");
    }

    #[test]
    fn test_rm_command_with_force() {
        let cmd = ComposeRmCommand::new().force().stop();
        let args = cmd.build_args();
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--stop".to_string()));
    }

    #[test]
    fn test_rm_command_with_services() {
        let cmd = ComposeRmCommand::new().service("web").service("db");
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_rm_command_with_volumes() {
        let cmd = ComposeRmCommand::new().volumes().all();
        let args = cmd.build_args();
        assert!(args.contains(&"--volumes".to_string()));
        assert!(args.contains(&"--all".to_string()));
    }
}
