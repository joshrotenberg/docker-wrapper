//! Docker load command implementation.
//!
//! This module provides the `docker load` command for loading Docker images from tar archives.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

/// Docker load command builder
///
/// Load an image from a tar archive or STDIN.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::LoadCommand;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Load image from file
/// let result = LoadCommand::new()
///     .input(Path::new("alpine.tar"))
///     .run()
///     .await?;
///
/// println!("Loaded images: {:?}", result.loaded_images());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct LoadCommand {
    /// Input file path
    input: Option<String>,
    /// Suppress progress output during load
    quiet: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl LoadCommand {
    /// Create a new load command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::LoadCommand;
    ///
    /// let cmd = LoadCommand::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            input: None,
            quiet: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Set input file path
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::LoadCommand;
    /// use std::path::Path;
    ///
    /// let cmd = LoadCommand::new()
    ///     .input(Path::new("images.tar"));
    /// ```
    #[must_use]
    pub fn input(mut self, path: &Path) -> Self {
        self.input = Some(path.to_string_lossy().into_owned());
        self
    }

    /// Suppress progress output during load
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::LoadCommand;
    ///
    /// let cmd = LoadCommand::new().quiet();
    /// ```
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Execute the load command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The input file doesn't exist or is not readable
    /// - The tar archive is corrupted or invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::LoadCommand;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = LoadCommand::new()
    ///     .input(Path::new("alpine.tar"))
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Images loaded successfully");
    ///     for image in result.loaded_images() {
    ///         println!("  - {}", image);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<LoadResult> {
        let output = self.execute().await?;

        // Parse loaded images from output
        let loaded_images = Self::parse_loaded_images(&output.stdout);

        Ok(LoadResult {
            output,
            loaded_images,
        })
    }

    /// Parse loaded image names from the command output
    fn parse_loaded_images(stdout: &str) -> Vec<String> {
        let mut images = Vec::new();
        for line in stdout.lines() {
            if line.starts_with("Loaded image:") {
                if let Some(image) = line.strip_prefix("Loaded image:") {
                    images.push(image.trim().to_string());
                }
            } else if line.starts_with("Loaded image ID:") {
                if let Some(id) = line.strip_prefix("Loaded image ID:") {
                    images.push(id.trim().to_string());
                }
            }
        }
        images
    }
}

impl Default for LoadCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for LoadCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["load".to_string()];

        if let Some(ref input_file) = self.input {
            args.push("--input".to_string());
            args.push(input_file.clone());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

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

/// Result from the load command
#[derive(Debug, Clone)]
pub struct LoadResult {
    /// Raw command output
    pub output: CommandOutput,
    /// List of loaded image names/IDs
    pub loaded_images: Vec<String>,
}

impl LoadResult {
    /// Check if the load was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the list of loaded images
    #[must_use]
    pub fn loaded_images(&self) -> &[String] {
        &self.loaded_images
    }

    /// Get the count of loaded images
    #[must_use]
    pub fn image_count(&self) -> usize {
        self.loaded_images.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_basic() {
        let cmd = LoadCommand::new();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["load"]);
    }

    #[test]
    fn test_load_with_input() {
        let cmd = LoadCommand::new().input(Path::new("images.tar"));
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["load", "--input", "images.tar"]);
    }

    #[test]
    fn test_load_with_quiet() {
        let cmd = LoadCommand::new().quiet();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["load", "--quiet"]);
    }

    #[test]
    fn test_load_with_all_options() {
        let cmd = LoadCommand::new()
            .input(Path::new("/tmp/alpine.tar"))
            .quiet();
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["load", "--input", "/tmp/alpine.tar", "--quiet"]);
    }

    #[test]
    fn test_parse_loaded_images() {
        let output =
            "Loaded image: alpine:latest\nLoaded image: nginx:1.21\nLoaded image ID: sha256:abc123";
        let images = LoadCommand::parse_loaded_images(output);
        assert_eq!(images, vec!["alpine:latest", "nginx:1.21", "sha256:abc123"]);
    }

    #[test]
    fn test_parse_loaded_images_empty() {
        let output = "";
        let images = LoadCommand::parse_loaded_images(output);
        assert!(images.is_empty());
    }
}
