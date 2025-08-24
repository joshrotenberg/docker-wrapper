//! Docker context inspect command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Docker context inspect command builder
///
/// Display detailed information on one or more contexts.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::command::context::ContextInspectCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Inspect a context
/// let info = ContextInspectCommand::new("production")
///     .format("{{.Endpoints.docker.Host}}")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ContextInspectCommand {
    /// Contexts to inspect
    contexts: Vec<String>,
    /// Format the output
    format: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ContextInspectCommand {
    /// Create a new context inspect command
    #[must_use]
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            contexts: vec![context.into()],
            format: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another context to inspect
    #[must_use]
    pub fn add_context(mut self, context: impl Into<String>) -> Self {
        self.contexts.push(context.into());
        self
    }

    /// Format the output using a Go template
    #[must_use]
    pub fn format(mut self, template: impl Into<String>) -> Self {
        self.format = Some(template.into());
        self
    }
}

#[async_trait]
impl DockerCommand for ContextInspectCommand {
    type Output = CommandOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["context".to_string(), "inspect".to_string()];

        if let Some(format) = &self.format {
            args.push("--format".to_string());
            args.push(format.clone());
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

/// Extension methods for `ContextInspectCommand` output
impl CommandOutput {
    /// Parse context information as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON parsing fails
    pub fn parse_context_info(&self) -> Result<Vec<Value>> {
        if self.stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Docker returns array of contexts
        let contexts: Vec<Value> = serde_json::from_str(&self.stdout)?;
        Ok(contexts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_inspect_single() {
        let cmd = ContextInspectCommand::new("production");
        let args = cmd.build_command_args();
        assert_eq!(args[0], "context");
        assert_eq!(args[1], "inspect");
        assert!(args.contains(&"production".to_string()));
    }

    #[test]
    fn test_context_inspect_multiple() {
        let cmd = ContextInspectCommand::new("context1").add_context("context2");
        let args = cmd.build_command_args();
        assert!(args.contains(&"context1".to_string()));
        assert!(args.contains(&"context2".to_string()));
    }

    #[test]
    fn test_context_inspect_with_format() {
        let cmd = ContextInspectCommand::new("production").format("{{.Name}}");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"{{.Name}}".to_string()));
    }
}
