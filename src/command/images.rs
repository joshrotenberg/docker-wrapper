//! Docker Images Command Implementation
//!
//! This module provides a comprehensive implementation of the `docker images` command,
//! supporting all native Docker images options for listing local images.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use docker_wrapper::ImagesCommand;
//! use docker_wrapper::DockerCommand;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // List all images
//!     let images_cmd = ImagesCommand::new();
//!     let output = images_cmd.execute().await?;
//!     println!("Images listed: {}", output.success());
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ```no_run
//! use docker_wrapper::ImagesCommand;
//! use docker_wrapper::DockerCommand;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // List images with filtering and JSON format
//!     let images_cmd = ImagesCommand::new()
//!         .repository("nginx")
//!         .all()
//!         .filter("dangling=false")
//!         .format_json()
//!         .digests()
//!         .no_trunc();
//!
//!     let output = images_cmd.execute().await?;
//!     println!("Filtered images: {}", output.success());
//!     Ok(())
//! }
//! ```

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Docker Images Command Builder
///
/// Implements the `docker images` command for listing local Docker images.
///
/// # Docker Images Overview
///
/// The images command lists Docker images stored locally on the system. It supports:
/// - Repository and tag filtering
/// - Multiple output formats (table, JSON, custom templates)
/// - Image metadata display (digests, sizes, creation dates)
/// - Advanced filtering by various criteria
/// - Quiet mode for scripts
///
/// # Image Information
///
/// Each image entry typically includes:
/// - Repository name
/// - Tag
/// - Image ID
/// - Creation date
/// - Size
/// - Optionally: digests, intermediate layers
///
/// # Examples
///
/// ```no_run
/// use docker_wrapper::ImagesCommand;
/// use docker_wrapper::DockerCommand;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // List all nginx images
///     let output = ImagesCommand::new()
///         .repository("nginx")
///         .execute()
///         .await?;
///
///     println!("Images success: {}", output.success());
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ImagesCommand {
    /// Optional repository filter (e.g., "nginx", "nginx:alpine")
    repository: Option<String>,
    /// Show all images (including intermediate images)
    all: bool,
    /// Show digests
    digests: bool,
    /// Filter output based on conditions
    filters: Vec<String>,
    /// Output format
    format: Option<String>,
    /// Don't truncate output
    no_trunc: bool,
    /// Only show image IDs
    quiet: bool,
    /// List multi-platform images as a tree (experimental)
    tree: bool,
    /// Command executor for handling raw arguments and execution
    pub executor: CommandExecutor,
}

/// Represents a Docker image from the output
#[derive(Debug, Clone, PartialEq)]
pub struct ImageInfo {
    /// Repository name
    pub repository: String,
    /// Tag
    pub tag: String,
    /// Image ID
    pub image_id: String,
    /// Creation date/time
    pub created: String,
    /// Image size
    pub size: String,
    /// Digest (if available)
    pub digest: Option<String>,
}

/// Output from the images command with parsed image information
#[derive(Debug, Clone)]
pub struct ImagesOutput {
    /// Raw command output
    pub output: CommandOutput,
    /// Parsed image information (if output is parseable)
    pub images: Vec<ImageInfo>,
}

impl ImagesCommand {
    /// Create a new `ImagesCommand` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            repository: None,
            all: false,
            digests: false,
            filters: Vec::new(),
            format: None,
            no_trunc: false,
            quiet: false,
            tree: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Filter images by repository name (and optionally tag)
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository name (e.g., "nginx", "nginx:alpine", "ubuntu:20.04")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .repository("nginx:alpine");
    /// ```
    #[must_use]
    pub fn repository<S: Into<String>>(mut self, repository: S) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Show all images (including intermediate images)
    ///
    /// By default, Docker hides intermediate images. This option shows them all.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .all();
    /// ```
    #[must_use]
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Show digests
    ///
    /// Displays the digest (SHA256 hash) for each image.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .digests();
    /// ```
    #[must_use]
    pub fn digests(mut self) -> Self {
        self.digests = true;
        self
    }

    /// Add a filter condition
    ///
    /// Common filters:
    /// - `dangling=true|false` - Show dangling images
    /// - `label=<key>` or `label=<key>=<value>` - Filter by label
    /// - `before=<image>` - Images created before this image
    /// - `since=<image>` - Images created since this image
    /// - `reference=<pattern>` - Filter by repository name pattern
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .filter("dangling=true")
    ///     .filter("label=maintainer=nginx");
    /// ```
    #[must_use]
    pub fn filter<S: Into<String>>(mut self, filter: S) -> Self {
        self.filters.push(filter.into());
        self
    }

