//! Docker tag command implementation.
//!
//! This module provides the `docker tag` command for creating tags for images.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker tag command builder
///
/// Create a tag `TARGET_IMAGE` that refers to `SOURCE_IMAGE`.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::TagCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Tag an image with a new name
/// TagCommand::new("myapp:latest", "myregistry.com/myapp:v1.0.0")
///     .run()
///     .await?;
///
/// // Tag with just a new tag on the same repository
/// TagCommand::new("myapp:latest", "myapp:stable")
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TagCommand {
    /// Source image (name:tag or image ID)
    source_image: String,
    /// Target image (name:tag)
    target_image: String,
    /// Command executor
    executor: CommandExecutor,
}

impl TagCommand {
    /// Create a new tag command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::TagCommand;
    ///
    /// // Tag an image for a registry
    /// let cmd = TagCommand::new("myapp:latest", "docker.io/myuser/myapp:v1.0");
    ///
    /// // Create an alias tag
    /// let cmd = TagCommand::new("nginx:1.21", "nginx:stable");
    /// ```
    #[must_use]
    pub fn new(source_image: impl Into<String>, target_image: impl Into<String>) -> Self {
        Self {
            source_image: source_image.into(),
            target_image: target_image.into(),
            executor: CommandExecutor::new(),
        }
    }

    /// Execute the tag command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The source image doesn't exist
    /// - The target image name is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::TagCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = TagCommand::new("alpine:latest", "my-alpine:latest")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Image tagged successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<TagResult> {
        let output = self.execute().await?;
        Ok(TagResult {
            output,
            source_image: self.source_image.clone(),
            target_image: self.target_image.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for TagCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "tag"
    }

    fn build_args(&self) -> Vec<String> {
        vec![self.source_image.clone(), self.target_image.clone()]
    }

    async fn execute(&self) -> Result<Self::Output> {
        self.executor
            .execute_command(self.command_name(), self.build_args())
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

/// Result from the tag command
#[derive(Debug, Clone)]
pub struct TagResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Source image that was tagged
    pub source_image: String,
    /// Target image name
    pub target_image: String,
}

impl TagResult {
    /// Check if the tag was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the source image
    #[must_use]
    pub fn source_image(&self) -> &str {
        &self.source_image
    }

    /// Get the target image
    #[must_use]
    pub fn target_image(&self) -> &str {
        &self.target_image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_basic() {
        let cmd = TagCommand::new("alpine:latest", "my-alpine:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["alpine:latest", "my-alpine:latest"]);
    }

    #[test]
    fn test_tag_with_registry() {
        let cmd = TagCommand::new("myapp:latest", "docker.io/myuser/myapp:v1.0.0");
        let args = cmd.build_args();
        assert_eq!(args, vec!["myapp:latest", "docker.io/myuser/myapp:v1.0.0"]);
    }

    #[test]
    fn test_tag_with_image_id() {
        let cmd = TagCommand::new("sha256:abc123", "myimage:tagged");
        let args = cmd.build_args();
        assert_eq!(args, vec!["sha256:abc123", "myimage:tagged"]);
    }

    #[test]
    fn test_tag_same_repo_different_tag() {
        let cmd = TagCommand::new("nginx:1.21", "nginx:stable");
        let args = cmd.build_args();
        assert_eq!(args, vec!["nginx:1.21", "nginx:stable"]);
    }
}
