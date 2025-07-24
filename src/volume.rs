//! Docker volume management module.
//!
//! This module provides complete Docker volume lifecycle management including:
//! - Volume creation and removal
//! - Volume listing and inspection
//! - Volume mounting and unmounting
//! - Volume cleanup and pruning
//! - Different volume driver support
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DockerError> {
//!     let client = DockerClient::new().await?;
//!     let volume_manager = client.volumes();
//!
//!     // Create a volume
//!     let volume_config = VolumeConfig::new("my-volume")
//!         .driver("local")
//!         .label("purpose", "database")
//!         .option("type", "tmpfs")
//!         .option("device", "tmpfs");
//!
//!     let volume = volume_manager.create(volume_config).await?;
//!
//!     // List all volumes
//!     let volumes = volume_manager.list(ListVolumesOptions::default()).await?;
//!     println!("Found {} volumes", volumes.len());
//!
//!     // Inspect the volume
//!     let inspect = volume_manager.inspect(&volume.name).await?;
//!     println!("Volume driver: {}", inspect.driver);
//!
//!     // Cleanup
//!     volume_manager.remove(&volume.name, RemoveVolumeOptions::default()).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::executor::ExecutionConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Docker volume representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerVolume {
    /// Volume name
    #[serde(rename = "Name")]
    pub name: String,
    /// Driver name
    #[serde(rename = "Driver")]
    pub driver: String,
    /// Mount point on host
    #[serde(rename = "Mountpoint")]
    pub mountpoint: String,
    /// Creation timestamp
    #[serde(rename = "CreatedAt")]
    pub created_at: Option<String>,
    /// Scope (local or global)
    #[serde(rename = "Scope")]
    pub scope: String,
    /// Labels
    #[serde(rename = "Labels")]
    pub labels: Option<HashMap<String, String>>,
    /// Options
    #[serde(rename = "Options")]
    pub options: Option<HashMap<String, String>>,
    /// Usage data
    #[serde(rename = "UsageData")]
    pub usage_data: Option<VolumeUsageData>,
}

impl DockerVolume {
    /// Get the created time as SystemTime
    pub fn created_time(&self) -> DockerResult<Option<SystemTime>> {
        if let Some(created_str) = &self.created_at {
            let timestamp = chrono::DateTime::parse_from_rfc3339(created_str)
                .map_err(|e| DockerError::ParseError(format!("Invalid timestamp: {}", e)))?;
            Ok(Some(
                SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_secs(timestamp.timestamp() as u64),
            ))
        } else {
            Ok(None)
        }
    }

    /// Get mountpoint as PathBuf
    pub fn mountpoint_path(&self) -> PathBuf {
        PathBuf::from(&self.mountpoint)
    }

    /// Check if volume has label
    pub fn has_label(&self, key: &str) -> bool {
        self.labels
            .as_ref()
            .map_or(false, |labels| labels.contains_key(key))
    }

    /// Get label value
    pub fn get_label(&self, key: &str) -> Option<&str> {
        self.labels.as_ref()?.get(key).map(String::as_str)
    }

    /// Check if volume has option
    pub fn has_option(&self, key: &str) -> bool {
        self.options
            .as_ref()
            .map_or(false, |options| options.contains_key(key))
    }

    /// Get option value
    pub fn get_option(&self, key: &str) -> Option<&str> {
        self.options.as_ref()?.get(key).map(String::as_str)
    }

    /// Get size in bytes if available
    pub fn size_bytes(&self) -> Option<i64> {
        self.usage_data.as_ref()?.size
    }

    /// Get reference count if available
    pub fn ref_count(&self) -> Option<i64> {
        self.usage_data.as_ref()?.ref_count
    }
}

/// Volume usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeUsageData {
    /// Size in bytes
    #[serde(rename = "Size")]
    pub size: Option<i64>,
    /// Reference count
    #[serde(rename = "RefCount")]
    pub ref_count: Option<i64>,
}

