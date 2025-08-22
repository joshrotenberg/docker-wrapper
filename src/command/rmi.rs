//! Docker rmi command implementation.
//!
//! This module provides the `docker rmi` command for removing Docker images.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker rmi command builder
///
/// Remove one or more images.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::RmiCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Remove a single image
/// RmiCommand::new("old-image:v1.0")
///     .run()
///     .await?;
///
/// // Force remove multiple images
/// RmiCommand::new_multiple(vec!["image1", "image2", "image3"])
///     .force()
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RmiCommand {
    /// Image names or IDs to remove
    images: Vec<String>,
    /// Force removal of images
    force: bool,
    /// Do not delete untagged parents
    no_prune: bool,
    /// Command executor
    executor: CommandExecutor,
}

impl RmiCommand {
    /// Create a new rmi command for a single image
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::RmiCommand;
    ///
    /// let cmd = RmiCommand::new("old-image:latest");
    /// ```
    #[must_use]
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            images: vec![image.into()],
            force: false,
            no_prune: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new rmi command for multiple images
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::RmiCommand;
    ///
    /// let cmd = RmiCommand::new_multiple(vec!["image1:latest", "image2:v1.0"]);
    /// ```
    #[must_use]
    pub fn new_multiple(images: Vec<impl Into<String>>) -> Self {
        Self {
            images: images.into_iter().map(Into::into).collect(),
            force: false,
            no_prune: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another image to remove
    #[must_use]
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.images.push(image.into());
        self
    }

    /// Force removal of the images
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::RmiCommand;
    ///
    /// let cmd = RmiCommand::new("stubborn-image:latest")
    ///     .force();
    /// ```
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Do not delete untagged parents
    #[must_use]
    pub fn no_prune(mut self) -> Self {
        self.no_prune = true;
        self
    }

    /// Execute the rmi command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Any of the specified images don't exist
    /// - Images are in use by containers (unless force is used)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::RmiCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = RmiCommand::new("unused-image:latest")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Removed {} images", result.removed_images().len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<RmiResult> {
        let output = self.execute().await?;

        // Parse removed images from output
        let removed_images = Self::parse_removed_images(&output.stdout);

        Ok(RmiResult {
            output,
            removed_images,
        })
    }

    /// Parse removed image IDs from the command output
    fn parse_removed_images(stdout: &str) -> Vec<String> {
        let mut removed = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("Deleted:") {
                if let Some(id) = line.strip_prefix("Deleted:") {
                    removed.push(id.trim().to_string());
                }
            } else if line.starts_with("Untagged:") {
                if let Some(tag) = line.strip_prefix("Untagged:") {
                    removed.push(tag.trim().to_string());
                }
            }
        }

        removed
    }
}

#[async_trait]
impl DockerCommand for RmiCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "rmi"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.force {
            args.push("--force".to_string());
        }

        if self.no_prune {
            args.push("--no-prune".to_string());
        }

        // Add image names/IDs
        args.extend(self.images.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.images.is_empty() {
            return Err(crate::error::Error::invalid_config(
                "No images specified for removal",
            ));
        }

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

/// Result from the rmi command
#[derive(Debug, Clone)]
pub struct RmiResult {
    /// Raw command output
    pub output: CommandOutput,
    /// List of removed image IDs/tags
    pub removed_images: Vec<String>,
}

impl RmiResult {
    /// Check if the removal was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the list of removed images
    #[must_use]
    pub fn removed_images(&self) -> &[String] {
        &self.removed_images
    }

    /// Get the count of removed images
    #[must_use]
    pub fn removed_count(&self) -> usize {
        self.removed_images.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rmi_single_image() {
        let cmd = RmiCommand::new("test-image:latest");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-image:latest"]);
    }

    #[test]
    fn test_rmi_multiple_images() {
        let cmd = RmiCommand::new_multiple(vec!["image1:latest", "image2:v1.0", "image3"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["image1:latest", "image2:v1.0", "image3"]);
    }

    #[test]
    fn test_rmi_with_force() {
        let cmd = RmiCommand::new("stubborn-image:latest").force();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--force", "stubborn-image:latest"]);
    }

    #[test]
    fn test_rmi_with_no_prune() {
        let cmd = RmiCommand::new("test-image:latest").no_prune();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--no-prune", "test-image:latest"]);
    }

    #[test]
    fn test_rmi_all_options() {
        let cmd = RmiCommand::new("test-image:latest")
            .image("another-image:v1.0")
            .force()
            .no_prune();
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "--force",
                "--no-prune",
                "test-image:latest",
                "another-image:v1.0"
            ]
        );
    }

    #[test]
    fn test_parse_removed_images() {
        let output =
            "Untagged: test-image:latest\nDeleted: sha256:abc123def456\nDeleted: sha256:789xyz123";
        let removed = RmiCommand::parse_removed_images(output);
        assert_eq!(
            removed,
            vec![
                "test-image:latest",
                "sha256:abc123def456",
                "sha256:789xyz123"
            ]
        );
    }

    #[test]
    fn test_parse_removed_images_empty() {
        let removed = RmiCommand::parse_removed_images("");
        assert!(removed.is_empty());
    }
}
