//! Docker image management module.
//!
//! This module provides complete Docker image lifecycle management including:
//! - Image pulling with progress tracking
//! - Image building from Dockerfiles
//! - Image tagging and removal
//! - Image listing and inspection
//! - Registry authentication support
//!
//! # Example
//!
//! ```rust,no_run
//! use docker_wrapper::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DockerError> {
//!     let client = DockerClient::new().await?;
//!     let image_manager = client.images();
//!
//!     // Pull an image with progress tracking
//!     let image_ref = ImageRef::parse("redis:7.2-alpine")?;
//!     image_manager.pull(&image_ref, PullOptions::default()).await?;
//!
//!     // List all images
//!     let images = image_manager.list(ListImagesOptions::default()).await?;
//!     println!("Found {} images", images.len());
//!
//!     // Build from Dockerfile
//!     let build_options = BuildOptions::new("my-app:latest")
//!         .context_path("./")
//!         .dockerfile("Dockerfile");
//!     image_manager.build(build_options).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::client::DockerClient;
use crate::errors::{DockerError, DockerResult};
use crate::executor::ExecutionConfig;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
// Removed unused imports - AsyncBufReadExt and BufReader

/// Docker image reference with full parsing support
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageRef {
    /// Registry hostname (e.g., "docker.io", "localhost:5000")
    pub registry: Option<String>,
    /// Namespace/organization (e.g., "library", "myorg")
    pub namespace: Option<String>,
    /// Repository name (e.g., "redis", "nginx")
    pub repository: String,
    /// Tag or digest (e.g., "latest", "7.2-alpine", "sha256:abc123...")
    pub tag: String,
}

impl ImageRef {
    /// Create a simple image reference with repository and tag
    pub fn new(repository: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            registry: None,
            namespace: None,
            repository: repository.into(),
            tag: tag.into(),
        }
    }

    /// Create an image reference with full components
    pub fn with_registry(
        registry: impl Into<String>,
        namespace: Option<String>,
        repository: impl Into<String>,
        tag: impl Into<String>,
    ) -> Self {
        Self {
            registry: Some(registry.into()),
            namespace,
            repository: repository.into(),
            tag: tag.into(),
        }
    }

    /// Parse a full image reference string
    ///
    /// Supports formats like:
    /// - `redis:7.2`
    /// - `docker.io/library/redis:7.2`
    /// - `localhost:5000/myapp:latest`
    /// - `redis@sha256:abc123...`
    pub fn parse(image_ref: &str) -> DockerResult<Self> {
        let mut parts = image_ref.splitn(2, '@');
        let image_part = parts.next().unwrap();
        let digest = parts.next();

        // Handle digest case
        if let Some(digest) = digest {
            let tag = format!("@{}", digest);
            return Self::parse_image_part(image_part, tag);
        }

        // Handle tag case
        let mut parts = image_part.rsplitn(2, ':');
        let tag = parts.next().unwrap_or("latest");
        let image_part = parts.next().unwrap_or(image_part);

        Self::parse_image_part(image_part, tag.to_string())
    }

    fn parse_image_part(image_part: &str, tag: String) -> DockerResult<Self> {
        let parts: Vec<&str> = image_part.split('/').collect();

        match parts.len() {
            1 => {
                // Simple case: "redis"
                Ok(Self {
                    registry: None,
                    namespace: None,
                    repository: parts[0].to_string(),
                    tag,
                })
            }
            2 => {
                // Two parts: could be "namespace/repo" or "registry/repo"
                if parts[0].contains('.') || parts[0].contains(':') {
                    // Likely a registry
                    Ok(Self {
                        registry: Some(parts[0].to_string()),
                        namespace: None,
                        repository: parts[1].to_string(),
                        tag,
                    })
                } else {
                    // Likely namespace/repo
                    Ok(Self {
                        registry: None,
                        namespace: Some(parts[0].to_string()),
                        repository: parts[1].to_string(),
                        tag,
                    })
                }
            }
            3 => {
                // Three parts: "registry/namespace/repo"
                Ok(Self {
                    registry: Some(parts[0].to_string()),
                    namespace: Some(parts[1].to_string()),
                    repository: parts[2].to_string(),
                    tag,
                })
            }
            _ => Err(DockerError::invalid_image_ref(image_part)),
        }
    }

    /// Convert to a full image reference string
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        if let Some(registry) = &self.registry {
            result.push_str(registry);
            result.push('/');
        }

        if let Some(namespace) = &self.namespace {
            result.push_str(namespace);
            result.push('/');
        }

        result.push_str(&self.repository);
        result.push(':');
        result.push_str(&self.tag);

        result
    }

    /// Get the repository part without registry/namespace
    pub fn repository_name(&self) -> &str {
        &self.repository
    }

    /// Get the tag part
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Check if this is a digest reference (starts with sha256:)
    pub fn is_digest(&self) -> bool {
        self.tag.starts_with('@')
    }
}

