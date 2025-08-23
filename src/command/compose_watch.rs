//! Docker Compose watch command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose watch command builder
#[derive(Debug, Clone)]
pub struct ComposeWatchCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to watch for changes (empty for all)
    pub services: Vec<String>,
    /// Don't build images before starting
    pub no_up: bool,
    /// Quiet mode
    pub quiet: bool,
}

/// Result from compose watch command
#[derive(Debug, Clone)]
pub struct ComposeWatchResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were watched
    pub services: Vec<String>,
}

impl ComposeWatchCommand {
    /// Create a new compose watch command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            no_up: false,
            quiet: false,
        }
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

    /// Don't build images before starting
    #[must_use]
    pub fn no_up(mut self) -> Self {
        self.no_up = true;
        self
    }

    /// Enable quiet mode
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }
}

impl Default for ComposeWatchCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeWatchCommand {
    type Output = ComposeWatchResult;

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

        Ok(ComposeWatchResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeWatchCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "watch"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.no_up {
            args.push("--no-up".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args.extend(self.services.clone());
        args
    }
}

impl ComposeWatchResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were watched
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_watch_basic() {
        let cmd = ComposeWatchCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"watch".to_string()));
    }

    #[test]
    fn test_compose_watch_with_options() {
        let cmd = ComposeWatchCommand::new()
            .no_up()
            .quiet()
            .services(vec!["frontend", "api"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--no-up".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"frontend".to_string()));
        assert!(args.contains(&"api".to_string()));
    }
}
