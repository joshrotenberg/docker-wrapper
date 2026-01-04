//! Docker manifest push command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of manifest push command
#[derive(Debug, Clone)]
pub struct ManifestPushResult {
    /// The manifest list that was pushed
    pub manifest_list: String,
    /// The digest of the pushed manifest (if available)
    pub digest: Option<String>,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl ManifestPushResult {
    /// Parse the manifest push output
    fn parse(manifest_list: &str, output: &CommandOutput) -> Self {
        // The output typically contains the digest of the pushed manifest
        let digest = output
            .stdout
            .lines()
            .find(|line| line.starts_with("sha256:"))
            .map(|s| s.trim().to_string());

        Self {
            manifest_list: manifest_list.to_string(),
            digest,
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker manifest push command builder
///
/// Pushes a manifest list to a repository.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, ManifestPushCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = ManifestPushCommand::new("myapp:latest")
///     .purge()
///     .execute()
///     .await?;
///
/// if let Some(digest) = &result.digest {
///     println!("Pushed manifest with digest: {}", digest);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ManifestPushCommand {
    /// The manifest list to push
    manifest_list: String,
    /// Allow push to an insecure registry
    insecure: bool,
    /// Remove the local manifest list after push
    purge: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ManifestPushCommand {
    /// Create a new manifest push command
    ///
    /// # Arguments
    ///
    /// * `manifest_list` - The manifest list to push (e.g., "myapp:latest")
    #[must_use]
    pub fn new(manifest_list: impl Into<String>) -> Self {
        Self {
            manifest_list: manifest_list.into(),
            insecure: false,
            purge: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Allow push to an insecure registry
    #[must_use]
    pub fn insecure(mut self) -> Self {
        self.insecure = true;
        self
    }

    /// Remove the local manifest list after push
    #[must_use]
    pub fn purge(mut self) -> Self {
        self.purge = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["manifest".to_string(), "push".to_string()];

        if self.insecure {
            args.push("--insecure".to_string());
        }

        if self.purge {
            args.push("--purge".to_string());
        }

        args.push(self.manifest_list.clone());

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for ManifestPushCommand {
    type Output = ManifestPushResult;

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
        Ok(ManifestPushResult::parse(&self.manifest_list, &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_push_basic() {
        let cmd = ManifestPushCommand::new("myapp:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["manifest", "push", "myapp:latest"]);
    }

    #[test]
    fn test_manifest_push_with_insecure() {
        let cmd = ManifestPushCommand::new("myapp:latest").insecure();
        let args = cmd.build_args();
        assert!(args.contains(&"--insecure".to_string()));
    }

    #[test]
    fn test_manifest_push_with_purge() {
        let cmd = ManifestPushCommand::new("myapp:latest").purge();
        let args = cmd.build_args();
        assert!(args.contains(&"--purge".to_string()));
    }

    #[test]
    fn test_manifest_push_all_options() {
        let cmd = ManifestPushCommand::new("myapp:latest").insecure().purge();
        let args = cmd.build_args();
        assert!(args.contains(&"--insecure".to_string()));
        assert!(args.contains(&"--purge".to_string()));
    }

    #[test]
    fn test_manifest_push_result_parse_with_digest() {
        let output = CommandOutput {
            stdout: "sha256:abc123def456".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let result = ManifestPushResult::parse("myapp:latest", &output);
        assert_eq!(result.digest, Some("sha256:abc123def456".to_string()));
    }
}
