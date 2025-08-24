//! Docker context ls command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

/// Information about a Docker context
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContextInfo {
    /// Context name
    pub name: String,

    /// Description of the context
    #[serde(default)]
    pub description: String,

    /// Docker endpoint
    #[serde(rename = "DockerEndpoint")]
    pub docker_endpoint: String,

    /// Kubernetes endpoint (if configured)
    #[serde(rename = "KubernetesEndpoint", default)]
    pub kubernetes_endpoint: String,

    /// Whether this is the current context
    #[serde(default)]
    pub current: bool,
}

/// Docker context ls command builder
///
/// Lists all Docker contexts.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::command::context::ContextLsCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let contexts = ContextLsCommand::new()
///     .quiet()
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ContextLsCommand {
    /// Format the output
    format: Option<String>,
    /// Only display context names
    quiet: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ContextLsCommand {
    /// Create a new context ls command
    #[must_use]
    pub fn new() -> Self {
        Self {
            format: None,
            quiet: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Format the output using a Go template
    #[must_use]
    pub fn format(mut self, template: impl Into<String>) -> Self {
        self.format = Some(template.into());
        self
    }

    /// Format output as JSON
    #[must_use]
    pub fn format_json(self) -> Self {
        self.format("json")
    }

    /// Only display context names
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }
}

impl Default for ContextLsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ContextLsCommand {
    type Output = CommandOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["context".to_string(), "ls".to_string()];

        if let Some(format) = &self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        self.execute_command(args).await
    }
}

/// Extension methods for `ContextLsCommand` output
impl CommandOutput {
    /// Parse contexts from JSON output
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON parsing fails
    pub fn parse_contexts(&self) -> Result<Vec<ContextInfo>> {
        if self.stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let contexts: Vec<ContextInfo> = serde_json::from_str(&self.stdout)?;
        Ok(contexts)
    }

    /// Get the current context name
    #[must_use]
    pub fn current_context(&self) -> Option<String> {
        // Look for the line with an asterisk indicating current context
        for line in self.stdout.lines() {
            if line.contains('*') {
                // Extract context name (usually second column after asterisk)
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    if name == &"*" {
                        if let Some(actual_name) = parts.get(1) {
                            return Some((*actual_name).to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_ls_basic() {
        let cmd = ContextLsCommand::new();
        let args = cmd.build_command_args();
        assert_eq!(args[0], "context");
        assert_eq!(args[1], "ls");
    }

    #[test]
    fn test_context_ls_with_format() {
        let cmd = ContextLsCommand::new().format("{{.Name}}");
        let args = cmd.build_command_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"{{.Name}}".to_string()));
    }

    #[test]
    fn test_context_ls_quiet() {
        let cmd = ContextLsCommand::new().quiet();
        let args = cmd.build_command_args();
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_current_context_parsing() {
        let output = CommandOutput {
            stdout: "default          Default local daemon                          \n* production *   Production environment  unix:///var/run/docker.sock".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        assert_eq!(output.current_context(), Some("production".to_string()));
    }
}
