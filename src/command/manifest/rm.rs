//! Docker manifest rm command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of manifest rm command
#[derive(Debug, Clone)]
pub struct ManifestRmResult {
    /// The manifest lists that were removed
    pub manifest_lists: Vec<String>,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl ManifestRmResult {
    /// Parse the manifest rm output
    fn parse(manifest_lists: &[String], output: &CommandOutput) -> Self {
        Self {
            manifest_lists: manifest_lists.to_vec(),
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker manifest rm command builder
///
/// Deletes one or more manifest lists from local storage.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, ManifestRmCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = ManifestRmCommand::new("myapp:latest")
///     .execute()
///     .await?;
///
/// println!("Removed {} manifest lists", result.manifest_lists.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ManifestRmCommand {
    /// The manifest lists to remove
    manifest_lists: Vec<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ManifestRmCommand {
    /// Create a new manifest rm command
    ///
    /// # Arguments
    ///
    /// * `manifest_list` - The manifest list to remove (e.g., "myapp:latest")
    #[must_use]
    pub fn new(manifest_list: impl Into<String>) -> Self {
        Self {
            manifest_lists: vec![manifest_list.into()],
            executor: CommandExecutor::new(),
        }
    }

    /// Add another manifest list to remove
    #[must_use]
    pub fn manifest_list(mut self, manifest_list: impl Into<String>) -> Self {
        self.manifest_lists.push(manifest_list.into());
        self
    }

    /// Add multiple manifest lists to remove
    #[must_use]
    pub fn manifest_lists<I, S>(mut self, manifest_lists: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for list in manifest_lists {
            self.manifest_lists.push(list.into());
        }
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["manifest".to_string(), "rm".to_string()];

        for list in &self.manifest_lists {
            args.push(list.clone());
        }

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for ManifestRmCommand {
    type Output = ManifestRmResult;

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
        Ok(ManifestRmResult::parse(&self.manifest_lists, &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_rm_basic() {
        let cmd = ManifestRmCommand::new("myapp:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["manifest", "rm", "myapp:latest"]);
    }

    #[test]
    fn test_manifest_rm_multiple() {
        let cmd = ManifestRmCommand::new("myapp:latest").manifest_list("myapp:v1");
        let args = cmd.build_args();
        assert_eq!(args, vec!["manifest", "rm", "myapp:latest", "myapp:v1"]);
    }

    #[test]
    fn test_manifest_rm_with_manifest_lists() {
        let cmd =
            ManifestRmCommand::new("myapp:latest").manifest_lists(vec!["myapp:v1", "myapp:v2"]);
        let args = cmd.build_args();
        assert!(args.contains(&"myapp:latest".to_string()));
        assert!(args.contains(&"myapp:v1".to_string()));
        assert!(args.contains(&"myapp:v2".to_string()));
    }
}