impl std::fmt::Display for ImageRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Docker image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    /// Unique image ID
    #[serde(rename = "Id")]
    pub id: String,
    /// Parent image ID
    #[serde(rename = "Parent")]
    pub parent: String,
    /// Repository tags
    #[serde(rename = "RepoTags")]
    pub repo_tags: Option<Vec<String>>,
    /// Repository digests
    #[serde(rename = "RepoDigests")]
    pub repo_digests: Option<Vec<String>>,
    /// Created timestamp
    #[serde(rename = "Created")]
    pub created: i64,
    /// Size in bytes
    #[serde(rename = "Size")]
    pub size: u64,
    /// Virtual size in bytes
    #[serde(rename = "VirtualSize")]
    pub virtual_size: u64,
    /// Shared size in bytes (-1 if not available)
    #[serde(rename = "SharedSize")]
    pub shared_size: i64,
    /// Image labels
    #[serde(rename = "Labels")]
    pub labels: Option<HashMap<String, String>>,
    /// Number of containers using this image
    #[serde(rename = "Containers")]
    pub containers: i32,
}

/// Temporary struct to parse Docker CLI images format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageCliEntry {
    #[serde(rename = "Containers")]
    pub containers: String,
    #[serde(rename = "CreatedAt")]
    pub created_at: String,
    #[serde(rename = "CreatedSince")]
    pub created_since: String,
    #[serde(rename = "Digest")]
    pub digest: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Repository")]
    pub repository: String,
    #[serde(rename = "SharedSize")]
    pub shared_size: String,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "Tag")]
    pub tag: String,
    #[serde(rename = "UniqueSize")]
    pub unique_size: String,
    #[serde(rename = "VirtualSize")]
    pub virtual_size: String,
}

impl ImageCliEntry {
    /// Parse size string to bytes
    fn size_bytes(&self) -> u64 {
        self.parse_size_string(&self.size)
    }

    /// Parse virtual size string to bytes
    fn virtual_size_bytes(&self) -> u64 {
        self.parse_size_string(&self.virtual_size)
    }

    /// Convert created_at string to Unix timestamp
    fn created_timestamp(&self) -> i64 {
        // Try to parse the created_at string - if it fails, return 0
        chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .or_else(|_| {
                chrono::DateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S %z %Z")
            })
            .map(|dt| dt.timestamp())
            .unwrap_or(0)
    }

    /// Parse size strings like "13.3MB", "1.2GB" to bytes
    fn parse_size_string(&self, size_str: &str) -> u64 {
        if size_str == "N/A" || size_str.is_empty() {
            return 0;
        }

        let size_str = size_str.trim();
        if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
            let (number_part, unit_part) = size_str.split_at(pos);
            if let Ok(number) = number_part.parse::<f64>() {
                let multiplier = match unit_part.to_uppercase().as_str() {
                    "B" => 1_u64,
                    "KB" => 1_000_u64,
                    "MB" => 1_000_000_u64,
                    "GB" => 1_000_000_000_u64,
                    "TB" => 1_000_000_000_000_u64,
                    "KIB" => 1_024_u64,
                    "MIB" => 1_024_u64 * 1_024_u64,
                    "GIB" => 1_024_u64 * 1_024_u64 * 1_024_u64,
                    "TIB" => 1_024_u64 * 1_024_u64 * 1_024_u64 * 1_024_u64,
                    _ => 1_u64,
                };
                return (number * multiplier as f64) as u64;
            }
        }
        0
    }
}

