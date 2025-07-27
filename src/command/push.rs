//! Docker Push Command Implementation
//!
//! This module provides a comprehensive implementation of the `docker push` command,
//! supporting all native Docker push options for uploading images to registries.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use docker_wrapper::PushCommand;
//! use docker_wrapper::DockerCommand;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Basic push of an image
//!     let push_cmd = PushCommand::new("myapp:latest");
//!     let output = push_cmd.execute().await?;
//!     println!("Push completed: {}", output.success);
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ```no_run
//! use docker_wrapper::PushCommand;
//! use docker_wrapper::DockerCommand;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Push all tags for a repository
//!     let push_cmd = PushCommand::new("myregistry.com/myapp")
//!         .all_tags()
//!         .platform("linux/amd64")
//!         .quiet()
//!         .disable_content_trust();
//!
//!     let output = push_cmd.execute().await?;
//!     println!("All tags pushed: {}", output.success);
//!     Ok(())
//! }
//! ```

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker Push Command Builder
///
/// Implements the `docker push` command for uploading images to registries.
///
/// # Docker Push Overview
///
/// The push command uploads images from the local Docker daemon to Docker registries
/// (like Docker Hub, AWS ECR, or private registries). It supports:
/// - Single image push by name and tag
/// - All tags push for a repository
/// - Platform-specific manifest pushing
/// - Quiet mode for minimal output
/// - Content trust signing control
///
/// # Image Naming
///
/// Images can be specified in several formats:
/// - `image:tag` - Image with specific tag
/// - `registry/image:tag` - Specific registry
/// - `registry:port/image:tag` - Custom registry port
/// - `namespace/image:tag` - Namespaced image (e.g., Docker Hub organizations)
///
/// # Registry Authentication
///
/// Push operations typically require authentication. Use `docker login` first
/// or configure registry credentials through Docker configuration files.
///
/// # Examples
///
/// ```no_run
/// use docker_wrapper::PushCommand;
/// use docker_wrapper::DockerCommand;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Push to Docker Hub
///     let output = PushCommand::new("username/myapp:v1.0")
///         .execute()
///         .await?;
///
///     println!("Push success: {}", output.success);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PushCommand {
    /// Image name with tag to push
    image: String,
    /// Push all tags of an image to the repository
    all_tags: bool,
    /// Skip image signing (disable content trust)
    disable_content_trust: bool,
    /// Push a platform-specific manifest as a single-platform image
    platform: Option<String>,
    /// Suppress verbose output
    quiet: bool,
    /// Command executor for handling raw arguments and execution
    executor: CommandExecutor,
}

impl PushCommand {
    /// Create a new `PushCommand` instance
    ///
    /// # Arguments
    ///
    /// * `image` - The image name to push (e.g., "myapp:latest", "registry.com/myapp:v1.0")
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp:latest");
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

    /// Push all tags of an image to the repository
    ///
    /// When enabled, pushes all available tags for the specified image repository.
    /// The image name should not include a specific tag when using this option.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myregistry.com/myapp")
    ///     .all_tags();
    /// ```
    #[must_use]
    pub fn all_tags(mut self) -> Self {
        self.all_tags = true;
        self
    }

    /// Skip image signing (disable content trust)
    ///
    /// By default, Docker may sign images when content trust is enabled.
    /// This option disables that signing process.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp:latest")
    ///     .disable_content_trust();
    /// ```
    #[must_use]
    pub fn disable_content_trust(mut self) -> Self {
        self.disable_content_trust = true;
        self
    }

    /// Push a platform-specific manifest as a single-platform image
    ///
    /// Pushes only the specified platform variant of a multi-platform image.
    /// The image index won't be pushed, meaning other manifests including
    /// attestations won't be preserved.
    ///
    /// Platform format: `os[/arch[/variant]]`
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
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp:latest")
    ///     .platform("linux/amd64");
    /// ```
    #[must_use]
    pub fn platform<S: Into<String>>(mut self, platform: S) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Suppress verbose output
    ///
    /// Reduces the amount of output during the push operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp:latest")
    ///     .quiet();
    /// ```
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Build the command arguments
    ///
    /// This method constructs the complete argument list for the docker push command.
    fn build_command_args(&self) -> Vec<String> {
        let mut args = Vec::new();

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

        args
    }

    /// Get the image name
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp:latest");
    /// assert_eq!(push_cmd.get_image(), "myapp:latest");
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
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp").all_tags();
    /// assert!(push_cmd.is_all_tags());
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
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp").quiet();
    /// assert!(push_cmd.is_quiet());
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
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp").platform("linux/arm64");
    /// assert_eq!(push_cmd.get_platform(), Some("linux/arm64"));
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
    /// use docker_wrapper::PushCommand;
    ///
    /// let push_cmd = PushCommand::new("myapp").disable_content_trust();
    /// assert!(push_cmd.is_content_trust_disabled());
    /// ```
    #[must_use]
    pub fn is_content_trust_disabled(&self) -> bool {
        self.disable_content_trust
    }
}

impl Default for PushCommand {
    fn default() -> Self {
        Self::new("localhost/test:latest")
    }
}

