//! Docker Compose wait command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose wait command builder
#[derive(Debug, Clone)]
pub struct ComposeWaitCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to wait for (empty for all)
    pub services: Vec<String>,
    /// Timeout for waiting
    pub timeout: Option<Duration>,
    /// Wait for services to be healthy (with health checks)
    pub wait_for_healthy: bool,
}

/// Result from compose wait command
#[derive(Debug, Clone)]
pub struct ComposeWaitResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were waited for
    pub services: Vec<String>,
    /// Whether all services became ready/healthy
    pub all_ready: bool,
}

impl ComposeWaitCommand {
    /// Create a new compose wait command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            timeout: None,
            wait_for_healthy: false,
        }
    }

    /// Add a service to wait for
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to wait for
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Set timeout for waiting
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Wait for services to be healthy (requires health checks)
    #[must_use]
    pub fn wait_for_healthy(mut self) -> Self {
        self.wait_for_healthy = true;
        self
    }
}

impl Default for ComposeWaitCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommandV2 for ComposeWaitCommand {
    type Output = ComposeWaitResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposeWaitResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
            all_ready: output.success,
        })
    }
}

impl ComposeCommand for ComposeWaitCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "wait"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(timeout) = self.timeout {
            args.push("--timeout".to_string());
            args.push(timeout.as_secs().to_string());
        }

        if self.wait_for_healthy {
            args.push("--wait-for-healthy".to_string());
        }

        args.extend(self.services.clone());
        args
    }
}

impl ComposeWaitResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were waited for
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }

    /// Check if all services became ready/healthy
    #[must_use]
    pub fn all_ready(&self) -> bool {
        self.all_ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_wait_basic() {
        let cmd = ComposeWaitCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"wait".to_string()));
    }

    #[test]
    fn test_compose_wait_with_options() {
        let cmd = ComposeWaitCommand::new()
            .timeout(Duration::from_secs(30))
            .wait_for_healthy()
            .services(vec!["web", "db"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--timeout".to_string()));
        assert!(args.contains(&"30".to_string()));
        assert!(args.contains(&"--wait-for-healthy".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }
}
