//! Docker buildx stop command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of buildx stop command
#[derive(Debug, Clone)]
pub struct BuildxStopResult {
    /// The name of the builder that was stopped
    pub name: Option<String>,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl BuildxStopResult {
    /// Parse the buildx stop output
    fn parse(name: Option<&str>, output: &CommandOutput) -> Self {
        Self {
            name: name.map(ToString::to_string),
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker buildx stop command builder
///
/// Stops a builder instance.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, BuildxStopCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = BuildxStopCommand::new()
///     .name("mybuilder")
///     .execute()
///     .await?;
///
/// if result.success {
///     println!("Builder stopped successfully");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct BuildxStopCommand {
    /// The builder name to stop (optional, defaults to current)
    name: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl BuildxStopCommand {
    /// Create a new buildx stop command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the builder name to stop
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["buildx".to_string(), "stop".to_string()];

        if let Some(ref name) = self.name {
            args.push(name.clone());
        }

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for BuildxStopCommand {
    type Output = BuildxStopResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self.execute_command(args).await?;
        Ok(BuildxStopResult::parse(self.name.as_deref(), &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buildx_stop_basic() {
        let cmd = BuildxStopCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["buildx", "stop"]);
    }

    #[test]
    fn test_buildx_stop_with_name() {
        let cmd = BuildxStopCommand::new().name("mybuilder");
        let args = cmd.build_args();
        assert!(args.contains(&"mybuilder".to_string()));
    }
}