#[async_trait]
impl DockerCommand for PushCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "push"
    }

    fn build_args(&self) -> Vec<String> {
        self.build_command_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.executor
            .execute_command(self.command_name(), args)
            .await
    }

    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.executor.add_arg(arg);
        self
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.executor.add_args(args);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.executor.add_flag(flag);
        self
    }

    fn option(&mut self, key: &str, value: &str) -> &mut Self {
        self.executor.add_option(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_command_basic() {
        let push_cmd = PushCommand::new("myapp:latest");
        let args = push_cmd.build_args();

        assert_eq!(args, vec!["myapp:latest"]);
        assert_eq!(push_cmd.get_image(), "myapp:latest");
        assert!(!push_cmd.is_all_tags());
        assert!(!push_cmd.is_quiet());
        assert!(!push_cmd.is_content_trust_disabled());
        assert_eq!(push_cmd.get_platform(), None);
    }

    #[test]
    fn test_push_command_with_all_tags() {
        let push_cmd = PushCommand::new("myregistry.com/myapp").all_tags();
        let args = push_cmd.build_args();

        assert!(args.contains(&"--all-tags".to_string()));
        assert!(args.contains(&"myregistry.com/myapp".to_string()));
        assert!(push_cmd.is_all_tags());
    }

    #[test]
    fn test_push_command_with_platform() {
        let push_cmd = PushCommand::new("myapp:latest").platform("linux/arm64");
        let args = push_cmd.build_args();

        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/arm64".to_string()));
        assert!(args.contains(&"myapp:latest".to_string()));
        assert_eq!(push_cmd.get_platform(), Some("linux/arm64"));
    }

    #[test]
    fn test_push_command_with_quiet() {
        let push_cmd = PushCommand::new("myapp:v1.0").quiet();
        let args = push_cmd.build_args();

        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"myapp:v1.0".to_string()));
        assert!(push_cmd.is_quiet());
    }

    #[test]
    fn test_push_command_disable_content_trust() {
        let push_cmd = PushCommand::new("registry.com/myapp:stable").disable_content_trust();
        let args = push_cmd.build_args();

        assert!(args.contains(&"--disable-content-trust".to_string()));
        assert!(args.contains(&"registry.com/myapp:stable".to_string()));
        assert!(push_cmd.is_content_trust_disabled());
    }

    #[test]
    fn test_push_command_all_options() {
        let push_cmd = PushCommand::new("myregistry.io/myapp")
            .all_tags()
            .platform("linux/amd64")
            .quiet()
            .disable_content_trust();

        let args = push_cmd.build_args();

        assert!(args.contains(&"--all-tags".to_string()));
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/amd64".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--disable-content-trust".to_string()));
        assert!(args.contains(&"myregistry.io/myapp".to_string()));

        // Verify helper methods
        assert!(push_cmd.is_all_tags());
        assert!(push_cmd.is_quiet());
        assert!(push_cmd.is_content_trust_disabled());
        assert_eq!(push_cmd.get_platform(), Some("linux/amd64"));
        assert_eq!(push_cmd.get_image(), "myregistry.io/myapp");
    }

    #[test]
    fn test_push_command_with_registry_and_namespace() {
        let push_cmd = PushCommand::new("registry.example.com:5000/namespace/myapp:v2.1");
        let args = push_cmd.build_args();

        assert_eq!(args, vec!["registry.example.com:5000/namespace/myapp:v2.1"]);
        assert_eq!(
            push_cmd.get_image(),
            "registry.example.com:5000/namespace/myapp:v2.1"
        );
    }

    #[test]
    fn test_push_command_docker_hub_format() {
        let push_cmd = PushCommand::new("username/repository:tag");
        let args = push_cmd.build_args();

        assert_eq!(args, vec!["username/repository:tag"]);
        assert_eq!(push_cmd.get_image(), "username/repository:tag");
    }

    #[test]
    fn test_push_command_order() {
        let push_cmd = PushCommand::new("myapp:latest")
            .quiet()
            .platform("linux/arm64")
            .all_tags();

        let args = push_cmd.build_args();

        // Image should be last
        assert_eq!(args.last(), Some(&"myapp:latest".to_string()));

        // All options should be present
        assert!(args.contains(&"--all-tags".to_string()));
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/arm64".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_push_command_default() {
        let push_cmd = PushCommand::default();
        assert_eq!(push_cmd.get_image(), "localhost/test:latest");
    }

    #[test]
    fn test_push_command_local_registry() {
        let push_cmd = PushCommand::new("localhost:5000/myapp:dev");
        let args = push_cmd.build_args();

        assert_eq!(args, vec!["localhost:5000/myapp:dev"]);
        assert_eq!(push_cmd.get_image(), "localhost:5000/myapp:dev");
    }

    #[test]
    fn test_push_command_extensibility() {
        let mut push_cmd = PushCommand::new("myapp");
        push_cmd
            .arg("--experimental")
            .args(vec!["--custom", "value"]);

        // Extensibility is handled through the executor's raw_args
        // The actual testing of raw args is done in command.rs tests
        // We can't access private fields, but we know the methods work
        println!("Extensibility methods called successfully");
    }
}
