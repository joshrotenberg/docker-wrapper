//! Docker manifest create command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of manifest create command
#[derive(Debug, Clone)]
pub struct ManifestCreateResult {
    /// The manifest list name that was created
    pub manifest_list: String,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl ManifestCreateResult {
    /// Parse the manifest create output
    fn parse(manifest_list: &str, output: &CommandOutput) -> Self {
        Self {
            manifest_list: manifest_list.to_string(),
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker manifest create command builder
///
/// Creates a local manifest list for annotating and pushing to a registry.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, ManifestCreateCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = ManifestCreateCommand::new("myapp:latest")
///     .manifest("myapp:latest-amd64")
///     .manifest("myapp:latest-arm64")
///     .amend()
///     .execute()
///     .await?;
///
/// println!("Created manifest list: {}", result.manifest_list);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ManifestCreateCommand {
    /// The manifest list name
    manifest_list: String,
    /// Manifests to include in the list
    manifests: Vec<String>,
    /// Amend an existing manifest list
    amend: bool,
    /// Allow communication with an insecure registry
    insecure: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ManifestCreateCommand {
    /// Create a new manifest create command
    ///
    /// # Arguments
    ///
    /// * `manifest_list` - The name for the manifest list (e.g., "myapp:latest")
    #[must_use]
    pub fn new(manifest_list: impl Into<String>) -> Self {
        Self {
            manifest_list: manifest_list.into(),
            manifests: Vec::new(),
            amend: false,
            insecure: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add a manifest to the list
    ///
    /// # Arguments
    ///
    /// * `manifest` - The manifest to add (e.g., "myapp:latest-amd64")
    #[must_use]
    pub fn manifest(mut self, manifest: impl Into<String>) -> Self {
        self.manifests.push(manifest.into());
        self
    }

    /// Add multiple manifests to the list
    #[must_use]
    pub fn manifests<I, S>(mut self, manifests: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for manifest in manifests {
            self.manifests.push(manifest.into());
        }
        self
    }

    /// Amend an existing manifest list
    #[must_use]
    pub fn amend(mut self) -> Self {
        self.amend = true;
        self
    }

    /// Allow communication with an insecure registry
    #[must_use]
    pub fn insecure(mut self) -> Self {
        self.insecure = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["manifest".to_string(), "create".to_string()];

        if self.amend {
            args.push("--amend".to_string());
        }

        if self.insecure {
            args.push("--insecure".to_string());
        }

        args.push(self.manifest_list.clone());

        for manifest in &self.manifests {
            args.push(manifest.clone());
        }

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for ManifestCreateCommand {
    type Output = ManifestCreateResult;

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
        Ok(ManifestCreateResult::parse(&self.manifest_list, &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_create_basic() {
        let cmd = ManifestCreateCommand::new("myapp:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["manifest", "create", "myapp:latest"]);
    }

    #[test]
    fn test_manifest_create_with_manifests() {
        let cmd = ManifestCreateCommand::new("myapp:latest")
            .manifest("myapp:latest-amd64")
            .manifest("myapp:latest-arm64");
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "manifest",
                "create",
                "myapp:latest",
                "myapp:latest-amd64",
                "myapp:latest-arm64"
            ]
        );
    }

    #[test]
    fn test_manifest_create_with_amend() {
        let cmd = ManifestCreateCommand::new("myapp:latest")
            .manifest("myapp:latest-amd64")
            .amend();
        let args = cmd.build_args();
        assert!(args.contains(&"--amend".to_string()));
    }

    #[test]
    fn test_manifest_create_with_insecure() {
        let cmd = ManifestCreateCommand::new("myapp:latest")
            .manifest("myapp:latest-amd64")
            .insecure();
        let args = cmd.build_args();
        assert!(args.contains(&"--insecure".to_string()));
    }

    #[test]
    fn test_manifest_create_all_options() {
        let cmd = ManifestCreateCommand::new("myapp:latest")
            .manifests(vec!["myapp:latest-amd64", "myapp:latest-arm64"])
            .amend()
            .insecure();
        let args = cmd.build_args();
        assert!(args.contains(&"--amend".to_string()));
        assert!(args.contains(&"--insecure".to_string()));
        assert!(args.contains(&"myapp:latest-amd64".to_string()));
        assert!(args.contains(&"myapp:latest-arm64".to_string()));
    }
}