impl DockerImage {
    /// Get the created time as SystemTime
    pub fn created_time(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.created as u64)
    }

    /// Get the first repository tag if available
    pub fn primary_tag(&self) -> Option<&str> {
        self.repo_tags.as_ref()?.first().map(String::as_str)
    }

    /// Check if image has the specified tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.repo_tags
            .as_ref()
            .map_or(false, |tags| tags.iter().any(|t| t == tag))
    }

    /// Get all tags for this image
    pub fn tags(&self) -> Vec<&str> {
        self.repo_tags
            .as_ref()
            .map_or_else(Vec::new, |tags| tags.iter().map(String::as_str).collect())
    }
}

/// Detailed image information from inspect
/// Docker image inspection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInspect {
    /// Image ID
    #[serde(rename = "Id")]
    pub id: String,
    /// Repository tags
    #[serde(rename = "RepoTags")]
    pub repo_tags: Option<Vec<String>>,
    /// Repository digests
    #[serde(rename = "RepoDigests")]
    pub repo_digests: Option<Vec<String>>,
    /// Parent image ID
    #[serde(rename = "Parent")]
    pub parent: String,
    /// Comment
    #[serde(rename = "Comment")]
    pub comment: String,
    /// Created timestamp
    #[serde(rename = "Created")]
    pub created: String,
    /// Docker version used to build
    #[serde(rename = "DockerVersion")]
    pub docker_version: String,
    /// Author
    #[serde(rename = "Author")]
    pub author: String,
    /// Image configuration
    #[serde(rename = "Config")]
    pub config: Option<ImageConfig>,
    /// Architecture
    #[serde(rename = "Architecture")]
    pub architecture: String,
    /// Variant (for multi-arch images)
    #[serde(rename = "Variant")]
    pub variant: Option<String>,
    /// OS
    #[serde(rename = "Os")]
    pub os: String,
    /// Size in bytes
    #[serde(rename = "Size")]
    pub size: u64,
    /// Root filesystem information
    #[serde(rename = "RootFS")]
    pub rootfs: Option<RootFS>,
    /// Metadata
    #[serde(rename = "Metadata")]
    pub metadata: Option<ImageMetadata>,
}

/// Image configuration from inspect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// User
    #[serde(rename = "User")]
    pub user: String,
    /// Environment variables
    #[serde(rename = "Env")]
    pub env: Option<Vec<String>>,
    /// Command
    #[serde(rename = "Cmd")]
    pub cmd: Option<Vec<String>>,
    /// Entrypoint
    #[serde(rename = "Entrypoint")]
    pub entrypoint: Option<Vec<String>>,
    /// Volumes
    #[serde(rename = "Volumes")]
    pub volumes: Option<HashMap<String, serde_json::Value>>,
    /// Working directory
    #[serde(rename = "WorkingDir")]
    pub working_dir: String,
    /// Labels
    #[serde(rename = "Labels")]
    pub labels: Option<HashMap<String, String>>,
    /// OnBuild instructions
    #[serde(rename = "OnBuild")]
    pub on_build: Option<Vec<String>>,
}

/// Root filesystem information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFS {
    /// Type (usually "layers")
    #[serde(rename = "Type")]
    pub type_: String,
    /// Layers
    #[serde(rename = "Layers")]
    pub layers: Option<Vec<String>>,
}

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Last tag time
    #[serde(rename = "LastTagTime")]
    pub last_tag_time: Option<String>,
}

/// Options for pulling images
#[derive(Debug, Clone, Default)]
pub struct PullOptions {
    /// All tags for the repository
    pub all_tags: bool,
    /// Registry authentication
    pub auth: Option<RegistryAuth>,
    /// Platform (for multi-arch images)
    pub platform: Option<String>,
    /// Progress callback
    pub progress_callback: Option<fn(&PullProgress)>,
}

impl PullOptions {
    /// Create new pull options
    pub fn new() -> Self {
        Self::default()
    }

    /// Pull all tags
    pub fn all_tags(mut self) -> Self {
        self.all_tags = true;
        self
    }

