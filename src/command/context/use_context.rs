//! Docker context use command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker context use command builder
///
/// Switch to a different Docker context.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::{ContextUseCommand, DockerCommand};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Switch to production context
/// ContextUseCommand::new("production")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ContextUseCommand {
    /// Context name to switch to
    name: String,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ContextUseCommand {
    /// Create a new context use command
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            executor: CommandExecutor::new(),
        }
    }
}

#[async_trait]
impl DockerCommand for ContextUseCommand {
    type Output = CommandOutput;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["context".to_string(), "use".to_string(), self.name.clone()];

        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        self.execute_command(args).await
    }
}

/// Extension methods for `ContextUseCommand` output
impl CommandOutput {
    /// Check if context switch was successful
    #[must_use]
    pub fn context_switched(&self) -> bool {
        self.exit_code == 0 && self.stderr.is_empty()
    }

    /// Get the name of the context that was switched to
    pub fn switched_to_context(&self) -> Option<String> {
        if self.context_switched() {
            // The output typically contains the context name
            self.stdout.split_whitespace().last().map(String::from)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_use_basic() {
        let cmd = ContextUseCommand::new("production");
        let args = cmd.build_command_args();
        assert_eq!(args[0], "context");
        assert_eq!(args[1], "use");
        assert_eq!(args[2], "production");
    }

    #[test]
    fn test_context_switched() {
        let output = CommandOutput {
            stdout: "Current context is now \"production\"".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        assert!(output.context_switched());
        assert_eq!(
            output.switched_to_context(),
            Some("\"production\"".to_string())
        );
    }
}
