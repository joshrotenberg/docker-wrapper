//! Docker Compose cp command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// Docker Compose cp command
///
/// Copy files/folders between a service container and the local filesystem.
#[derive(Debug, Clone)]
pub struct ComposeCpCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Source path (can be container:path or local path)
    pub source: String,
    /// Destination path (can be container:path or local path)
    pub destination: String,
    /// Archive mode (preserve permissions)
    pub archive: bool,
    /// Follow symbolic links
    pub follow_link: bool,
    /// Index of the container (if service has multiple instances)
    pub index: Option<u32>,
}

/// Result from cp command
#[derive(Debug, Clone)]
pub struct CpResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeCpCommand {
    /// Create a new cp command
    #[must_use]
    pub fn new(source: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            config: ComposeConfig::default(),
            source: source.into(),
            destination: destination.into(),
            archive: false,
            follow_link: false,
            index: None,
        }
    }

    /// Copy from container to local
    #[must_use]
    pub fn from_container(
        service: impl Into<String>,
        container_path: impl Into<String>,
        local_path: impl Into<PathBuf>,
    ) -> Self {
        let source = format!("{}:{}", service.into(), container_path.into());
        let destination = local_path.into().display().to_string();
        Self::new(source, destination)
    }

    /// Copy from local to container
    #[must_use]
    pub fn to_container(
        local_path: impl Into<PathBuf>,
        service: impl Into<String>,
        container_path: impl Into<String>,
    ) -> Self {
        let source = local_path.into().display().to_string();
        let destination = format!("{}:{}", service.into(), container_path.into());
        Self::new(source, destination)
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

    /// Enable archive mode
    #[must_use]
    pub fn archive(mut self) -> Self {
        self.archive = true;
        self
    }

    /// Follow symbolic links
    #[must_use]
    pub fn follow_link(mut self) -> Self {
        self.follow_link = true;
        self
    }

    /// Set container index
    #[must_use]
    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["cp".to_string()];

        // Add flags
        if self.archive {
            args.push("--archive".to_string());
        }
        if self.follow_link {
            args.push("--follow-link".to_string());
        }

        // Add index if specified
        if let Some(index) = self.index {
            args.push("--index".to_string());
            args.push(index.to_string());
        }

        // Add source and destination
        args.push(self.source.clone());
        args.push(self.destination.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeCpCommand {
    type Output = CpResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(CpResult {
            output: output.stdout,
            success: output.success,
        })
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        self.execute_compose(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_command_basic() {
        let cmd = ComposeCpCommand::new("web:/app/config.yml", "./config.yml");
        let args = cmd.build_args();
        assert_eq!(args[0], "cp");
        assert!(args.contains(&"web:/app/config.yml".to_string()));
        assert!(args.contains(&"./config.yml".to_string()));
    }

    #[test]
    fn test_cp_from_container() {
        let cmd = ComposeCpCommand::from_container("web", "/app/logs", "./logs");
        let args = cmd.build_args();
        assert!(args.contains(&"web:/app/logs".to_string()));
        assert!(args.contains(&"./logs".to_string()));
    }

    #[test]
    fn test_cp_to_container() {
        let cmd = ComposeCpCommand::to_container("./config.yml", "web", "/app/config.yml");
        let args = cmd.build_args();
        assert!(args.contains(&"./config.yml".to_string()));
        assert!(args.contains(&"web:/app/config.yml".to_string()));
    }

    #[test]
    fn test_cp_command_with_flags() {
        let cmd = ComposeCpCommand::new("web:/data", "./data")
            .archive()
            .follow_link()
            .index(1);
        let args = cmd.build_args();
        assert!(args.contains(&"--archive".to_string()));
        assert!(args.contains(&"--follow-link".to_string()));
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"1".to_string()));
    }
}