/// Volume configuration for creation
#[derive(Debug, Clone)]
pub struct VolumeConfig {
    /// Volume name
    pub name: String,
    /// Driver name
    pub driver: String,
    /// Driver options
    pub driver_opts: HashMap<String, String>,
    /// Labels
    pub labels: HashMap<String, String>,
}

impl VolumeConfig {
    /// Create a new volume configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            driver: "local".to_string(),
            driver_opts: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    /// Set driver
    pub fn driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = driver.into();
        self
    }

    /// Add driver option
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.driver_opts.insert(key.into(), value.into());
        self
    }

    /// Add label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set as tmpfs volume
    pub fn tmpfs(mut self) -> Self {
        self.driver_opts
            .insert("type".to_string(), "tmpfs".to_string());
        self.driver_opts
            .insert("device".to_string(), "tmpfs".to_string());
        self
    }

    /// Set as bind mount
    pub fn bind_mount(mut self, host_path: impl Into<String>) -> Self {
        self.driver_opts
            .insert("type".to_string(), "none".to_string());
        self.driver_opts.insert("o".to_string(), "bind".to_string());
        self.driver_opts
            .insert("device".to_string(), host_path.into());
        self
    }

    /// Set NFS mount
    pub fn nfs(mut self, server: impl Into<String>, path: impl Into<String>) -> Self {
        self.driver_opts
            .insert("type".to_string(), "nfs".to_string());
        self.driver_opts
            .insert("o".to_string(), format!("addr={},rw", server.into()));
        self.driver_opts
            .insert("device".to_string(), format!(":{}", path.into()));
        self
    }
}

/// Volume source types for container mounting
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeSource {
    /// Named volume
    Named(String),
    /// Host path bind mount
    HostPath(PathBuf),
    /// Anonymous volume
    Anonymous,
    /// Tmpfs mount
    Tmpfs,
}

impl VolumeSource {
    /// Create a named volume source
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    /// Create a host path bind mount
    pub fn host_path(path: impl Into<PathBuf>) -> Self {
        Self::HostPath(path.into())
    }

    /// Create an anonymous volume
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    /// Create a tmpfs mount
    pub fn tmpfs() -> Self {
        Self::Tmpfs
    }

    /// Convert to Docker CLI mount string
    pub fn to_mount_string(&self, target: &str) -> String {
        match self {
            Self::Named(name) => format!("{}:{}", name, target),
            Self::HostPath(path) => format!("{}:{}", path.display(), target),
            Self::Anonymous => target.to_string(),
            Self::Tmpfs => format!("tmpfs:{}", target),
        }
    }

    /// Get source identifier
    pub fn identifier(&self) -> String {
        match self {
            Self::Named(name) => name.clone(),
            Self::HostPath(path) => path.to_string_lossy().to_string(),
            Self::Anonymous => "<anonymous>".to_string(),
            Self::Tmpfs => "<tmpfs>".to_string(),
        }
    }
}

impl std::fmt::Display for VolumeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

/// Volume mount specification
#[derive(Debug, Clone)]
pub struct VolumeMount {
    /// Source specification
    pub source: VolumeSource,
    /// Target path in container
    pub target: String,
    /// Read-only mount
    pub read_only: bool,
    /// Mount options
    pub options: Vec<String>,
}

impl VolumeMount {
    /// Create a new volume mount
    pub fn new(source: VolumeSource, target: impl Into<String>) -> Self {
        Self {
            source,
            target: target.into(),
            read_only: false,
            options: Vec::new(),
        }
    }

    /// Make mount read-only
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Add mount option
    pub fn option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Convert to Docker CLI argument
    pub fn to_cli_arg(&self) -> String {
        let mut mount_str = self.source.to_mount_string(&self.target);

        if self.read_only {
            mount_str.push_str(":ro");
        }

        for option in &self.options {
            mount_str.push(':');
            mount_str.push_str(option);
        }

        mount_str
    }
}

/// Options for listing volumes
#[derive(Debug, Clone, Default)]
pub struct ListVolumesOptions {
    /// Show dangling volumes only
    pub dangling: Option<bool>,
    /// Filter by driver
    pub driver: Option<String>,
    /// Filter by label
    pub labels: Vec<String>,
    /// Filter by name pattern
    pub name: Option<String>,
}

