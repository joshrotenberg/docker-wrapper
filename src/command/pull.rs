//! Docker Pull Command Implementation
//!
//! This module provides a comprehensive implementation of the `docker pull` command,
//! supporting all native Docker pull options for downloading images from registries.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use docker_wrapper::PullCommand;
//! use docker_wrapper::DockerCommandV2;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Basic pull of an image
//!     let pull_cmd = PullCommand::new("nginx:latest");
//!     let output = pull_cmd.execute().await?;
//!     println!("Pull completed: {}", output.success);
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ```no_run
//! use docker_wrapper::PullCommand;
//! use docker_wrapper::DockerCommandV2;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Pull all tags for a repository
//!     let pull_cmd = PullCommand::new("alpine")
//!         .all_tags()
//!         .platform("linux/amd64")
//!         .quiet();
//!
//!     let output = pull_cmd.execute().await?;
//!     println!("All tags pulled: {}", output.success);
//!     Ok(())
//! }
//! ```

use super::{CommandExecutor, CommandOutput, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Pull Command Builder
///
/// Implements the `docker pull` command for downloading images from registries.
///
/// # Docker Pull Overview
///
/// The pull command downloads images from Docker registries (like Docker Hub)
/// to the local Docker daemon. It supports:
/// - Single image pull by name and tag
/// - All tags pull for a repository
/// - Multi-platform image selection
/// - Quiet mode for minimal output
/// - Content trust verification control
///
/// # Image Naming
///
/// Images can be specified in several formats:
/// - `image` - Defaults to latest tag
/// - `image:tag` - Specific tag
/// - `image@digest` - Specific digest
/// - `registry/image:tag` - Specific registry
/// - `registry:port/image:tag` - Custom registry port
///
/// # Examples
///
/// ```no_run
/// use docker_wrapper::PullCommand;
/// use docker_wrapper::DockerCommandV2;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Pull latest nginx
///     let output = PullCommand::new("nginx")
///         .execute()
///         .await?;
///
///     println!("Pull success: {}", output.success);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PullCommand {
    /// Image name with optional tag or digest
    image: String,
    /// Download all tagged images in the repository
    all_tags: bool,
    /// Skip image verification (disable content trust)
    disable_content_trust: bool,
    /// Set platform if server is multi-platform capable
    platform: Option<String>,
    /// Suppress verbose output
    quiet: bool,
    /// Command executor for handling raw arguments and execution
    pub executor: CommandExecutor,
}

