//! Docker Compose images command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

/// Docker Compose images command
///
/// List images used by services.
#[derive(Debug, Clone, Default)]
pub struct ComposeImagesCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Format output (table, json)
    pub format: Option<ImagesFormat>,
    /// Only display image IDs
    pub quiet: bool,
    /// Services to list images for
    pub services: Vec<String>,
}

/// Images output format
#[derive(Debug, Clone, Copy)]
pub enum ImagesFormat {
    /// Table format (default)
    Table,
    /// JSON format
    Json,
}

impl ImagesFormat {
    /// Convert to command line argument
    #[must_use]
    pub fn as_arg(&self) -> &str {
        match self {
            Self::Table => "table",
            Self::Json => "json",
        }
    }
}

/// Image information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageInfo {
    /// Container name
    pub container: String,
    /// Repository
    pub repository: String,
    /// Tag
    pub tag: String,
    /// Image ID
    pub image_id: String,
    /// Size
    pub size: String,
}

/// Result from images command
#[derive(Debug, Clone)]
pub struct ImagesResult {
    /// List of images
    pub images: Vec<ImageInfo>,
    /// Raw output (for non-JSON formats)
    pub raw_output: String,
}

impl ComposeImagesCommand {
    /// Create a new images command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a compose file
    #[must_use]
    pub fn file<P: Into<std::path::PathBuf>>(mut self, file: P) -> Self {
        self.config.files.push(file.into());
        self
    }

    /// Set project name
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config.project_name = Some(name.into());
        self
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: ImagesFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set output format to JSON
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some(ImagesFormat::Json);
        self
    }

    /// Only display image IDs
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Add a service to list images for
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["images".to_string()];

        // Add flags
        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add format
        if let Some(format) = &self.format {
            args.push("--format".to_string());
            args.push(format.as_arg().to_string());
        }

        // Add services
        args.extend(self.services.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeImagesCommand {
    type Output = ImagesResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        // Parse JSON output if format is JSON
        let images = if matches!(self.format, Some(ImagesFormat::Json)) {
            serde_json::from_str(&output.stdout).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(ImagesResult {
            images,
            raw_output: output.stdout,
        })
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.execute_compose(args).await
    }
}

impl ImagesResult {
    /// Get unique images
    #[must_use]
    pub fn unique_images(&self) -> Vec<String> {
        let mut images: Vec<_> = self
            .images
            .iter()
            .map(|img| format!("{}:{}", img.repository, img.tag))
            .collect();
        images.sort();
        images.dedup();
        images
    }

    /// Get total size
    #[must_use]
    pub fn total_size(&self) -> String {
        // This would need proper size parsing
        // For now just return placeholder
        "N/A".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_images_command_basic() {
        let cmd = ComposeImagesCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "images");
    }

    #[test]
    fn test_images_command_with_format() {
        let cmd = ComposeImagesCommand::new().format_json();
        let args = cmd.build_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_images_command_with_quiet() {
        let cmd = ComposeImagesCommand::new().quiet();
        let args = cmd.build_args();
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_images_command_with_services() {
        let cmd = ComposeImagesCommand::new().service("web").service("db");
        let args = cmd.build_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }
}
