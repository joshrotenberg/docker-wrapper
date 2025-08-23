//! Docker Compose wait command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose wait command
///
/// Wait for services to reach a desired state.
#[derive(Debug, Clone, Default)]
pub struct ComposeWaitCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Services to wait for
    pub services: Vec<String>,
    /// Wait for services to be running and healthy
    pub down_project: bool,
}

/// Result from wait command
#[derive(Debug, Clone)]
pub struct WaitResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Exit codes from services
    pub exit_codes: Vec<i32>,
}

impl ComposeWaitCommand {
    /// Create a new wait command
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

    /// Wait for the entire project to stop
    #[must_use]
    pub fn down_project(mut self) -> Self {
        self.down_project = true;
        self
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

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["wait".to_string()];

        // Add flags
        if self.down_project {
            args.push("--down-project".to_string());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeWaitCommand {
    type Output = WaitResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        // Parse exit codes from output if available
        let exit_codes = output
            .stdout
            .lines()
            .filter_map(|line| {
                // Try to parse exit codes from output
                line.split_whitespace()
                    .last()
                    .and_then(|s| s.parse::<i32>().ok())
            })
            .collect();

        Ok(WaitResult {
            output: output.stdout,
            success: output.success,
            exit_codes,
        })
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.execute_compose(args).await
    }
}

impl WaitResult {
    /// Check if all services exited successfully (code 0)
    #[must_use]
    pub fn all_successful(&self) -> bool {
        !self.exit_codes.is_empty() && self.exit_codes.iter().all(|&code| code == 0)
    }

    /// Get the first non-zero exit code
    #[must_use]
    pub fn first_failure(&self) -> Option<i32> {
        self.exit_codes.iter().find(|&&code| code != 0).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_command_basic() {
        let cmd = ComposeWaitCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "wait");
    }

    #[test]
    fn test_wait_command_with_services() {
        let cmd = ComposeWaitCommand::new().service("web").service("db");
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_wait_command_with_down_project() {
        let cmd = ComposeWaitCommand::new().down_project();
        let args = cmd.build_args();
        assert!(args.contains(&"--down-project".to_string()));
    }

    #[test]
    fn test_wait_result_helpers() {
        let result = WaitResult {
            output: String::new(),
            success: true,
            exit_codes: vec![0, 0, 0],
        };
        assert!(result.all_successful());
        assert_eq!(result.first_failure(), None);

        let result_with_failure = WaitResult {
            output: String::new(),
            success: false,
            exit_codes: vec![0, 1, 0],
        };
        assert!(!result_with_failure.all_successful());
        assert_eq!(result_with_failure.first_failure(), Some(1));
    }
}