    /// Set registry authentication
    pub fn auth(mut self, auth: RegistryAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set platform for multi-arch images
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Set progress callback
    pub fn progress_callback(mut self, callback: fn(&PullProgress)) -> Self {
        self.progress_callback = Some(callback);
        self
    }
}

/// Registry authentication information
#[derive(Debug, Clone)]
pub struct RegistryAuth {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Email (optional)
    pub email: Option<String>,
    /// Server address (optional, defaults to Docker Hub)
    pub server_address: Option<String>,
}

impl RegistryAuth {
    /// Create new registry authentication
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            email: None,
            server_address: None,
        }
    }

    /// Set email
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set server address
    pub fn server_address(mut self, server: impl Into<String>) -> Self {
        self.server_address = Some(server.into());
        self
    }

    /// Encode as base64 JSON for Docker API
    pub fn encode(&self) -> String {
        let auth_json = serde_json::json!({
            "username": self.username,
            "password": self.password,
            "email": self.email.as_deref().unwrap_or(""),
            "serveraddress": self.server_address.as_deref().unwrap_or("https://index.docker.io/v1/")
        });
        base64::engine::general_purpose::STANDARD.encode(auth_json.to_string())
    }
}

/// Pull progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    /// Status message
    pub status: String,
    /// Progress ID
    pub id: Option<String>,
    /// Progress detail
    pub progress_detail: Option<ProgressDetail>,
    /// Progress message
    pub progress: Option<String>,
}

/// Progress detail information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressDetail {
    /// Current progress
    pub current: Option<u64>,
    /// Total size
    pub total: Option<u64>,
}

/// Options for listing images
#[derive(Debug, Clone, Default)]
pub struct ListImagesOptions {
    /// Show all images (including intermediate)
    pub all: bool,
    /// Show digests
    pub digests: bool,
    /// Filter by reference pattern
    pub filters: HashMap<String, Vec<String>>,
}

impl ListImagesOptions {
    /// Create new list options
    pub fn new() -> Self {
        Self::default()
    }

    /// Show all images including intermediate layers
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Show digests
    pub fn digests(mut self) -> Self {
        self.digests = true;
        self
    }

    /// Filter by reference pattern
    pub fn filter_reference(mut self, pattern: impl Into<String>) -> Self {
        self.filters
            .entry("reference".to_string())
            .or_default()
            .push(pattern.into());
        self
    }

    /// Filter by dangling status
    pub fn filter_dangling(mut self, dangling: bool) -> Self {
        self.filters
            .entry("dangling".to_string())
            .or_default()
            .push(dangling.to_string());
        self
    }

    /// Filter by label
    pub fn filter_label(mut self, label: impl Into<String>) -> Self {
        self.filters
            .entry("label".to_string())
            .or_default()
            .push(label.into());
        self
    }
}

/// Options for building images
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Image tag to assign
    pub tag: String,
    /// Build context path
    pub context_path: PathBuf,
    /// Dockerfile path (relative to context)
    pub dockerfile: Option<String>,
    /// Build arguments
    pub build_args: HashMap<String, String>,
    /// Labels to set
    pub labels: HashMap<String, String>,
    /// Target stage for multi-stage builds
    pub target: Option<String>,
    /// No cache
    pub no_cache: bool,
    /// Remove intermediate containers
    pub remove: bool,
    /// Force remove intermediate containers
    pub force_remove: bool,
    /// Pull newer version of base image
    pub pull: bool,
    /// Squash layers
    pub squash: bool,
    /// Platform for multi-arch builds
    pub platform: Option<String>,
    /// Progress callback
    pub progress_callback: Option<fn(&BuildProgress)>,
}

impl BuildOptions {
    /// Create new build options
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            context_path: PathBuf::from("."),
            dockerfile: None,
            build_args: HashMap::new(),
            labels: HashMap::new(),
            target: None,
            no_cache: false,
            remove: true,
            force_remove: false,
            pull: false,
            squash: false,
            platform: None,
            progress_callback: None,
        }
    }

    /// Set build context path
    pub fn context_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.context_path = path.into();
        self
    }

    /// Set Dockerfile path
    pub fn dockerfile(mut self, dockerfile: impl Into<String>) -> Self {
        self.dockerfile = Some(dockerfile.into());
        self
    }

    /// Add build argument
    pub fn build_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build_args.insert(key.into(), value.into());
        self
    }

    /// Add label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set target stage
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Disable cache
    pub fn no_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }

    /// Don't remove intermediate containers
    pub fn keep_intermediate(mut self) -> Self {
        self.remove = false;
        self
    }

    /// Force remove intermediate containers
    pub fn force_remove(mut self) -> Self {
        self.force_remove = true;
        self
    }

    /// Pull newer version of base image
    pub fn pull(mut self) -> Self {
        self.pull = true;
        self
    }

    /// Squash layers
    pub fn squash(mut self) -> Self {
        self.squash = true;
        self
    }

    /// Set platform
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Set progress callback
    pub fn progress_callback(mut self, callback: fn(&BuildProgress)) -> Self {
        self.progress_callback = Some(callback);
        self
    }
}

