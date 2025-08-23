//! Docker system df command implementation.

use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

/// Image disk usage information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageUsage {
    /// Total number of images
    #[serde(default)]
    pub total_count: usize,

    /// Number of active images
    #[serde(default)]
    pub active: usize,

    /// Total size in bytes
    #[serde(default)]
    pub size: i64,

    /// Reclaimable size in bytes
    #[serde(default)]
    pub reclaimable_size: i64,
}

/// Container disk usage information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerUsage {
    /// Total number of containers
    #[serde(default)]
    pub total_count: usize,

    /// Number of running containers
    #[serde(default)]
    pub running: usize,

    /// Number of paused containers
    #[serde(default)]
    pub paused: usize,

    /// Number of stopped containers
    #[serde(default)]
    pub stopped: usize,

    /// Total size in bytes
    #[serde(default)]
    pub size: i64,

    /// Reclaimable size in bytes
    #[serde(default)]
    pub reclaimable_size: i64,
}

/// Volume disk usage information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VolumeUsage {
    /// Total number of volumes
    #[serde(default)]
    pub total_count: usize,

    /// Number of active volumes
    #[serde(default)]
    pub active: usize,

    /// Total size in bytes
    #[serde(default)]
    pub size: i64,

    /// Reclaimable size in bytes
    #[serde(default)]
    pub reclaimable_size: i64,
}

/// Build cache disk usage information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildCacheUsage {
    /// Total number of build cache entries
    #[serde(default)]
    pub total_count: usize,

    /// Number of active build cache entries
    #[serde(default)]
    pub active: usize,

    /// Total size in bytes
    #[serde(default)]
    pub size: i64,

    /// Reclaimable size in bytes
    #[serde(default)]
    pub reclaimable_size: i64,
}

/// Docker disk usage information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DiskUsage {
    /// Image disk usage
    pub images: Vec<ImageInfo>,

    /// Container disk usage
    pub containers: Vec<ContainerInfo>,

    /// Volume disk usage
    pub volumes: Vec<VolumeInfo>,

    /// Build cache disk usage
    #[serde(default)]
    pub build_cache: Vec<BuildCacheInfo>,
}

/// Detailed image information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageInfo {
    /// Image ID
    #[serde(rename = "ID")]
    pub id: String,

    /// Repository
    #[serde(default)]
    pub repository: String,

    /// Tag
    #[serde(default)]
    pub tag: String,

    /// Created timestamp
    #[serde(default)]
    pub created: i64,

    /// Size in bytes
    #[serde(default)]
    pub size: i64,

    /// Shared size in bytes
    #[serde(default)]
    pub shared_size: i64,

    /// Virtual size in bytes
    #[serde(default)]
    pub virtual_size: i64,

    /// Number of containers using this image
    #[serde(default)]
    pub containers: i32,
}

/// Detailed container information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerInfo {
    /// Container ID
    #[serde(rename = "ID")]
    pub id: String,

    /// Container names
    #[serde(default)]
    pub names: Vec<String>,

    /// Image
    #[serde(default)]
    pub image: String,

    /// Created timestamp
    #[serde(default)]
    pub created: i64,

    /// State
    #[serde(default)]
    pub state: String,

    /// Status
    #[serde(default)]
    pub status: String,

    /// Size in bytes (read/write layer)
    #[serde(default, rename = "SizeRw")]
    pub size_rw: i64,

    /// Root filesystem size in bytes
    #[serde(default, rename = "SizeRootFs")]
    pub size_root_fs: i64,
}

/// Detailed volume information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VolumeInfo {
    /// Volume name
    pub name: String,

    /// Driver
    #[serde(default)]
    pub driver: String,

    /// Mount point
    #[serde(default)]
    pub mount_point: String,

    /// Created timestamp
    #[serde(default)]
    pub created_at: String,

    /// Size in bytes
    #[serde(default)]
    pub size: i64,

    /// Number of containers using this volume
    #[serde(default, rename = "RefCount")]
    pub ref_count: i32,
}

/// Build cache information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildCacheInfo {
    /// Cache ID
    #[serde(rename = "ID")]
    pub id: String,

    /// Parent ID
    #[serde(default)]
    pub parent: String,

    /// Cache type
    #[serde(default, rename = "Type")]
    pub cache_type: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Created timestamp
    #[serde(default)]
    pub created_at: String,

    /// Last used timestamp
    #[serde(default)]
    pub last_used_at: String,

    /// Usage count
    #[serde(default)]
    pub usage_count: i64,

    /// Size in bytes
    #[serde(default)]
    pub size: i64,

    /// Whether the cache is in use
    #[serde(default)]
    pub in_use: bool,

    /// Whether the cache is shared
    #[serde(default)]
    pub shared: bool,
}

/// Docker system df command
///
/// Show docker disk usage
#[derive(Debug, Clone)]
pub struct SystemDfCommand {
    /// Show detailed information
    verbose: bool,

    /// Format output using a custom template
    format: Option<String>,

    /// Command executor
    pub executor: CommandExecutor,
}

impl SystemDfCommand {
    /// Create a new system df command
    #[must_use]
    pub fn new() -> Self {
        Self {
            verbose: false,
            format: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Show detailed information
    #[must_use]
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Format output using a custom template
    #[must_use]
    pub fn format(mut self, template: impl Into<String>) -> Self {
        self.format = Some(template.into());
        self
    }

    /// Execute the system df command
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute or if Docker is not available.
    pub async fn run(&self) -> Result<DiskUsage> {
        self.execute().await
    }
}

impl Default for SystemDfCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for SystemDfCommand {
    type Output = DiskUsage;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["system".to_string(), "df".to_string()];

        if self.verbose {
            args.push("--verbose".to_string());
        }

        // Always use JSON format for parsing
        args.push("--format".to_string());
        args.push("json".to_string());

        args.extend(self.executor.raw_args.clone());
        args
    }

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        let output = self
            .executor
            .execute_command(&command_name, command_args)
            .await?;
        let stdout = &output.stdout;

        // Parse JSON output
        let usage: DiskUsage =
            serde_json::from_str(stdout).map_err(|e| crate::error::Error::ParseError {
                message: format!("Failed to parse disk usage: {e}"),
            })?;

        Ok(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_df_builder() {
        let cmd = SystemDfCommand::new().verbose();

        let args = cmd.build_command_args();
        assert_eq!(args[0], "system");
        assert!(args.contains(&"df".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }
}
