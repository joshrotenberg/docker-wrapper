//! Docker context rm command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker context rm command builder
///
/// Remove one or more Docker contexts.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::{ContextRmCommand, DockerCommand};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Remove a context
/// ContextRmCommand::new("old-context")
///     .force()
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ContextRmCommand {
    /// Contexts to remove
    contexts: Vec<String>,
    /// Force removal
    force: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ContextRmCommand {
    /// Create a new context rm command
    #[must_use]
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            contexts: vec![context.into()],
            force: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another context to remove
    #[must_use]
    pub fn add_context(mut self, context: impl Into<String>) -> Self {
        self.contexts.push(context.into());
        self
    }

    /// Force removal (don't prompt for confirmation)
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
}

#[async_trait]
impl DockerCommand for ContextRmCommand {
    type Output = CommandOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["context".to_string(), "rm".to_string()];

        if self.force {
            args.push("--force".to_string());
        }

        args.extend(self.contexts.clone());
        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        self.execute_command(args).await
    }
}

/// Extension methods for `ContextRmCommand` output
impl CommandOutput {
    /// Get removed context names from output
    #[must_use]
    pub fn removed_contexts(&self) -> Vec<String> {
        self.stdout
            .lines()
            .filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some(line.trim().to_string())
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_rm_single() {
        let cmd = ContextRmCommand::new("test-context");
        let args = cmd.build_command_args();
        assert_eq!(args[0], "context");
        assert_eq!(args[1], "rm");
        assert!(args.contains(&"test-context".to_string()));
    }

    #[test]
    fn test_context_rm_multiple() {
        let cmd = ContextRmCommand::new("context1").add_context("context2");
        let args = cmd.build_command_args();
        assert!(args.contains(&"context1".to_string()));
        assert!(args.contains(&"context2".to_string()));
    }

    #[test]
    fn test_context_rm_force() {
        let cmd = ContextRmCommand::new("test-context").force();
        let args = cmd.build_command_args();
        assert!(args.contains(&"--force".to_string()));
    }

    #[test]
    fn test_removed_contexts_parsing() {
        let output = CommandOutput {
            stdout: "context1\ncontext2\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        let removed = output.removed_contexts();
        assert_eq!(removed.len(), 2);
        assert_eq!(removed[0], "context1");
        assert_eq!(removed[1], "context2");
    }
}
