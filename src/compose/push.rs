//! Docker Compose push command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose push command
///
/// Push service images to registry.
#[derive(Debug, Clone, Default)]
pub struct ComposePushCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Include all tags when pushing
    pub include_deps: bool,
    /// Ignore push failures
    pub ignore_push_failures: bool,
    /// Don't print progress
    pub quiet: bool,
    /// Services to push
    pub services: Vec<String>,
}

/// Result from push command
#[derive(Debug, Clone)]
pub struct PushResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposePushCommand {
    /// Create a new push command
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

    /// Include all dependencies
    #[must_use]
    pub fn include_deps(mut self) -> Self {
        self.include_deps = true;
        self
    }

    /// Ignore push failures
    #[must_use]
    pub fn ignore_push_failures(mut self) -> Self {
        self.ignore_push_failures = true;
        self
    }

    /// Quiet mode
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Add a service to push
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to push
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
        let mut args = vec!["push".to_string()];

        // Add flags
        if self.include_deps {
            args.push("--include-deps".to_string());
        }
        if self.ignore_push_failures {
            args.push("--ignore-push-failures".to_string());
        }
        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposePushCommand {
    type Output = PushResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(PushResult {
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
    fn test_push_command_basic() {
        let cmd = ComposePushCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "push");
    }

    #[test]
    fn test_push_command_with_flags() {
        let cmd = ComposePushCommand::new()
            .include_deps()
            .ignore_push_failures()
            .quiet();
        let args = cmd.build_args();
        assert!(args.contains(&"--include-deps".to_string()));
        assert!(args.contains(&"--ignore-push-failures".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_push_command_with_services() {
        let cmd = ComposePushCommand::new().service("web").service("api");
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"api".to_string()));
    }
}
