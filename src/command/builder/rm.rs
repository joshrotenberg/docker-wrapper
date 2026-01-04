//! Docker buildx rm command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of buildx rm command
#[derive(Debug, Clone)]
pub struct BuildxRmResult {
    /// The names of the builders that were removed
    pub names: Vec<String>,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl BuildxRmResult {
    /// Parse the buildx rm output
    fn parse(names: &[String], output: &CommandOutput) -> Self {
        Self {
            names: names.to_vec(),
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker buildx rm command builder
///
/// Removes one or more builder instances.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, BuildxRmCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = BuildxRmCommand::new("mybuilder")
///     .force()
///     .execute()
///     .await?;
///
/// println!("Removed {} builders", result.names.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BuildxRmCommand {
    /// The builder names to remove
    names: Vec<String>,
    /// Remove all inactive builders
    all_inactive: bool,
    /// Do not prompt for confirmation
    force: bool,
    /// Keep the `BuildKit` daemon running
    keep_daemon: bool,
    /// Keep `BuildKit` state
    keep_state: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl BuildxRmCommand {
    /// Create a new buildx rm command
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the builder to remove
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            names: vec![name.into()],
            all_inactive: false,
            force: false,
            keep_daemon: false,
            keep_state: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new buildx rm command to remove all inactive builders
    #[must_use]
    pub fn all_inactive() -> Self {
        Self {
            names: Vec::new(),
            all_inactive: true,
            force: false,
            keep_daemon: false,
            keep_state: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another builder to remove
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.names.push(name.into());
        self
    }

    /// Do not prompt for confirmation
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Keep the `BuildKit` daemon running
    #[must_use]
    pub fn keep_daemon(mut self) -> Self {
        self.keep_daemon = true;
        self
    }

    /// Keep `BuildKit` state
    #[must_use]
    pub fn keep_state(mut self) -> Self {
        self.keep_state = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["buildx".to_string(), "rm".to_string()];

        if self.all_inactive {
            args.push("--all-inactive".to_string());
        }

        if self.force {
            args.push("--force".to_string());
        }

        if self.keep_daemon {
            args.push("--keep-daemon".to_string());
        }

        if self.keep_state {
            args.push("--keep-state".to_string());
        }

        for name in &self.names {
            args.push(name.clone());
        }

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for BuildxRmCommand {
    type Output = BuildxRmResult;

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
        Ok(BuildxRmResult::parse(&self.names, &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buildx_rm_basic() {
        let cmd = BuildxRmCommand::new("mybuilder");
        let args = cmd.build_args();
        assert_eq!(args, vec!["buildx", "rm", "mybuilder"]);
    }

    #[test]
    fn test_buildx_rm_multiple() {
        let cmd = BuildxRmCommand::new("builder1").name("builder2");
        let args = cmd.build_args();
        assert!(args.contains(&"builder1".to_string()));
        assert!(args.contains(&"builder2".to_string()));
    }

    #[test]
    fn test_buildx_rm_all_inactive() {
        let cmd = BuildxRmCommand::all_inactive();
        let args = cmd.build_args();
        assert!(args.contains(&"--all-inactive".to_string()));
    }

    #[test]
    fn test_buildx_rm_with_force() {
        let cmd = BuildxRmCommand::new("mybuilder").force();
        let args = cmd.build_args();
        assert!(args.contains(&"--force".to_string()));
    }

    #[test]
    fn test_buildx_rm_all_options() {
        let cmd = BuildxRmCommand::new("mybuilder")
            .force()
            .keep_daemon()
            .keep_state();
        let args = cmd.build_args();
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--keep-daemon".to_string()));
        assert!(args.contains(&"--keep-state".to_string()));
    }
}
