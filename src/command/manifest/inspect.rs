//! Docker manifest inspect command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Platform information in a manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPlatform {
    /// Architecture (e.g., "amd64", "arm64")
    pub architecture: Option<String>,
    /// Operating system (e.g., "linux", "windows")
    pub os: Option<String>,
    /// OS version
    #[serde(rename = "os.version")]
    pub os_version: Option<String>,
    /// Architecture variant (e.g., "v8")
    pub variant: Option<String>,
}

/// Information about a manifest or manifest list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestInfo {
    /// Schema version
    #[serde(rename = "schemaVersion")]
    pub schema_version: Option<i32>,
    /// Media type
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    /// Digest of the manifest
    pub digest: Option<String>,
    /// Size of the manifest in bytes
    pub size: Option<i64>,
    /// Platform information
    pub platform: Option<ManifestPlatform>,
    /// Manifests in the list (for manifest lists)
    pub manifests: Option<Vec<ManifestInfo>>,
    /// Raw JSON output
    #[serde(skip)]
    pub raw_json: String,
}

impl ManifestInfo {
    /// Parse the manifest inspect output
    fn parse(output: &CommandOutput) -> Self {
        let stdout = output.stdout.trim();
        if stdout.is_empty() {
            return Self {
                schema_version: None,
                media_type: None,
                digest: None,
                size: None,
                platform: None,
                manifests: None,
                raw_json: String::new(),
            };
        }

        let mut info: ManifestInfo =
            serde_json::from_str(stdout).unwrap_or_else(|_| ManifestInfo {
                schema_version: None,
                media_type: None,
                digest: None,
                size: None,
                platform: None,
                manifests: None,
                raw_json: String::new(),
            });
        info.raw_json = stdout.to_string();
        info
    }
}

/// Docker manifest inspect command builder
///
/// Displays an image manifest or manifest list.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, ManifestInspectCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let info = ManifestInspectCommand::new("myapp:latest")
///     .verbose()
///     .execute()
///     .await?;
///
/// if let Some(manifests) = &info.manifests {
///     for manifest in manifests {
///         if let Some(platform) = &manifest.platform {
///             println!("Platform: {:?}/{:?}",
///                 platform.os, platform.architecture);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ManifestInspectCommand {
    /// The manifest list name (optional)
    manifest_list: Option<String>,
    /// The manifest to inspect
    manifest: String,
    /// Allow communication with an insecure registry
    insecure: bool,
    /// Output additional info including layers and platform
    verbose: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ManifestInspectCommand {
    /// Create a new manifest inspect command
    ///
    /// # Arguments
    ///
    /// * `manifest` - The manifest to inspect (e.g., "myapp:latest")
    #[must_use]
    pub fn new(manifest: impl Into<String>) -> Self {
        Self {
            manifest_list: None,
            manifest: manifest.into(),
            insecure: false,
            verbose: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Set the manifest list to inspect from
    #[must_use]
    pub fn manifest_list(mut self, manifest_list: impl Into<String>) -> Self {
        self.manifest_list = Some(manifest_list.into());
        self
    }

    /// Allow communication with an insecure registry
    #[must_use]
    pub fn insecure(mut self) -> Self {
        self.insecure = true;
        self
    }

    /// Output additional info including layers and platform
    #[must_use]
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["manifest".to_string(), "inspect".to_string()];

        if self.insecure {
            args.push("--insecure".to_string());
        }

        if self.verbose {
            args.push("--verbose".to_string());
        }

        if let Some(ref list) = self.manifest_list {
            args.push(list.clone());
        }

        args.push(self.manifest.clone());

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for ManifestInspectCommand {
    type Output = ManifestInfo;

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
        Ok(ManifestInfo::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_inspect_basic() {
        let cmd = ManifestInspectCommand::new("myapp:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["manifest", "inspect", "myapp:latest"]);
    }

    #[test]
    fn test_manifest_inspect_with_list() {
        let cmd = ManifestInspectCommand::new("myapp:latest-amd64").manifest_list("myapp:latest");
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["manifest", "inspect", "myapp:latest", "myapp:latest-amd64"]
        );
    }

    #[test]
    fn test_manifest_inspect_with_insecure() {
        let cmd = ManifestInspectCommand::new("myapp:latest").insecure();
        let args = cmd.build_args();
        assert!(args.contains(&"--insecure".to_string()));
    }

    #[test]
    fn test_manifest_inspect_with_verbose() {
        let cmd = ManifestInspectCommand::new("myapp:latest").verbose();
        let args = cmd.build_args();
        assert!(args.contains(&"--verbose".to_string()));
    }

    #[test]
    fn test_manifest_inspect_all_options() {
        let cmd = ManifestInspectCommand::new("myapp:latest")
            .insecure()
            .verbose();
        let args = cmd.build_args();
        assert!(args.contains(&"--insecure".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
    }

    #[test]
    fn test_manifest_info_parse_empty() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let info = ManifestInfo::parse(&output);
        assert!(info.schema_version.is_none());
    }

    #[test]
    fn test_manifest_info_parse_json() {
        let json = r#"{"schemaVersion": 2, "mediaType": "application/vnd.docker.distribution.manifest.list.v2+json"}"#;
        let output = CommandOutput {
            stdout: json.to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let info = ManifestInfo::parse(&output);
        assert_eq!(info.schema_version, Some(2));
        assert_eq!(
            info.media_type,
            Some("application/vnd.docker.distribution.manifest.list.v2+json".to_string())
        );
    }
}
