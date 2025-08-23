//! Docker Compose top command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose top command
///
/// Display running processes of a service.
#[derive(Debug, Clone, Default)]
pub struct ComposeTopCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Services to show processes for
    pub services: Vec<String>,
}

/// Result from top command
#[derive(Debug, Clone)]
pub struct TopResult {
    /// Output from the command (process list)
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeTopCommand {
    /// Create a new top command
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

    /// Add a service to show processes for
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

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["top".to_string()];

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeTopCommand {
    type Output = TopResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(TopResult {
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
    fn test_top_command_basic() {
        let cmd = ComposeTopCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "top");
    }

    #[test]
    fn test_top_command_with_services() {
        let cmd = ComposeTopCommand::new().service("web").service("worker");
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"worker".to_string()));
    }
}
