//! Docker Compose kill command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose kill command
///
/// Force stop service containers.
#[derive(Debug, Clone, Default)]
pub struct ComposeKillCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Signal to send (default: SIGKILL)
    pub signal: Option<String>,
    /// Remove containers after killing
    pub remove_orphans: bool,
    /// Services to kill
    pub services: Vec<String>,
}

/// Result from kill command
#[derive(Debug, Clone)]
pub struct KillResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeKillCommand {
    /// Create a new kill command
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

    /// Set signal to send
    #[must_use]
    pub fn signal(mut self, signal: impl Into<String>) -> Self {
        self.signal = Some(signal.into());
        self
    }

    /// Remove orphaned containers
    #[must_use]
    pub fn remove_orphans(mut self) -> Self {
        self.remove_orphans = true;
        self
    }

    /// Add a service to kill
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to kill
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
        let mut args = vec!["kill".to_string()];

        // Add signal
        if let Some(signal) = &self.signal {
            args.push("--signal".to_string());
            args.push(signal.clone());
        }

        // Add flags
        if self.remove_orphans {
            args.push("--remove-orphans".to_string());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeKillCommand {
    type Output = KillResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(KillResult {
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
    fn test_kill_command_basic() {
        let cmd = ComposeKillCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "kill");
    }

    #[test]
    fn test_kill_command_with_signal() {
        let cmd = ComposeKillCommand::new().signal("SIGTERM");
        let args = cmd.build_args();
        assert!(args.contains(&"--signal".to_string()));
        assert!(args.contains(&"SIGTERM".to_string()));
    }

    #[test]
    fn test_kill_command_with_services() {
        let cmd = ComposeKillCommand::new()
            .service("web")
            .service("worker")
            .remove_orphans();
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"worker".to_string()));
        assert!(args.contains(&"--remove-orphans".to_string()));
    }
}