impl PullCommand {
    /// Create a new `PullCommand` instance
    ///
    /// # Arguments
    ///
    /// * `image` - The image name to pull (e.g., "nginx:latest", "alpine", "redis:7.0")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx:latest");
    /// ```
    #[must_use]
    pub fn new<S: Into<String>>(image: S) -> Self {
        Self {
            image: image.into(),
            all_tags: false,
            disable_content_trust: false,
            platform: None,
            quiet: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Download all tagged images in the repository
    ///
    /// When enabled, pulls all available tags for the specified image repository.
    /// Cannot be used with specific tags or digests.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("alpine")
    ///     .all_tags();
    /// ```
    #[must_use]
    pub fn all_tags(mut self) -> Self {
        self.all_tags = true;
        self
    }

    /// Skip image verification (disable content trust)
    ///
    /// By default, Docker may verify image signatures when content trust is enabled.
    /// This option disables that verification.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx:latest")
    ///     .disable_content_trust();
    /// ```
    #[must_use]
    pub fn disable_content_trust(mut self) -> Self {
        self.disable_content_trust = true;
        self
    }

    /// Set platform if server is multi-platform capable
    ///
    /// Specifies the platform for which to pull the image when the image
    /// supports multiple platforms (architectures).
    ///
    /// Common platform values:
    /// - `linux/amd64` - 64-bit Intel/AMD Linux
    /// - `linux/arm64` - 64-bit ARM Linux
    /// - `linux/arm/v7` - 32-bit ARM Linux
    /// - `windows/amd64` - 64-bit Windows
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx:latest")
    ///     .platform("linux/arm64");
    /// ```
    #[must_use]
    pub fn platform<S: Into<String>>(mut self, platform: S) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Suppress verbose output
    ///
    /// Reduces the amount of output during the pull operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx:latest")
    ///     .quiet();
    /// ```
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Get the image name
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx:latest");
    /// assert_eq!(pull_cmd.get_image(), "nginx:latest");
    /// ```
    #[must_use]
    pub fn get_image(&self) -> &str {
        &self.image
    }

    /// Check if all tags mode is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("alpine").all_tags();
    /// assert!(pull_cmd.is_all_tags());
    /// ```
    #[must_use]
    pub fn is_all_tags(&self) -> bool {
        self.all_tags
    }

    /// Check if quiet mode is enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx").quiet();
    /// assert!(pull_cmd.is_quiet());
    /// ```
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Get the platform if set
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx").platform("linux/arm64");
    /// assert_eq!(pull_cmd.get_platform(), Some("linux/arm64"));
    /// ```
    #[must_use]
    pub fn get_platform(&self) -> Option<&str> {
        self.platform.as_deref()
    }

    /// Check if content trust is disabled
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PullCommand;
    ///
    /// let pull_cmd = PullCommand::new("nginx").disable_content_trust();
    /// assert!(pull_cmd.is_content_trust_disabled());
    /// ```
    #[must_use]
    pub fn is_content_trust_disabled(&self) -> bool {
        self.disable_content_trust
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

impl Default for PullCommand {
    fn default() -> Self {
        Self::new("hello-world")
    }
}

#[async_trait]
impl DockerCommandV2 for PullCommand {
    type Output = CommandOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["pull".to_string()];

        // Add all-tags flag
        if self.all_tags {
            args.push("--all-tags".to_string());
        }

        // Add disable-content-trust flag
        if self.disable_content_trust {
            args.push("--disable-content-trust".to_string());
        }

        // Add platform
        if let Some(ref platform) = self.platform {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }

        // Add quiet flag
        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add image name (must be last)
        args.push(self.image.clone());

        // Add raw args from executor
        args.extend(self.executor.raw_args.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        self.executor.execute_command("docker", args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_command_basic() {
        let pull_cmd = PullCommand::new("nginx:latest");
        let args = pull_cmd.build_command_args();

        assert_eq!(args, vec!["pull", "nginx:latest"]);
        assert_eq!(pull_cmd.get_image(), "nginx:latest");
        assert!(!pull_cmd.is_all_tags());
        assert!(!pull_cmd.is_quiet());
        assert!(!pull_cmd.is_content_trust_disabled());
        assert_eq!(pull_cmd.get_platform(), None);
    }

    #[test]
    fn test_pull_command_with_all_tags() {
        let pull_cmd = PullCommand::new("alpine").all_tags();
        let args = pull_cmd.build_command_args();

        assert!(args.contains(&"--all-tags".to_string()));
        assert!(args.contains(&"alpine".to_string()));
        assert_eq!(args[0], "pull");
        assert!(pull_cmd.is_all_tags());
    }

    #[test]
    fn test_pull_command_with_platform() {
        let pull_cmd = PullCommand::new("nginx:latest").platform("linux/arm64");
        let args = pull_cmd.build_command_args();

        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/arm64".to_string()));
        assert!(args.contains(&"nginx:latest".to_string()));
        assert_eq!(args[0], "pull");
        assert_eq!(pull_cmd.get_platform(), Some("linux/arm64"));
    }

    #[test]
    fn test_pull_command_with_quiet() {
        let pull_cmd = PullCommand::new("redis:7.0").quiet();
        let args = pull_cmd.build_command_args();

        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"redis:7.0".to_string()));
        assert_eq!(args[0], "pull");
        assert!(pull_cmd.is_quiet());
    }

    #[test]
    fn test_pull_command_disable_content_trust() {
        let pull_cmd = PullCommand::new("ubuntu:22.04").disable_content_trust();
        let args = pull_cmd.build_command_args();

        assert!(args.contains(&"--disable-content-trust".to_string()));
        assert!(args.contains(&"ubuntu:22.04".to_string()));
        assert_eq!(args[0], "pull");
        assert!(pull_cmd.is_content_trust_disabled());
    }

    #[test]
    fn test_pull_command_all_options() {
        let pull_cmd = PullCommand::new("postgres")
            .all_tags()
            .platform("linux/amd64")
            .quiet()
            .disable_content_trust();

        let args = pull_cmd.build_command_args();

        assert!(args.contains(&"--all-tags".to_string()));
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/amd64".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--disable-content-trust".to_string()));
        assert!(args.contains(&"postgres".to_string()));
        assert_eq!(args[0], "pull");

        // Verify helper methods
        assert!(pull_cmd.is_all_tags());
        assert!(pull_cmd.is_quiet());
        assert!(pull_cmd.is_content_trust_disabled());
        assert_eq!(pull_cmd.get_platform(), Some("linux/amd64"));
        assert_eq!(pull_cmd.get_image(), "postgres");
    }

    #[test]
    fn test_pull_command_with_registry() {
        let pull_cmd = PullCommand::new("registry.hub.docker.com/library/nginx:alpine");
        let args = pull_cmd.build_command_args();

        assert_eq!(
            args,
            vec!["pull", "registry.hub.docker.com/library/nginx:alpine"]
        );
        assert_eq!(
            pull_cmd.get_image(),
            "registry.hub.docker.com/library/nginx:alpine"
        );
    }

    #[test]
    fn test_pull_command_with_digest() {
        let pull_cmd = PullCommand::new(
            "nginx@sha256:abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
        );
        let args = pull_cmd.build_command_args();

        assert_eq!(
            args,
            vec![
                "pull",
                "nginx@sha256:abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab"
            ]
        );
    }

    #[test]
    fn test_pull_command_order() {
        let pull_cmd = PullCommand::new("alpine:3.18")
            .quiet()
            .platform("linux/arm64")
            .all_tags();

        let args = pull_cmd.build_command_args();

        // Command should be first
        assert_eq!(args[0], "pull");

        // Image should be last
        assert_eq!(args.last(), Some(&"alpine:3.18".to_string()));

        // All options should be present
        assert!(args.contains(&"--all-tags".to_string()));
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/arm64".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_pull_command_default() {
        let pull_cmd = PullCommand::default();
        assert_eq!(pull_cmd.get_image(), "hello-world");
    }
}