impl ListVolumesOptions {
    /// Create new list options
    pub fn new() -> Self {
        Self::default()
    }

    /// Show only dangling volumes
    pub fn dangling_only(mut self) -> Self {
        self.dangling = Some(true);
        self
    }

    /// Show only non-dangling volumes
    pub fn non_dangling_only(mut self) -> Self {
        self.dangling = Some(false);
        self
    }

    /// Filter by driver
    pub fn driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = Some(driver.into());
        self
    }

    /// Filter by label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Filter by name pattern
    pub fn name_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.name = Some(pattern.into());
        self
    }
}

/// Options for removing volumes
#[derive(Debug, Clone, Default)]
pub struct RemoveVolumeOptions {
    /// Force removal
    pub force: bool,
}

impl RemoveVolumeOptions {
    /// Create new remove options
    pub fn new() -> Self {
        Self::default()
    }

    /// Force removal
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
}

/// Volume inspection details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInspect {
    /// Volume name
    #[serde(rename = "Name")]
    pub name: String,
    /// Driver
    #[serde(rename = "Driver")]
    pub driver: String,
    /// Mount point
    #[serde(rename = "Mountpoint")]
    pub mountpoint: String,
    /// Creation time
    #[serde(rename = "CreatedAt")]
    pub created_at: String,
    /// Status
    #[serde(rename = "Status")]
    pub status: Option<HashMap<String, serde_json::Value>>,
    /// Labels
    #[serde(rename = "Labels")]
    pub labels: HashMap<String, String>,
    /// Scope
    #[serde(rename = "Scope")]
    pub scope: String,
    /// Options
    #[serde(rename = "Options")]
    pub options: HashMap<String, String>,
    /// Usage data
    #[serde(rename = "UsageData")]
    pub usage_data: Option<VolumeUsageData>,
}

impl VolumeInspect {
    /// Get the created time as SystemTime
    pub fn created_time(&self) -> DockerResult<SystemTime> {
        let timestamp = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| DockerError::ParseError(format!("Invalid timestamp: {}", e)))?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp.timestamp() as u64))
    }

    /// Get mountpoint as PathBuf
    pub fn mountpoint_path(&self) -> PathBuf {
        PathBuf::from(&self.mountpoint)
    }
}

/// Volume manager providing all volume operations
pub struct VolumeManager<'a> {
    client: &'a DockerClient,
}

