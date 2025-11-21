//! Docker Compose push command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;

/// Docker Compose push command builder.
#[derive(Debug, Clone)]
pub struct ComposePushCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Services to push images for (empty for all).
    pub services: Vec<String>,
    /// Ignore build failures.
    pub ignore_build_failures: bool,
    /// Include dependencies.
    pub include_deps: bool,
    /// Quiet mode.
    pub quiet: bool,
}

/// Result from compose push command.
#[derive(Debug, Clone)]
pub struct ComposePushResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Services that were pushed.
    pub services: Vec<String>,
}

impl ComposePushCommand {
    /// Creates a new compose push command.
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            ignore_build_failures: false,
            include_deps: false,
            quiet: false,
        }
    }

    /// Adds a service to push.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to push.
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Ignores build failures.
    #[must_use]
    pub fn ignore_build_failures(mut self) -> Self {
        self.ignore_build_failures = true;
        self
    }

    /// Includes dependencies.
    #[must_use]
    pub fn include_deps(mut self) -> Self {
        self.include_deps = true;
        self
    }

    /// Enables quiet mode.
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }
}

impl Default for ComposePushCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposePushCommand {
    type Output = ComposePushResult;

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

        Ok(ComposePushResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposePushCommand {
    fn subcommand_name() -> &'static str {
        "push"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.ignore_build_failures {
            args.push("--ignore-build-failures".to_string());
        }

        if self.include_deps {
            args.push("--include-deps".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args.extend(self.services.clone());
        args
    }
}

impl ComposePushResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the services that were pushed.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_push_basic() {
        let cmd = ComposePushCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"push".to_string()));
    }

    #[test]
    fn test_compose_push_with_options() {
        let cmd = ComposePushCommand::new()
            .ignore_build_failures()
            .include_deps()
            .quiet()
            .service("web");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--ignore-build-failures".to_string()));
        assert!(args.contains(&"--include-deps".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}