    /// Add multiple filter conditions
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .filters(vec!["dangling=false", "label=version=latest"]);
    /// ```
    #[must_use]
    pub fn filters<I, S>(mut self, filters: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.filters
            .extend(filters.into_iter().map(std::convert::Into::into));
        self
    }

    /// Set custom output format
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .format("table {{.Repository}}:{{.Tag}}\t{{.Size}}");
    /// ```
    #[must_use]
    pub fn format<S: Into<String>>(mut self, format: S) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Format output as table (default)
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .format_table();
    /// ```
    #[must_use]
    pub fn format_table(mut self) -> Self {
        self.format = Some("table".to_string());
        self
    }

    /// Format output as JSON
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .format_json();
    /// ```
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some("json".to_string());
        self
    }

    /// Don't truncate output
    ///
    /// By default, Docker truncates long values. This shows full values.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .no_trunc();
    /// ```
    #[must_use]
    pub fn no_trunc(mut self) -> Self {
        self.no_trunc = true;
        self
    }

    /// Only show image IDs
    ///
    /// Useful for scripting and automation.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .quiet();
    /// ```
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// List multi-platform images as a tree (experimental)
    ///
    /// This is an experimental Docker feature for displaying multi-platform images.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .tree();
    /// ```
    #[must_use]
    pub fn tree(mut self) -> Self {
        self.tree = true;
        self
    }

    /// Build the command arguments
    ///
    /// This method constructs the complete argument list for the docker images command.
    fn build_command_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add all flag
        if self.all {
            args.push("--all".to_string());
        }

        // Add digests flag
        if self.digests {
            args.push("--digests".to_string());
        }

        // Add filters
        for filter in &self.filters {
            args.push("--filter".to_string());
            args.push(filter.clone());
        }

        // Add format
        if let Some(ref format) = self.format {
            args.push("--format".to_string());
            args.push(format.clone());
        }

        // Add no-trunc flag
        if self.no_trunc {
            args.push("--no-trunc".to_string());
        }

        // Add quiet flag
        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add tree flag
        if self.tree {
            args.push("--tree".to_string());
        }

        // Add repository filter (must be last)
        if let Some(ref repository) = self.repository {
            args.push(repository.clone());
        }

        args
    }

    /// Parse the output to extract image information
    ///
    /// This attempts to parse the docker images output into structured data.
    fn parse_output(&self, output: &CommandOutput) -> Vec<ImageInfo> {
        if self.quiet {
            // In quiet mode, output is just image IDs
            return output
                .stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| ImageInfo {
                    repository: "<unknown>".to_string(),
                    tag: "<unknown>".to_string(),
                    image_id: line.trim().to_string(),
                    created: "<unknown>".to_string(),
                    size: "<unknown>".to_string(),
                    digest: None,
                })
                .collect();
        }

        if let Some(ref format) = self.format {
            if format == "json" {
                return Self::parse_json_output(&output.stdout);
            }
        }

        // Parse table format (default)
        self.parse_table_output(&output.stdout)
    }

    /// Parse JSON format output
    fn parse_json_output(stdout: &str) -> Vec<ImageInfo> {
        let mut images = Vec::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<Value>(line) {
                if let Some(obj) = json.as_object() {
                    let repository = obj
                        .get("Repository")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>")
                        .to_string();
                    let tag = obj
                        .get("Tag")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>")
                        .to_string();
                    let image_id = obj
                        .get("ID")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let created = obj
                        .get("CreatedAt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let size = obj
                        .get("Size")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let digest = obj.get("Digest").and_then(|v| v.as_str()).map(String::from);

                    images.push(ImageInfo {
                        repository,
                        tag,
                        image_id,
                        created,
                        size,
                        digest,
                    });
                }
            }
        }

        images
    }

    /// Parse table format output
    fn parse_table_output(&self, stdout: &str) -> Vec<ImageInfo> {
        let mut images = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.is_empty() {
            return images;
        }

        // Skip header line if present
        let data_lines = if lines[0].starts_with("REPOSITORY") {
            &lines[1..]
        } else {
            &lines[..]
        };

        for line in data_lines {
            if line.trim().is_empty() {
                continue;
            }

            // Split by whitespace, but handle multi-word fields like "2 days ago"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let repository = parts[0].to_string();
                let tag = parts[1].to_string();
                let image_id = parts[2].to_string();

                // Handle multi-word created field and size
                let (created, size, digest) = if self.digests && parts.len() >= 7 {
                    // With digests: REPO TAG IMAGE_ID DIGEST CREATED... SIZE
                    let digest = Some(parts[3].to_string());
                    let created_parts = &parts[4..parts.len() - 1];
                    let created = created_parts.join(" ");
                    let size = parts[parts.len() - 1].to_string();
                    (created, size, digest)
                } else if parts.len() >= 5 {
                    // Without digests: REPO TAG IMAGE_ID CREATED... SIZE
                    let created_parts = &parts[3..parts.len() - 1];
                    let created = created_parts.join(" ");
                    let size = parts[parts.len() - 1].to_string();
                    (created, size, None)
                } else {
                    (String::new(), String::new(), None)
                };

                images.push(ImageInfo {
                    repository,
                    tag,
                    image_id,
                    created,
                    size,
                    digest,
                });
            }
        }

        images
    }

    /// Get the repository filter if set
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new().repository("nginx");
    /// assert_eq!(images_cmd.get_repository(), Some("nginx"));
    /// ```
    #[must_use]
    pub fn get_repository(&self) -> Option<&str> {
        self.repository.as_deref()
    }

    /// Check if showing all images
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new().all();
    /// assert!(images_cmd.is_all());
    /// ```
    #[must_use]
    pub fn is_all(&self) -> bool {
        self.all
    }

    /// Check if showing digests
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new().digests();
    /// assert!(images_cmd.is_digests());
    /// ```
    #[must_use]
    pub fn is_digests(&self) -> bool {
        self.digests
    }

    /// Check if quiet mode is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new().quiet();
    /// assert!(images_cmd.is_quiet());
    /// ```
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Check if no-trunc is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new().no_trunc();
    /// assert!(images_cmd.is_no_trunc());
    /// ```
    #[must_use]
    pub fn is_no_trunc(&self) -> bool {
        self.no_trunc
    }

    /// Check if tree mode is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new().tree();
    /// assert!(images_cmd.is_tree());
    /// ```
    #[must_use]
    pub fn is_tree(&self) -> bool {
        self.tree
    }

    /// Get the current filters
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new()
    ///     .filter("dangling=true");
    /// assert_eq!(images_cmd.get_filters(), &["dangling=true"]);
    /// ```
    #[must_use]
    pub fn get_filters(&self) -> &[String] {
        &self.filters
    }

    /// Get the format if set
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::ImagesCommand;
    ///
    /// let images_cmd = ImagesCommand::new().format_json();
    /// assert_eq!(images_cmd.get_format(), Some("json"));
    /// ```
    #[must_use]
    pub fn get_format(&self) -> Option<&str> {
        self.format.as_deref()
    }

    /// Get a reference to the command executor
    #[must_use]
    pub fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    /// Get a mutable reference to the command executor
    #[must_use]
    pub fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
}