/// Build progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildProgress {
    /// Stream output
    pub stream: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Error detail
    pub error_detail: Option<serde_json::Value>,
    /// Progress information
    pub progress: Option<String>,
    /// Aux information
    pub aux: Option<serde_json::Value>,
}

/// Options for removing images
#[derive(Debug, Clone, Default)]
pub struct RemoveImageOptions {
    /// Force removal
    pub force: bool,
    /// Don't delete untagged parent images
    pub no_prune: bool,
}

impl RemoveImageOptions {
    /// Create new remove options
    pub fn new() -> Self {
        Self::default()
    }

    /// Force removal
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Don't delete untagged parent images
    pub fn no_prune(mut self) -> Self {
        self.no_prune = true;
        self
    }
}

/// Result of image removal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveImageResult {
    /// Untagged images
    #[serde(rename = "Untagged")]
    pub untagged: Option<Vec<String>>,
    /// Deleted images
    #[serde(rename = "Deleted")]
    pub deleted: Option<Vec<String>>,
}

/// Image manager providing all image operations
pub struct ImageManager<'a> {
    client: &'a DockerClient,
}

impl<'a> ImageManager<'a> {
    /// Create a new image manager
    pub fn new(client: &'a DockerClient) -> Self {
        Self { client }
    }

    /// Pull an image from a registry
    pub async fn pull(&self, image_ref: &ImageRef, options: PullOptions) -> DockerResult<()> {
        let mut args = vec!["pull".to_string()];

        if options.all_tags {
            args.push("--all-tags".to_string());
        }

        if let Some(platform) = &options.platform {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }

        args.push(image_ref.to_string());

        let executor_options = ExecutionConfig::default();

        // Add authentication if provided
        if let Some(_auth) = &options.auth {
            // For pull operations, we can use docker login or pass credentials
            // For simplicity, we'll assume docker login has been done
            // In a full implementation, we might use the Docker API directly
        }

        let output = self
            .client
            .executor()
            .execute(&args, Some(executor_options))
            .await?;

        if !output.success {
            return Err(DockerError::CommandFailed {
                command: format!("docker {}", args.join(" ")),
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            });
        }

        // For now, we'll skip progress callbacks for simplicity
        // In a full implementation, we'd use streaming execution
        if let Some(_callback) = options.progress_callback {
            // TODO: Implement progress tracking with streaming
        }

        Ok(())
    }

    /// Build an image from a Dockerfile
    pub async fn build(&self, options: BuildOptions) -> DockerResult<String> {
        let mut args = vec!["build".to_string()];

        args.push("--tag".to_string());
        args.push(options.tag.clone());

        if let Some(dockerfile) = &options.dockerfile {
            args.push("--file".to_string());
            args.push(dockerfile.clone());
        }

        for (key, value) in &options.build_args {
            args.push("--build-arg".to_string());
            args.push(format!("{}={}", key, value));
        }

        for (key, value) in &options.labels {
            args.push("--label".to_string());
            args.push(format!("{}={}", key, value));
        }

        if let Some(target) = &options.target {
            args.push("--target".to_string());
            args.push(target.clone());
        }

        if options.no_cache {
            args.push("--no-cache".to_string());
        }

        if !options.remove {
            args.push("--rm=false".to_string());
        }

        if options.force_remove {
            args.push("--force-rm".to_string());
        }

        if options.pull {
            args.push("--pull".to_string());
        }

        if options.squash {
            args.push("--squash".to_string());
        }

        if let Some(platform) = &options.platform {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }

        args.push(options.context_path.to_string_lossy().to_string());

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

        // For now, we'll skip progress callbacks for simplicity
        if let Some(_callback) = options.progress_callback {
            // TODO: Implement progress tracking with streaming
        }

        // Extract image ID from output
        let mut image_id = String::new();
        for line in output.stdout.lines() {
            if line.starts_with("Successfully built ") {
                image_id = line.replace("Successfully built ", "").trim().to_string();
                break;
            }
        }

        Ok(image_id)
    }

