//! Docker Compose down command implementation.

use super::{execute_compose_command, ComposeCommand, ComposeConfig, ComposeOutput};
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose down command builder
#[derive(Debug, Clone)]
pub struct ComposeDownCommand {
    /// Base compose configuration
    config: ComposeConfig,
    /// Remove images
    remove_images: Option<RemoveImages>,
    /// Remove named volumes
    volumes: bool,
    /// Remove orphan containers
    remove_orphans: bool,
    /// Timeout for container shutdown
    timeout: Option<Duration>,
    /// Services to stop (empty for all)
    services: Vec<String>,
}

/// Image removal options for compose down
#[derive(Debug, Clone, Copy)]
pub enum RemoveImages {
    /// Remove all images used by services
    All,
    /// Remove only images that don't have a custom tag
    Local,
}

impl std::fmt::Display for RemoveImages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Local => write!(f, "local"),
        }
    }
}

impl ComposeDownCommand {
    /// Create a new compose down command
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ComposeConfig::new(),
            remove_images: None,
            volumes: false,
            remove_orphans: false,
            timeout: None,
            services: Vec::new(),
        }
    }

    /// Create with a specific compose configuration
    #[must_use]
    pub fn with_config(config: ComposeConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Remove images (all or local)
    #[must_use]
    pub fn remove_images(mut self, policy: RemoveImages) -> Self {
        self.remove_images = Some(policy);
        self
    }

    /// Remove named volumes declared in the volumes section
    #[must_use]
    pub fn volumes(mut self) -> Self {
        self.volumes = true;
        self
    }

    /// Remove containers for services not defined in the compose file
    #[must_use]
    pub fn remove_orphans(mut self) -> Self {
        self.remove_orphans = true;
        self
    }

    /// Set timeout for container shutdown
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add a service to stop
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

    /// Set compose file
    #[must_use]
    pub fn file(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config = self.config.file(path);
        self
    }

    /// Set project name
    #[must_use]
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config = self.config.project_name(name);
        self
    }

    /// Execute the compose down command
    ///
    /// # Errors
    /// Returns an error if:
    /// - Docker Compose is not installed
    /// - Compose file is not found
    /// - Container stop/removal fails
    pub async fn run(&self) -> Result<ComposeDownResult> {
        let output = self.execute().await?;

        Ok(ComposeDownResult {
            output,
            removed_volumes: self.volumes,
            removed_images: self.remove_images.is_some(),
        })
    }
}

impl Default for ComposeDownCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ComposeCommand for ComposeDownCommand {
    type Output = ComposeOutput;

    fn subcommand(&self) -> &'static str {
        "down"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref remove) = self.remove_images {
            args.push("--rmi".to_string());
            args.push(remove.to_string());
        }

        if self.volumes {
            args.push("--volumes".to_string());
        }

        if self.remove_orphans {
            args.push("--remove-orphans".to_string());
        }

        if let Some(timeout) = self.timeout {
            args.push("--timeout".to_string());
            args.push(timeout.as_secs().to_string());
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        execute_compose_command(&self.config, self.subcommand(), self.build_args()).await
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }
}

/// Result from compose down command
#[derive(Debug, Clone)]
pub struct ComposeDownResult {
    /// Raw command output
    pub output: ComposeOutput,
    /// Whether volumes were removed
    pub removed_volumes: bool,
    /// Whether images were removed
    pub removed_images: bool,
}

impl ComposeDownResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Check if volumes were removed
    #[must_use]
    pub fn volumes_removed(&self) -> bool {
        self.removed_volumes
    }

    /// Check if images were removed
    #[must_use]
    pub fn images_removed(&self) -> bool {
        self.removed_images
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_down_basic() {
        let cmd = ComposeDownCommand::new();
        let args = cmd.build_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_compose_down_with_volumes() {
        let cmd = ComposeDownCommand::new().volumes();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--volumes"]);
    }

    #[test]
    fn test_compose_down_remove_images() {
        let cmd = ComposeDownCommand::new().remove_images(RemoveImages::All);
        let args = cmd.build_args();
        assert_eq!(args, vec!["--rmi", "all"]);
    }

    #[test]
    fn test_compose_down_all_options() {
        let cmd = ComposeDownCommand::new()
            .remove_images(RemoveImages::Local)
            .volumes()
            .remove_orphans()
            .timeout(Duration::from_secs(30))
            .service("web")
            .service("db");

        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "--rmi",
                "local",
                "--volumes",
                "--remove-orphans",
                "--timeout",
                "30",
                "web",
                "db"
            ]
        );
    }

    #[test]
    fn test_remove_images_display() {
        assert_eq!(RemoveImages::All.to_string(), "all");
        assert_eq!(RemoveImages::Local.to_string(), "local");
    }
}
