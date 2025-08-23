//! Docker Compose watch command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose watch command
///
/// Watch build context for changes and rebuild/restart services automatically.
#[derive(Debug, Clone, Default)]
pub struct ComposeWatchCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Don't build images
    pub no_up: bool,
    /// Services to watch
    pub services: Vec<String>,
}

/// Result from watch command
#[derive(Debug, Clone)]
pub struct WatchResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeWatchCommand {
    /// Create a new watch command
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

    /// Don't start services before watching
    #[must_use]
    pub fn no_up(mut self) -> Self {
        self.no_up = true;
        self
    }

    /// Add a service to watch
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to watch
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
        let mut args = vec!["watch".to_string()];

        // Add flags
        if self.no_up {
            args.push("--no-up".to_string());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeWatchCommand {
    type Output = WatchResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(WatchResult {
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
    fn test_watch_command_basic() {
        let cmd = ComposeWatchCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "watch");
    }

    #[test]
    fn test_watch_command_with_no_up() {
        let cmd = ComposeWatchCommand::new().no_up();
        let args = cmd.build_args();
        assert!(args.contains(&"--no-up".to_string()));
    }

    #[test]
    fn test_watch_command_with_services() {
        let cmd = ComposeWatchCommand::new().service("web").service("worker");
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"worker".to_string()));
    }
}
