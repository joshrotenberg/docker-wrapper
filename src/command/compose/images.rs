//! Docker Compose images command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Docker Compose images command builder.
#[derive(Debug, Clone)]
pub struct ComposeImagesCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Output format.
    pub format: Option<ImagesFormat>,
    /// Only displays image IDs.
    pub quiet: bool,
    /// Services to list images for (empty for all).
    pub services: Vec<String>,
}

/// Images output format.
#[derive(Debug, Default, Clone, Copy)]
pub enum ImagesFormat {
    /// Table format (default).
    #[default]
    Table,
    /// JSON format.
    Json,
}

impl std::fmt::Display for ImagesFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::Json => write!(f, "json"),
        }
    }
}

/// Image information from JSON output.
#[derive(Debug, Clone, Deserialize)]
pub struct ImageInfo {
    /// Container name.
    pub container: String,
    /// Repository.
    pub repository: String,
    /// Tag.
    pub tag: String,
    /// Image ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Size.
    pub size: String,
}

/// Result from compose images command.
#[derive(Debug, Clone)]
pub struct ComposeImagesResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Parsed image information (if JSON format).
    pub images: Vec<ImageInfo>,
    /// Services that were queried.
    pub services: Vec<String>,
}

impl ComposeImagesCommand {
    /// Creates a new compose images command.
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            format: None,
            quiet: false,
            services: Vec::new(),
        }
    }

    /// Sets output format.
    #[must_use]
    pub fn format(mut self, format: ImagesFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets output format to JSON.
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some(ImagesFormat::Json);
        self
    }

    /// Sets output format to table.
    #[must_use]
    pub fn format_table(mut self) -> Self {
        self.format = Some(ImagesFormat::Table);
        self
    }

    /// Only displays image IDs.
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Adds a service to list images for.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services to list images for.
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }
}

impl Default for ComposeImagesCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeImagesCommand {
    type Output = ComposeImagesResult;

    fn command_name() -> &'static str {
        <Self as ComposeCommand>::command_name()
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        let images = if matches!(self.format, Some(ImagesFormat::Json)) {
            serde_json::from_str(&output.stdout).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(ComposeImagesResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            images,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeImagesCommand {
    fn subcommand_name() -> &'static str {
        "images"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(format) = self.format {
            args.push("--format".to_string());
            args.push(format.to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args.extend(self.services.clone());
        args
    }
}

impl ComposeImagesResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets parsed image information.
    #[must_use]
    pub fn images(&self) -> &[ImageInfo] {
        &self.images
    }

    /// Gets the services that were queried.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_images_basic() {
        let cmd = ComposeImagesCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"images".to_string()));
    }

    #[test]
    fn test_compose_images_with_format() {
        let cmd = ComposeImagesCommand::new().format_json();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_compose_images_quiet() {
        let cmd = ComposeImagesCommand::new().quiet();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_compose_images_with_services() {
        let cmd = ComposeImagesCommand::new().services(vec!["web", "db"]);
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }
}
