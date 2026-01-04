//! Docker buildx ls command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Information about a builder instance
#[derive(Debug, Clone)]
pub struct BuilderInfo {
    /// The name of the builder
    pub name: String,
    /// The driver used by the builder
    pub driver: Option<String>,
    /// Whether this is the current/default builder
    pub is_default: bool,
    /// The status of the builder
    pub status: Option<String>,
}

/// Result of buildx ls command
#[derive(Debug, Clone)]
pub struct BuildxLsResult {
    /// List of builder instances
    pub builders: Vec<BuilderInfo>,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl BuildxLsResult {
    /// Parse the buildx ls output
    fn parse(output: &CommandOutput) -> Self {
        let stdout = &output.stdout;
        let mut builders = Vec::new();

        // Parse table output (skip header line)
        for line in stdout.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with(' ') {
                continue;
            }

            // Parse the line - format is typically:
            // NAME/NODE       DRIVER/ENDPOINT  STATUS   BUILDKIT PLATFORMS
            // default *       docker
            // mybuilder       docker-container running  v0.12.5  linux/amd64
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let mut name = parts[0].to_string();
            let is_default = name.ends_with('*') || parts.get(1) == Some(&"*");

            // Remove the asterisk from name if present
            if name.ends_with('*') {
                name = name.trim_end_matches('*').to_string();
            }

            let driver = if is_default && parts.len() > 2 {
                Some(parts[2].to_string())
            } else if !is_default && parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            };

            // Skip node entries (indented or contain /)
            if name.contains('/') && !name.starts_with('/') {
                continue;
            }

            builders.push(BuilderInfo {
                name,
                driver,
                is_default,
                status: None,
            });
        }

        Self {
            builders,
            output: stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker buildx ls command builder
///
/// Lists builder instances.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, BuildxLsCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = BuildxLsCommand::new()
///     .execute()
///     .await?;
///
/// for builder in &result.builders {
///     println!("Builder: {} (default: {})", builder.name, builder.is_default);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct BuildxLsCommand {
    /// Format the output
    format: Option<String>,
    /// Don't truncate output
    no_trunc: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl BuildxLsCommand {
    /// Create a new buildx ls command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the output format
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Don't truncate output
    #[must_use]
    pub fn no_trunc(mut self) -> Self {
        self.no_trunc = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["buildx".to_string(), "ls".to_string()];

        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        if self.no_trunc {
            args.push("--no-trunc".to_string());
        }

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for BuildxLsCommand {
    type Output = BuildxLsResult;

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
        Ok(BuildxLsResult::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buildx_ls_basic() {
        let cmd = BuildxLsCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["buildx", "ls"]);
    }

    #[test]
    fn test_buildx_ls_with_format() {
        let cmd = BuildxLsCommand::new().format("json");
        let args = cmd.build_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_buildx_ls_with_no_trunc() {
        let cmd = BuildxLsCommand::new().no_trunc();
        let args = cmd.build_args();
        assert!(args.contains(&"--no-trunc".to_string()));
    }

    #[test]
    fn test_buildx_ls_result_parse() {
        let output = CommandOutput {
            stdout: "NAME/NODE       DRIVER/ENDPOINT  STATUS   BUILDKIT PLATFORMS\ndefault *       docker\nmybuilder       docker-container running  v0.12.5  linux/amd64".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let result = BuildxLsResult::parse(&output);
        assert_eq!(result.builders.len(), 2);
        assert!(result.builders[0].is_default);
        assert_eq!(result.builders[0].name, "default");
        assert_eq!(result.builders[1].name, "mybuilder");
    }
}
