//! Docker Compose scale command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker Compose scale command
///
/// Scale services to specific number of instances.
#[derive(Debug, Clone, Default)]
pub struct ComposeScaleCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Service scale specifications (service=num)
    pub scales: HashMap<String, u32>,
    /// Don't start new containers
    pub no_deps: bool,
}

/// Result from scale command
#[derive(Debug, Clone)]
pub struct ScaleResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeScaleCommand {
    /// Create a new scale command
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

    /// Scale a service to a specific number of instances
    #[must_use]
    pub fn scale(mut self, service: impl Into<String>, instances: u32) -> Self {
        self.scales.insert(service.into(), instances);
        self
    }

    /// Scale multiple services
    #[must_use]
    pub fn scales<I, S>(mut self, scales: I) -> Self
    where
        I: IntoIterator<Item = (S, u32)>,
        S: Into<String>,
    {
        for (service, count) in scales {
            self.scales.insert(service.into(), count);
        }
        self
    }

    /// Don't start dependency services
    #[must_use]
    pub fn no_deps(mut self) -> Self {
        self.no_deps = true;
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["scale".to_string()];

        // Add flags
        if self.no_deps {
            args.push("--no-deps".to_string());
        }

        // Add service scales
        for (service, count) in &self.scales {
            args.push(format!("{service}={count}"));
        }

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeScaleCommand {
    type Output = ScaleResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(ScaleResult {
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
    fn test_scale_command_basic() {
        let cmd = ComposeScaleCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "scale");
    }

    #[test]
    fn test_scale_command_with_service() {
        let cmd = ComposeScaleCommand::new()
            .scale("web", 3)
            .scale("worker", 5);
        let args = cmd.build_args();
        assert!(args.iter().any(|arg| arg == "web=3" || arg == "worker=5"));
    }

    #[test]
    fn test_scale_command_with_no_deps() {
        let cmd = ComposeScaleCommand::new().scale("web", 2).no_deps();
        let args = cmd.build_args();
        assert!(args.contains(&"--no-deps".to_string()));
        assert!(args.iter().any(|arg| arg == "web=2"));
    }

    #[test]
    fn test_scale_command_with_multiple() {
        let scales = vec![("app", 4), ("cache", 2)];
        let cmd = ComposeScaleCommand::new().scales(scales);
        let args = cmd.build_args();
        assert!(args.iter().any(|arg| arg == "app=4" || arg == "cache=2"));
    }
}
