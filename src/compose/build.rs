//! Docker Compose build command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose build command builder
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Stub implementation
#[allow(clippy::struct_excessive_bools)] // Will be refactored when implemented
pub struct ComposeBuildCommand {
    config: ComposeConfig,
    services: Vec<String>,
    no_cache: bool,
    pull: bool,
    quiet: bool,
    build_arg: Vec<(String, String)>,
    parallel: bool,
}

impl ComposeBuildCommand {
    /// Create a new compose build command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute the build command
    ///
    /// # Errors
    ///
    /// Returns an error if the docker compose build command fails
    pub async fn run(&self) -> Result<ComposeOutput> {
        self.execute().await
    }
}

#[async_trait]
impl ComposeCommand for ComposeBuildCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
        "build"
    }

    fn build_args(&self) -> Vec<String> {
        Vec::new()
    }

    async fn execute(&self) -> Result<Self::Output> {
        execute_compose_command(&self.config, self.subcommand(), self.build_args()).await
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_build_basic() {
        let cmd = ComposeBuildCommand::new();
        assert_eq!(cmd.subcommand(), "build");
        assert_eq!(cmd.build_args(), Vec::<String>::new());
    }

    #[test]
    fn test_compose_build_with_config() {
        let config = ComposeConfig::new()
            .file("docker-compose.yml")
            .project_name("test-project");

        let cmd = ComposeBuildCommand {
            config: config.clone(),
            ..Default::default()
        };

        assert_eq!(cmd.config().project_name, Some("test-project".to_string()));
    }

    #[test]
    fn test_compose_build_future_implementation() {
        // Test that fields exist for future implementation
        let cmd = ComposeBuildCommand {
            config: ComposeConfig::new(),
            services: vec!["web".to_string(), "db".to_string()],
            no_cache: true,
            pull: true,
            quiet: false,
            build_arg: vec![("VERSION".to_string(), "1.0".to_string())],
            parallel: true,
        };

        // Currently returns empty args as it's a stub
        assert_eq!(cmd.build_args(), Vec::<String>::new());

        // When fully implemented, it should build proper args
        // Future test would verify:
        // assert!(args.contains(&"--no-cache".to_string()));
        // assert!(args.contains(&"--pull".to_string()));
        // assert!(args.contains(&"web".to_string()));
        // assert!(args.contains(&"db".to_string()));
    }
}
