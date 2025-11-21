//! Docker save command implementation.
//!
//! This module provides the `docker save` command for saving Docker images to tar archives.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

/// Docker save command builder
///
/// Save one or more images to a tar archive (streamed to STDOUT by default).
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::SaveCommand;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Save image to file
/// SaveCommand::new("alpine:latest")
///     .output(Path::new("alpine.tar"))
///     .run()
///     .await?;
///
/// // Save multiple images
/// SaveCommand::new_multiple(vec!["alpine:latest", "nginx:latest"])
///     .output(Path::new("images.tar"))
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SaveCommand {
    /// Images to save
    images: Vec<String>,
    /// Output file path
    output: Option<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl SaveCommand {
    /// Create a new save command for a single image
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::SaveCommand;
    ///
    /// let cmd = SaveCommand::new("alpine:latest");
    /// ```
    #[must_use]
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            images: vec![image.into()],
            output: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new save command for multiple images
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::SaveCommand;
    ///
    /// let cmd = SaveCommand::new_multiple(vec!["alpine:latest", "nginx:latest"]);
    /// ```
    #[must_use]
    pub fn new_multiple(images: Vec<impl Into<String>>) -> Self {
        Self {
            images: images.into_iter().map(Into::into).collect(),
            output: None,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another image to save
    #[must_use]
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.images.push(image.into());
        self
    }

    /// Set output file path
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::SaveCommand;
    /// use std::path::Path;
    ///
    /// let cmd = SaveCommand::new("alpine:latest")
    ///     .output(Path::new("alpine.tar"));
    /// ```
    #[must_use]
    pub fn output(mut self, path: &Path) -> Self {
        self.output = Some(path.to_string_lossy().into_owned());
        self
    }

    /// Execute the save command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - Any of the specified images don't exist
    /// - Cannot write to the output file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::SaveCommand;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = SaveCommand::new("alpine:latest")
    ///     .output(Path::new("alpine.tar"))
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Images saved successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<SaveResult> {
        let output = self.execute().await?;
        Ok(SaveResult {
            output,
            images: self.images.clone(),
            output_file: self.output.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for SaveCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["save".to_string()];

        if let Some(ref output_file) = self.output {
            args.push("--output".to_string());
            args.push(output_file.clone());
        }

        // Add image names
        args.extend(self.images.clone());

        args.extend(self.executor.raw_args.clone());
        args
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }
}

/// Result from the save command
#[derive(Debug, Clone)]
pub struct SaveResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Images that were saved
    pub images: Vec<String>,
    /// Output file path if specified
    pub output_file: Option<String>,
}

impl SaveResult {
    /// Check if the save was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the saved images
    #[must_use]
    pub fn images(&self) -> &[String] {
        &self.images
    }

    /// Get the output file path
    #[must_use]
    pub fn output_file(&self) -> Option<&str> {
        self.output_file.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_single_image() {
        let cmd = SaveCommand::new("alpine:latest");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["save", "alpine:latest"]);
    }

    #[test]
    fn test_save_multiple_images() {
        let cmd = SaveCommand::new_multiple(vec!["alpine:latest", "nginx:latest", "redis:latest"]);
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["save", "alpine:latest", "nginx:latest", "redis:latest"]
        );
    }

    #[test]
    fn test_save_with_output() {
        let cmd = SaveCommand::new("alpine:latest").output(Path::new("alpine.tar"));
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["save", "--output", "alpine.tar", "alpine:latest"]
        );
    }

    #[test]
    fn test_save_multiple_with_output() {
        let cmd = SaveCommand::new_multiple(vec!["alpine", "nginx"])
            .image("redis")
            .output(Path::new("/tmp/images.tar"));
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "save",
                "--output",
                "/tmp/images.tar",
                "alpine",
                "nginx",
                "redis"
            ]
        );
    }
}