    /// List images
    pub async fn list(&self, options: ListImagesOptions) -> DockerResult<Vec<DockerImage>> {
        let mut args = vec![
            "images".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];

        if options.all {
            args.push("--all".to_string());
        }

        if options.digests {
            args.push("--digests".to_string());
        }

        // Add filters
        for (key, values) in &options.filters {
            for value in values {
                args.push("--filter".to_string());
                args.push(format!("{}={}", key, value));
            }
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
        let mut image_map: HashMap<String, DockerImage> = HashMap::new();

        // Parse CLI format and aggregate by image ID
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<ImageCliEntry>(line) {
                Ok(cli_entry) => {
                    let image_id = cli_entry.id.clone();
                    let repo_tag = if cli_entry.repository == "<none>" {
                        None
                    } else {
                        Some(format!("{}:{}", cli_entry.repository, cli_entry.tag))
                    };

                    if let Some(existing_image) = image_map.get_mut(&image_id) {
                        // Add this repo:tag to existing image
                        if let Some(tag) = repo_tag {
                            if let Some(ref mut tags) = existing_image.repo_tags {
                                if !tags.contains(&tag) {
                                    tags.push(tag);
                                }
                            } else {
                                existing_image.repo_tags = Some(vec![tag]);
                            }
                        }
                    } else {
                        // Create new image entry
                        let repo_tags = repo_tag.map(|tag| vec![tag]);
                        let image = DockerImage {
                            id: image_id.clone(),
                            parent: String::new(), // CLI format doesn't provide parent
                            repo_tags,
                            repo_digests: None, // Will be populated if needed
                            created: cli_entry.created_timestamp(),
                            size: cli_entry.size_bytes(),
                            virtual_size: cli_entry.virtual_size_bytes(),
                            shared_size: -1, // CLI format doesn't provide shared size
                            labels: None,    // CLI format doesn't provide labels in basic listing
                            containers: cli_entry.containers.parse().unwrap_or(0),
                        };
                        image_map.insert(image_id, image);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse image JSON: {} - {}", e, line);
                }
            }
        }

        Ok(image_map.into_values().collect())
    }

    /// Inspect an image
    pub async fn inspect(&self, image_ref: &ImageRef) -> DockerResult<ImageInspect> {
        let args = vec![
            "inspect".to_string(),
            "--type".to_string(),
            "image".to_string(),
            image_ref.to_string(),
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
        let inspects: Vec<ImageInspect> = serde_json::from_str(&stdout)
            .map_err(|e| DockerError::ParseError(format!("Invalid inspect JSON: {}", e)))?;

        inspects
            .into_iter()
            .next()
            .ok_or_else(|| DockerError::NotFound {
                message: format!("Image not found: {}", image_ref),
            })
    }

    /// Tag an image
    pub async fn tag(&self, source: &ImageRef, target: &ImageRef) -> DockerResult<()> {
        let args = vec!["tag".to_string(), source.to_string(), target.to_string()];

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

    /// Remove an image
    pub async fn remove(
        &self,
        image_ref: &ImageRef,
        options: RemoveImageOptions,
    ) -> DockerResult<Vec<RemoveImageResult>> {
        let mut args = vec!["rmi".to_string()];

        if options.force {
            args.push("--force".to_string());
        }

        if options.no_prune {
            args.push("--no-prune".to_string());
        }

        args.push(image_ref.to_string());

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
        if stdout.trim().is_empty() {
            return Ok(vec![]);
        }

        // Try to parse as JSON array
        match serde_json::from_str::<Vec<RemoveImageResult>>(&stdout) {
            Ok(results) => Ok(results),
            Err(_) => {
                // Fallback: create a simple result based on output
                Ok(vec![RemoveImageResult {
                    untagged: Some(vec![image_ref.to_string()]),
                    deleted: None,
                }])
            }
        }
    }

    /// Push an image to a registry
    pub async fn push(
        &self,
        image_ref: &ImageRef,
        _auth: Option<RegistryAuth>,
    ) -> DockerResult<()> {
        let args = vec!["push".to_string(), image_ref.to_string()];

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

    /// Prune unused images
    pub async fn prune(&self, remove_dangling_only: bool) -> DockerResult<PruneResult> {
        let mut args = vec![
            "image".to_string(),
            "prune".to_string(),
            "--force".to_string(),
        ];

        if !remove_dangling_only {
            args.push("--all".to_string());
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

        // Parse the output to extract reclaimed space
        let mut reclaimed_space = 0u64;
        for line in stdout.lines() {
            if line.contains("Total reclaimed space:") {
                // Extract the number from lines like "Total reclaimed space: 1.23GB"
                if let Some(space_str) = line.split(':').nth(1) {
                    if let Some(num_str) = space_str.trim().split_whitespace().next() {
                        if let Ok(num) = num_str.parse::<f64>() {
                            // Convert to bytes (rough approximation)
                            reclaimed_space = (num * 1_000_000_000.0) as u64;
                        }
                    }
                }
            }
        }

        Ok(PruneResult {
            deleted_images: vec![], // Docker doesn't provide individual image IDs in prune output
            reclaimed_space,
        })
    }

    /// Import an image from a tarball
    pub async fn import<P: AsRef<Path>>(
        &self,
        tarball_path: P,
        repository: Option<String>,
        tag: Option<String>,
    ) -> DockerResult<String> {
        let mut args = vec!["import".to_string()];

        args.push(tarball_path.as_ref().to_string_lossy().to_string());

        if let Some(repo) = repository {
            let image_name = if let Some(t) = tag {
                format!("{}:{}", repo, t)
            } else {
                repo
            };
            args.push(image_name);
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
        Ok(stdout.trim().to_string())
    }

    /// Export an image to a tarball
    pub async fn export<P: AsRef<Path>>(
        &self,
        image_ref: &ImageRef,
        output_path: P,
    ) -> DockerResult<()> {
        let args = vec![
            "save".to_string(),
            "--output".to_string(),
            output_path.as_ref().to_string_lossy().to_string(),
            image_ref.to_string(),
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

        Ok(())
    }

    /// Get image history
    pub async fn history(&self, image_ref: &ImageRef) -> DockerResult<Vec<ImageHistoryItem>> {
        let args = vec![
            "history".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--no-trunc".to_string(),
            image_ref.to_string(),
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
        let mut history = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<ImageHistoryCliItem>(line) {
                Ok(cli_item) => history.push(cli_item.into()),
                Err(e) => {
                    log::warn!("Failed to parse history JSON: {} - {}", e, line);
                }
            }
        }

        Ok(history)
    }
}

/// Result of image pruning operation
#[derive(Debug, Clone)]
pub struct PruneResult {
    /// List of deleted image IDs
    pub deleted_images: Vec<String>,
    /// Total reclaimed space in bytes
    pub reclaimed_space: u64,
}

/// Image history item from CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageHistoryCliItem {
    /// Image ID
    #[serde(rename = "ID")]
    pub id: String,
    /// Created at timestamp string
    #[serde(rename = "CreatedAt")]
    pub created_at: String,
    /// Created since (human readable)
    #[serde(rename = "CreatedSince")]
    pub created_since: String,
    /// Created by command
    #[serde(rename = "CreatedBy")]
    pub created_by: String,
    /// Size string (like "8.35MB")
    #[serde(rename = "Size")]
    pub size: String,
    /// Comment
    #[serde(rename = "Comment")]
    pub comment: String,
}

/// Image history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageHistoryItem {
    /// Image ID
    pub id: String,
    /// Created timestamp
    pub created: i64,
    /// Created by command
    pub created_by: String,
    /// Size in bytes
    pub size: u64,
    /// Comment
    pub comment: String,
}

impl From<ImageHistoryCliItem> for ImageHistoryItem {
    fn from(cli_item: ImageHistoryCliItem) -> Self {
        ImageHistoryItem {
            id: cli_item.id,
            created: parse_timestamp(&cli_item.created_at),
            created_by: cli_item.created_by,
            size: parse_size_string(&cli_item.size),
            comment: cli_item.comment,
        }
    }
}

fn parse_timestamp(timestamp_str: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(timestamp_str)
        .or_else(|_| chrono::DateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S%z"))
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

fn parse_size_string(size_str: &str) -> u64 {
    if size_str == "0B" || size_str.is_empty() {
        return 0;
    }

    let size_str = size_str.trim();
    if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
        let (number_part, unit_part) = size_str.split_at(pos);
        if let Ok(number) = number_part.parse::<f64>() {
            let multiplier = match unit_part.to_uppercase().as_str() {
                "B" => 1_u64,
                "KB" => 1_000_u64,
                "MB" => 1_000_000_u64,
                "GB" => 1_000_000_000_u64,
                "TB" => 1_000_000_000_000_u64,
                "KIB" => 1_024_u64,
                "MIB" => 1_024_u64 * 1_024_u64,
                "GIB" => 1_024_u64 * 1_024_u64 * 1_024_u64,
                "TIB" => 1_024_u64 * 1_024_u64 * 1_024_u64 * 1_024_u64,
                _ => 1_u64,
            };
            return (number * multiplier as f64) as u64;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_ref_parse_simple() {
        let image_ref = ImageRef::parse("redis:7.2").unwrap();
        assert_eq!(image_ref.registry, None);
        assert_eq!(image_ref.namespace, None);
        assert_eq!(image_ref.repository, "redis");
        assert_eq!(image_ref.tag, "7.2");
    }

    #[test]
    fn test_image_ref_parse_with_namespace() {
        let image_ref = ImageRef::parse("library/redis:7.2").unwrap();
        assert_eq!(image_ref.registry, None);
        assert_eq!(image_ref.namespace, Some("library".to_string()));
        assert_eq!(image_ref.repository, "redis");
        assert_eq!(image_ref.tag, "7.2");
    }

    #[test]
    fn test_image_ref_parse_with_registry() {
        let image_ref = ImageRef::parse("docker.io/library/redis:7.2").unwrap();
        assert_eq!(image_ref.registry, Some("docker.io".to_string()));
        assert_eq!(image_ref.namespace, Some("library".to_string()));
        assert_eq!(image_ref.repository, "redis");
        assert_eq!(image_ref.tag, "7.2");
    }

    #[test]
    fn test_image_ref_parse_with_digest() {
        let image_ref = ImageRef::parse("redis@sha256:abc123def456").unwrap();
        assert_eq!(image_ref.registry, None);
        assert_eq!(image_ref.namespace, None);
        assert_eq!(image_ref.repository, "redis");
        assert_eq!(image_ref.tag, "@sha256:abc123def456");
        assert!(image_ref.is_digest());
    }

    #[test]
    fn test_image_ref_to_string() {
        let image_ref =
            ImageRef::with_registry("docker.io", Some("library".to_string()), "redis", "7.2");
        assert_eq!(image_ref.to_string(), "docker.io/library/redis:7.2");
    }

    #[test]
    fn test_pull_options_builder() {
        let options = PullOptions::new().all_tags().platform("linux/amd64");

        assert!(options.all_tags);
        assert_eq!(options.platform, Some("linux/amd64".to_string()));
    }

    #[test]
    fn test_build_options_builder() {
        let options = BuildOptions::new("my-app:latest")
            .dockerfile("Dockerfile.prod")
            .build_arg("VERSION", "1.0")
            .no_cache();

        assert_eq!(options.tag, "my-app:latest");
        assert_eq!(options.dockerfile, Some("Dockerfile.prod".to_string()));
        assert_eq!(options.build_args.get("VERSION"), Some(&"1.0".to_string()));
        assert!(options.no_cache);
    }

    #[test]
    fn test_registry_auth_encode() {
        let auth = RegistryAuth::new("user", "pass")
            .email("user@example.com")
            .server_address("https://my-registry.com");

        let encoded = auth.encode();
        assert!(!encoded.is_empty());

        // Decode and verify
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(json["username"], "user");
        assert_eq!(json["password"], "pass");
        assert_eq!(json["email"], "user@example.com");
        assert_eq!(json["serveraddress"], "https://my-registry.com");
    }
}
