//! Docker manifest annotate command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of manifest annotate command
#[derive(Debug, Clone)]
pub struct ManifestAnnotateResult {
    /// The manifest list that was annotated
    pub manifest_list: String,
    /// The manifest that was annotated
    pub manifest: String,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl ManifestAnnotateResult {
    /// Parse the manifest annotate output
    fn parse(manifest_list: &str, manifest: &str, output: &CommandOutput) -> Self {
        Self {
            manifest_list: manifest_list.to_string(),
            manifest: manifest.to_string(),
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker manifest annotate command builder
///
/// Adds additional information to a local image manifest.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, ManifestAnnotateCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = ManifestAnnotateCommand::new("myapp:latest", "myapp:latest-arm64")
///     .os("linux")
///     .arch("arm64")
///     .variant("v8")
///     .execute()
///     .await?;
///
/// println!("Annotated manifest: {}", result.manifest);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ManifestAnnotateCommand {
    /// The manifest list name
    manifest_list: String,
    /// The manifest to annotate
    manifest: String,
    /// Set architecture
    arch: Option<String>,
    /// Set operating system
    os: Option<String>,
    /// Set operating system features
    os_features: Vec<String>,
    /// Set operating system version
    os_version: Option<String>,
    /// Set architecture variant
    variant: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ManifestAnnotateCommand {
    /// Create a new manifest annotate command
    ///
    /// # Arguments
    ///
    /// * `manifest_list` - The manifest list name (e.g., "myapp:latest")
    /// * `manifest` - The manifest to annotate (e.g., "myapp:latest-arm64")
    #[must_use]
    pub fn new(manifest_list: impl Into<String>, manifest: impl Into<String>) -> Self {
        Self {
            manifest_list: manifest_list.into(),
            manifest: manifest.into(),
            arch: None,
            os: None,
            os_features: Vec::new(),
            os_version: None,
            variant: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Set the architecture (e.g., "amd64", "arm64", "arm")
    #[must_use]
    pub fn arch(mut self, arch: impl Into<String>) -> Self {
        self.arch = Some(arch.into());
        self
    }

    /// Set the operating system (e.g., "linux", "windows")
    #[must_use]
    pub fn os(mut self, os: impl Into<String>) -> Self {
        self.os = Some(os.into());
        self
    }

    /// Add an operating system feature
    #[must_use]
    pub fn os_feature(mut self, feature: impl Into<String>) -> Self {
        self.os_features.push(feature.into());
        self
    }

    /// Set the operating system version
    #[must_use]
    pub fn os_version(mut self, version: impl Into<String>) -> Self {
        self.os_version = Some(version.into());
        self
    }

    /// Set the architecture variant (e.g., "v7", "v8")
    #[must_use]
    pub fn variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["manifest".to_string(), "annotate".to_string()];

        if let Some(ref arch) = self.arch {
            args.push("--arch".to_string());
            args.push(arch.clone());
        }

        if let Some(ref os) = self.os {
            args.push("--os".to_string());
            args.push(os.clone());
        }

        for feature in &self.os_features {
            args.push("--os-features".to_string());
            args.push(feature.clone());
        }

        if let Some(ref version) = self.os_version {
            args.push("--os-version".to_string());
            args.push(version.clone());
        }

        if let Some(ref variant) = self.variant {
            args.push("--variant".to_string());
            args.push(variant.clone());
        }

        args.push(self.manifest_list.clone());
        args.push(self.manifest.clone());

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for ManifestAnnotateCommand {
    type Output = ManifestAnnotateResult;

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
        Ok(ManifestAnnotateResult::parse(
            &self.manifest_list,
            &self.manifest,
            &output,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_annotate_basic() {
        let cmd = ManifestAnnotateCommand::new("myapp:latest", "myapp:latest-amd64");
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["manifest", "annotate", "myapp:latest", "myapp:latest-amd64"]
        );
    }

    #[test]
    fn test_manifest_annotate_with_arch() {
        let cmd = ManifestAnnotateCommand::new("myapp:latest", "myapp:latest-arm64").arch("arm64");
        let args = cmd.build_args();
        assert!(args.contains(&"--arch".to_string()));
        assert!(args.contains(&"arm64".to_string()));
    }

    #[test]
    fn test_manifest_annotate_with_os() {
        let cmd = ManifestAnnotateCommand::new("myapp:latest", "myapp:latest-amd64").os("linux");
        let args = cmd.build_args();
        assert!(args.contains(&"--os".to_string()));
        assert!(args.contains(&"linux".to_string()));
    }

    #[test]
    fn test_manifest_annotate_with_variant() {
        let cmd = ManifestAnnotateCommand::new("myapp:latest", "myapp:latest-arm64").variant("v8");
        let args = cmd.build_args();
        assert!(args.contains(&"--variant".to_string()));
        assert!(args.contains(&"v8".to_string()));
    }

    #[test]
    fn test_manifest_annotate_all_options() {
        let cmd = ManifestAnnotateCommand::new("myapp:latest", "myapp:latest-arm64")
            .arch("arm64")
            .os("linux")
            .os_feature("sse4")
            .os_version("1.0")
            .variant("v8");
        let args = cmd.build_args();
        assert!(args.contains(&"--arch".to_string()));
        assert!(args.contains(&"--os".to_string()));
        assert!(args.contains(&"--os-features".to_string()));
        assert!(args.contains(&"--os-version".to_string()));
        assert!(args.contains(&"--variant".to_string()));
    }
}