impl<'a> VolumeManager<'a> {
    /// Create a new volume manager
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Create a new volume
    pub async fn create(&self, config: VolumeConfig) -> DockerResult<DockerVolume> {
        let mut args = vec!["volume".to_string(), "create".to_string()];

        // Add driver
        if config.driver != "local" {
            args.push("--driver".to_string());
            args.push(config.driver);
        }

        // Add driver options
        for (key, value) in &config.driver_opts {
            args.push("--opt".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add labels
        for (key, value) in &config.labels {
            args.push("--label".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add volume name
        args.push(config.name.clone());

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        // Get the created volume information
        self.get_by_name(&config.name)
            .await?
            .ok_or_else(|| DockerError::NotFound {
                message: format!("Created volume not found: {}", config.name),
            })
    }

    /// List volumes
    pub async fn list(&self, options: ListVolumesOptions) -> DockerResult<Vec<DockerVolume>> {
        let mut args = vec![
            "volume".to_string(),
            "ls".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];

        // Add filters
        if let Some(dangling) = options.dangling {
            args.push("--filter".to_string());
            args.push(format!("dangling={}", dangling));
        }

        if let Some(driver) = &options.driver {
            args.push("--filter".to_string());
            args.push(format!("driver={}", driver));
        }

        for label in &options.labels {
            args.push("--filter".to_string());
            args.push(format!("label={}", label));
        }

        if let Some(name) = &options.name {
            args.push("--filter".to_string());
            args.push(format!("name={}", name));
        }

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;
        let mut volumes = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<DockerVolume>(line) {
                Ok(volume) => volumes.push(volume),
                Err(e) => {
                    log::warn!("Failed to parse volume JSON: {} - {}", e, line);
                }
            }
        }

        Ok(volumes)
    }

    /// Inspect a volume
    pub async fn inspect(&self, volume_name: &str) -> DockerResult<VolumeInspect> {
        let args = vec![
            "volume".to_string(),
            "inspect".to_string(),
            volume_name.to_string(),
        ];

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;
        let inspects: Vec<VolumeInspect> = serde_json::from_str(&stdout)
            .map_err(|e| DockerError::ParseError(format!("Invalid inspect JSON: {}", e)))?;

        inspects
            .into_iter()
            .next()
            .ok_or_else(|| DockerError::VolumeNotFound {
                name: volume_name.to_string(),
            })
    }

    /// Remove a volume
    pub async fn remove(
        &self,
        volume_name: &str,
        options: RemoveVolumeOptions,
    ) -> DockerResult<()> {
        let mut args = vec!["volume".to_string(), "rm".to_string()];

        if options.force {
            args.push("--force".to_string());
        }

        args.push(volume_name.to_string());

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        Ok(())
    }

    /// Prune unused volumes
    pub async fn prune(&self) -> DockerResult<VolumePruneResult> {
        let args = vec![
            "volume".to_string(),
            "prune".to_string(),
            "--force".to_string(),
        ];

        let output = self
            .client
            .executor()
            .execute(&args, Some(ExecutionConfig::default()))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        let stdout = &output.stdout;

        // Parse deleted volumes and reclaimed space from output
        let mut deleted_volumes = Vec::new();
        let mut reclaimed_space = 0u64;

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if line.contains("Total reclaimed space:") {
                // Extract the space amount
                if let Some(space_str) = line.split(':').nth(1) {
                    if let Some(num_str) = space_str.trim().split_whitespace().next() {
                        if let Ok(num) = num_str.parse::<f64>() {
                            // Convert to bytes (rough approximation)
                            reclaimed_space = (num * 1_000_000_000.0) as u64;
                        }
                    }
                }
            } else if !line.contains("Deleted Volumes") {
                // Volume names are typically listed
                deleted_volumes.push(line.trim().to_string());
            }
        }

        Ok(VolumePruneResult {
            deleted_volumes,
            reclaimed_space,
        })
    }

    /// Get volume by name
    pub async fn get_by_name(&self, name: &str) -> DockerResult<Option<DockerVolume>> {
        let options = ListVolumesOptions::new().name_pattern(name);
        let volumes = self.list(options).await?;

        Ok(volumes.into_iter().find(|v| v.name == name))
    }

    /// Check if volume exists
    pub async fn exists(&self, volume_name: &str) -> DockerResult<bool> {
        match self.inspect(volume_name).await {
            Ok(_) => Ok(true),
            Err(DockerError::VolumeNotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Create volume if it doesn't exist
    pub async fn create_if_not_exists(&self, config: VolumeConfig) -> DockerResult<DockerVolume> {
        match self.get_by_name(&config.name).await? {
            Some(volume) => Ok(volume),
            None => self.create(config).await,
        }
    }

    /// Remove multiple volumes
    pub async fn remove_many(
        &self,
        volume_names: &[String],
        options: RemoveVolumeOptions,
    ) -> DockerResult<Vec<String>> {
        let mut removed = Vec::new();
        let mut errors = Vec::new();

        for name in volume_names {
            match self.remove(name, options.clone()).await {
                Ok(()) => removed.push(name.clone()),
                Err(e) => errors.push(format!("{}: {}", name, e)),
            }
        }

        if !errors.is_empty() {
            return Err(DockerError::CommandFailed {
                command: "volume rm".to_string(),
                exit_code: 1,
                stdout: String::new(),
                stderr: format!("Failed to remove some volumes: {}", errors.join(", ")),
            });
        }

        Ok(removed)
    }

    /// Get volumes usage statistics
    pub async fn usage_stats(&self) -> DockerResult<VolumeUsageStats> {
        let volumes = self.list(ListVolumesOptions::default()).await?;

        let total_volumes = volumes.len();
        let mut total_size = 0i64;
        let mut volumes_with_size = 0;
        let mut dangling_count = 0;

        for volume in &volumes {
            if let Some(size) = volume.size_bytes() {
                total_size += size;
                volumes_with_size += 1;
            }

            if volume.ref_count().unwrap_or(0) == 0 {
                dangling_count += 1;
            }
        }

        Ok(VolumeUsageStats {
            total_volumes,
            total_size,
            volumes_with_size,
            dangling_count,
            drivers: volumes
                .iter()
                .map(|v| v.driver.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        })
    }
}

/// Result of volume pruning operation
#[derive(Debug, Clone)]
pub struct VolumePruneResult {
    /// List of deleted volume names
    pub deleted_volumes: Vec<String>,
    /// Total reclaimed space in bytes
    pub reclaimed_space: u64,
}

/// Volume usage statistics
#[derive(Debug, Clone)]
pub struct VolumeUsageStats {
    /// Total number of volumes
    pub total_volumes: usize,
    /// Total size in bytes (only for volumes with size info)
    pub total_size: i64,
    /// Number of volumes with size information
    pub volumes_with_size: usize,
    /// Number of dangling volumes
    pub dangling_count: usize,
    /// List of unique drivers in use
    pub drivers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_config_builder() {
        let config = VolumeConfig::new("test-volume")
            .driver("local")
            .option("type", "tmpfs")
            .option("device", "tmpfs")
            .label("env", "test");

        assert_eq!(config.name, "test-volume");
        assert_eq!(config.driver, "local");
        assert_eq!(config.driver_opts.get("type"), Some(&"tmpfs".to_string()));
        assert_eq!(config.labels.get("env"), Some(&"test".to_string()));
    }

    #[test]
    fn test_volume_source_mount_string() {
        let named = VolumeSource::named("my-volume");
        assert_eq!(named.to_mount_string("/data"), "my-volume:/data");

        let host_path = VolumeSource::host_path("/host/path");
        assert_eq!(
            host_path.to_mount_string("/container/path"),
            "/host/path:/container/path"
        );

        let tmpfs = VolumeSource::tmpfs();
        assert_eq!(tmpfs.to_mount_string("/tmp"), "tmpfs:/tmp");
    }

    #[test]
    fn test_volume_mount_cli_arg() {
        let mount = VolumeMount::new(VolumeSource::named("data"), "/app/data").read_only();
        assert_eq!(mount.to_cli_arg(), "data:/app/data:ro");

        let mount_with_options =
            VolumeMount::new(VolumeSource::host_path("/host"), "/container").option("cached");
        assert_eq!(mount_with_options.to_cli_arg(), "/host:/container:cached");
    }

    #[test]
    fn test_list_volumes_options() {
        let options = ListVolumesOptions::new()
            .dangling_only()
            .driver("local")
            .label("env=test")
            .name_pattern("test*");

        assert_eq!(options.dangling, Some(true));
        assert_eq!(options.driver, Some("local".to_string()));
        assert!(options.labels.contains(&"env=test".to_string()));
        assert_eq!(options.name, Some("test*".to_string()));
    }

    #[test]
    fn test_volume_config_shortcuts() {
        let tmpfs_config = VolumeConfig::new("tmpfs-vol").tmpfs();
        assert_eq!(
            tmpfs_config.driver_opts.get("type"),
            Some(&"tmpfs".to_string())
        );

        let bind_config = VolumeConfig::new("bind-vol").bind_mount("/host/path");
        assert_eq!(
            bind_config.driver_opts.get("device"),
            Some(&"/host/path".to_string())
        );

        let nfs_config = VolumeConfig::new("nfs-vol").nfs("nfs.example.com", "/export/data");
        assert_eq!(nfs_config.driver_opts.get("type"), Some(&"nfs".to_string()));
        assert!(nfs_config
            .driver_opts
            .get("o")
            .unwrap()
            .contains("nfs.example.com"));
    }

    #[test]
    fn test_remove_volume_options() {
        let options = RemoveVolumeOptions::new().force();
        assert!(options.force);
    }
}
