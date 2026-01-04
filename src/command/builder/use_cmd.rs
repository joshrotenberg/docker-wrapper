//! Docker buildx use command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of buildx use command
#[derive(Debug, Clone)]
pub struct BuildxUseResult {
    /// The name of the builder that was set
    pub name: String,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl BuildxUseResult {
    /// Parse the buildx use output
    fn parse(name: &str, output: &CommandOutput) -> Self {
        Self {
            name: name.to_string(),
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker buildx use command builder
///
/// Sets the current builder instance.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, BuildxUseCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = BuildxUseCommand::new("mybuilder")
///     .default()
///     .execute()
///     .await?;
///
/// println!("Now using builder: {}", result.name);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuildxUseCommand {
    /// The builder name to use
    name: String,
    /// Set builder as default for current context
    default: bool,
    /// Builder persists context changes
    global: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl BuildxUseCommand {
    /// Create a new buildx use command
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the builder to use
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: false,
            global: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Set builder as default for current context
    #[must_use]
    pub fn default(mut self) -> Self {
        self.default = true;
        self
    }

    /// Builder persists context changes
    #[must_use]
    pub fn global(mut self) -> Self {
        self.global = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["buildx".to_string(), "use".to_string()];

        if self.default {
            args.push("--default".to_string());
        }

        if self.global {
            args.push("--global".to_string());
        }

        args.push(self.name.clone());

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for BuildxUseCommand {
    type Output = BuildxUseResult;

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
        Ok(BuildxUseResult::parse(&self.name, &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buildx_use_basic() {
        let cmd = BuildxUseCommand::new("mybuilder");
        let args = cmd.build_args();
        assert_eq!(args, vec!["buildx", "use", "mybuilder"]);
    }

    #[test]
    fn test_buildx_use_with_default() {
        let cmd = BuildxUseCommand::new("mybuilder").default();
        let args = cmd.build_args();
        assert!(args.contains(&"--default".to_string()));
    }

    #[test]
    fn test_buildx_use_with_global() {
        let cmd = BuildxUseCommand::new("mybuilder").global();
        let args = cmd.build_args();
        assert!(args.contains(&"--global".to_string()));
    }

    #[test]
    fn test_buildx_use_all_options() {
        let cmd = BuildxUseCommand::new("mybuilder").default().global();
        let args = cmd.build_args();
        assert!(args.contains(&"--default".to_string()));
        assert!(args.contains(&"--global".to_string()));
        assert!(args.contains(&"mybuilder".to_string()));
    }
}