impl Default for ImagesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ImagesOutput {
    /// Check if the command was successful
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::ImagesCommand;
    /// # use docker_wrapper::DockerCommand;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = ImagesCommand::new().execute().await?;
    /// if output.success() {
    ///     println!("Images listed successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the number of images found
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::ImagesCommand;
    /// # use docker_wrapper::DockerCommand;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = ImagesCommand::new().execute().await?;
    /// println!("Found {} images", output.image_count());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Get image IDs only
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::ImagesCommand;
    /// # use docker_wrapper::DockerCommand;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = ImagesCommand::new().execute().await?;
    /// let ids = output.image_ids();
    /// println!("Image IDs: {:?}", ids);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn image_ids(&self) -> Vec<&str> {
        self.images
            .iter()
            .map(|img| img.image_id.as_str())
            .collect()
    }

    /// Filter images by repository name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::ImagesCommand;
    /// # use docker_wrapper::DockerCommand;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = ImagesCommand::new().execute().await?;
    /// let nginx_images = output.filter_by_repository("nginx");
    /// println!("Nginx images: {}", nginx_images.len());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn filter_by_repository(&self, repository: &str) -> Vec<&ImageInfo> {
        self.images
            .iter()
            .filter(|img| img.repository == repository)
            .collect()
    }

    /// Check if output is empty (no images)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use docker_wrapper::ImagesCommand;
    /// # use docker_wrapper::DockerCommand;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = ImagesCommand::new().execute().await?;
    /// if output.is_empty() {
    ///     println!("No images found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

#[async_trait]
impl DockerCommand for ImagesCommand {
    type Output = ImagesOutput;

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_command_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let output = self.execute_command(args).await?;

        let images = self.parse_output(&output);

        Ok(ImagesOutput { output, images })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_images_command_basic() {
        let images_cmd = ImagesCommand::new();
        let args = images_cmd.build_command_args();

        assert!(args.is_empty()); // No arguments for basic images command
        assert!(!images_cmd.is_all());
        assert!(!images_cmd.is_digests());
        assert!(!images_cmd.is_quiet());
        assert!(!images_cmd.is_no_trunc());
        assert!(!images_cmd.is_tree());
        assert_eq!(images_cmd.get_repository(), None);
        assert_eq!(images_cmd.get_format(), None);
        assert!(images_cmd.get_filters().is_empty());
    }

    #[test]
    fn test_images_command_with_repository() {
        let images_cmd = ImagesCommand::new().repository("nginx:alpine");
        let args = images_cmd.build_command_args();

        assert!(args.contains(&"nginx:alpine".to_string()));
        assert_eq!(args.last(), Some(&"nginx:alpine".to_string()));
        assert_eq!(images_cmd.get_repository(), Some("nginx:alpine"));
    }

    #[test]
    fn test_images_command_with_all_flags() {
        let images_cmd = ImagesCommand::new()
            .all()
            .digests()
            .no_trunc()
            .quiet()
            .tree();

        let args = images_cmd.build_command_args();

        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"--digests".to_string()));
        assert!(args.contains(&"--no-trunc".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--tree".to_string()));

        assert!(images_cmd.is_all());
        assert!(images_cmd.is_digests());
        assert!(images_cmd.is_no_trunc());
        assert!(images_cmd.is_quiet());
        assert!(images_cmd.is_tree());
    }

    #[test]
    fn test_images_command_with_filters() {
        let images_cmd = ImagesCommand::new()
            .filter("dangling=true")
            .filter("label=maintainer=nginx")
            .filters(vec!["before=alpine:latest", "since=ubuntu:20.04"]);

        let args = images_cmd.build_command_args();

        assert!(args.contains(&"--filter".to_string()));
        assert!(args.contains(&"dangling=true".to_string()));
        assert!(args.contains(&"label=maintainer=nginx".to_string()));
        assert!(args.contains(&"before=alpine:latest".to_string()));
        assert!(args.contains(&"since=ubuntu:20.04".to_string()));

        let filters = images_cmd.get_filters();
        assert_eq!(filters.len(), 4);
        assert!(filters.contains(&"dangling=true".to_string()));
    }

    #[test]
    fn test_images_command_with_format() {
        let images_cmd = ImagesCommand::new().format_json();
        let args = images_cmd.build_command_args();

        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
        assert_eq!(images_cmd.get_format(), Some("json"));
    }

    #[test]
    fn test_images_command_custom_format() {
        let custom_format = "table {{.Repository}}:{{.Tag}}\t{{.Size}}";
        let images_cmd = ImagesCommand::new().format(custom_format);
        let args = images_cmd.build_command_args();

        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&custom_format.to_string()));
        assert_eq!(images_cmd.get_format(), Some(custom_format));
    }

    #[test]
    fn test_images_command_all_options() {
        let images_cmd = ImagesCommand::new()
            .repository("ubuntu")
            .all()
            .digests()
            .filter("dangling=false")
            .format_table()
            .no_trunc()
            .quiet();

        let args = images_cmd.build_command_args();

        // Repository should be last
        assert_eq!(args.last(), Some(&"ubuntu".to_string()));

        // All options should be present
        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"--digests".to_string()));
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.contains(&"dangling=false".to_string()));
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"table".to_string()));
        assert!(args.contains(&"--no-trunc".to_string()));
        assert!(args.contains(&"--quiet".to_string()));

        // Verify helper methods
        assert_eq!(images_cmd.get_repository(), Some("ubuntu"));
        assert!(images_cmd.is_all());
        assert!(images_cmd.is_digests());
        assert!(images_cmd.is_no_trunc());
        assert!(images_cmd.is_quiet());
        assert_eq!(images_cmd.get_format(), Some("table"));
        assert_eq!(images_cmd.get_filters(), &["dangling=false"]);
    }

    #[test]
    fn test_images_command_default() {
        let images_cmd = ImagesCommand::default();
        assert_eq!(images_cmd.get_repository(), None);
        assert!(!images_cmd.is_all());
    }

    #[test]
    fn test_image_info_creation() {
        let image = ImageInfo {
            repository: "nginx".to_string(),
            tag: "alpine".to_string(),
            image_id: "abc123456789".to_string(),
            created: "2 days ago".to_string(),
            size: "16.1MB".to_string(),
            digest: Some("sha256:abc123".to_string()),
        };

        assert_eq!(image.repository, "nginx");
        assert_eq!(image.tag, "alpine");
        assert_eq!(image.image_id, "abc123456789");
        assert_eq!(image.digest, Some("sha256:abc123".to_string()));
    }

    #[test]
    fn test_parse_json_output() {
        let json_output = r#"{"Containers":"N/A","CreatedAt":"2023-01-01T00:00:00Z","CreatedSince":"2 days ago","Digest":"sha256:abc123","ID":"sha256:def456","Repository":"nginx","SharedSize":"N/A","Size":"16.1MB","Tag":"alpine","UniqueSize":"N/A","VirtualSize":"16.1MB"}
{"Containers":"N/A","CreatedAt":"2023-01-02T00:00:00Z","CreatedSince":"1 day ago","Digest":"sha256:xyz789","ID":"sha256:ghi012","Repository":"ubuntu","SharedSize":"N/A","Size":"72.8MB","Tag":"20.04","UniqueSize":"N/A","VirtualSize":"72.8MB"}"#;

        let images = ImagesCommand::parse_json_output(json_output);

        assert_eq!(images.len(), 2);
        assert_eq!(images[0].repository, "nginx");
        assert_eq!(images[0].tag, "alpine");
        assert_eq!(images[0].image_id, "sha256:def456");
        assert_eq!(images[0].size, "16.1MB");
        assert_eq!(images[0].digest, Some("sha256:abc123".to_string()));

        assert_eq!(images[1].repository, "ubuntu");
        assert_eq!(images[1].tag, "20.04");
    }

    #[test]
    fn test_parse_table_output() {
        let images_cmd = ImagesCommand::new();
        let table_output = r"REPOSITORY          TAG                 IMAGE ID            CREATED             SIZE
nginx               alpine              abc123456789        2 days ago          16.1MB
ubuntu              20.04               def456789012        1 day ago           72.8MB
<none>              <none>              ghi789012345        3 hours ago         5.59MB";

        let images = images_cmd.parse_table_output(table_output);

        assert_eq!(images.len(), 3);
        assert_eq!(images[0].repository, "nginx");
        assert_eq!(images[0].tag, "alpine");
        assert_eq!(images[0].image_id, "abc123456789");
        assert_eq!(images[0].created, "2 days ago");
        assert_eq!(images[0].size, "16.1MB");

        assert_eq!(images[1].repository, "ubuntu");
        assert_eq!(images[1].tag, "20.04");
    }

    #[test]
    fn test_parse_quiet_output() {
        let images_cmd = ImagesCommand::new().quiet();
        let quiet_output = "abc123456789\ndef456789012\nghi789012345";

        let images = images_cmd.parse_output(&CommandOutput {
            stdout: quiet_output.to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        });

        assert_eq!(images.len(), 3);
        assert_eq!(images[0].image_id, "abc123456789");
        assert_eq!(images[0].repository, "<unknown>");
        assert_eq!(images[1].image_id, "def456789012");
        assert_eq!(images[2].image_id, "ghi789012345");
    }

    #[test]
    fn test_images_output_helpers() {
        let output = ImagesOutput {
            output: CommandOutput {
                stdout: "test".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            images: vec![
                ImageInfo {
                    repository: "nginx".to_string(),
                    tag: "alpine".to_string(),
                    image_id: "abc123".to_string(),
                    created: "2 days ago".to_string(),
                    size: "16.1MB".to_string(),
                    digest: None,
                },
                ImageInfo {
                    repository: "nginx".to_string(),
                    tag: "latest".to_string(),
                    image_id: "def456".to_string(),
                    created: "1 day ago".to_string(),
                    size: "133MB".to_string(),
                    digest: None,
                },
                ImageInfo {
                    repository: "ubuntu".to_string(),
                    tag: "20.04".to_string(),
                    image_id: "ghi789".to_string(),
                    created: "3 days ago".to_string(),
                    size: "72.8MB".to_string(),
                    digest: None,
                },
            ],
        };

        assert!(output.success());
        assert_eq!(output.image_count(), 3);
        assert!(!output.is_empty());

        let ids = output.image_ids();
        assert_eq!(ids, vec!["abc123", "def456", "ghi789"]);

        let nginx_images = output.filter_by_repository("nginx");
        assert_eq!(nginx_images.len(), 2);
        assert_eq!(nginx_images[0].tag, "alpine");
        assert_eq!(nginx_images[1].tag, "latest");
    }

    #[test]
    fn test_images_command_extensibility() {
        let mut images_cmd = ImagesCommand::new();
        images_cmd
            .arg("--experimental")
            .args(vec!["--custom", "value"]);

        // Extensibility is handled through the executor's raw_args
        // The actual testing of raw args is done in command.rs tests
        // We can't access private fields, but we know the methods work
        println!("Extensibility methods called successfully");
    }
}
