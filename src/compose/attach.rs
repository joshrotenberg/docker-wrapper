//! Docker Compose attach command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose attach command
///
/// Attach to a running container's output.
#[derive(Debug, Clone, Default)]
pub struct ComposeAttachCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Service to attach to
    pub service: String,
    /// Detach keys sequence
    pub detach_keys: Option<String>,
    /// Container index if service has multiple instances
    pub index: Option<u32>,
    /// Don't stream STDIN
    pub no_stdin: bool,
    /// Use a pseudo-TTY
    pub sig_proxy: bool,
}

/// Result from attach command
#[derive(Debug, Clone)]
pub struct AttachResult {
    /// Output from the command
    pub output: String,
    /// Whether the operation succeeded
    pub success: bool,
}

impl ComposeAttachCommand {
    /// Create a new attach command
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            sig_proxy: true, // Default to true
            ..Default::default()
        }
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

    /// Set detach keys
    #[must_use]
    pub fn detach_keys(mut self, keys: impl Into<String>) -> Self {
        self.detach_keys = Some(keys.into());
        self
    }

    /// Set container index
    #[must_use]
    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    /// Don't attach to STDIN
    #[must_use]
    pub fn no_stdin(mut self) -> Self {
        self.no_stdin = true;
        self
    }

    /// Disable signal proxy
    #[must_use]
    pub fn no_sig_proxy(mut self) -> Self {
        self.sig_proxy = false;
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["attach".to_string()];

        // Add flags
        if let Some(ref keys) = self.detach_keys {
            args.push("--detach-keys".to_string());
            args.push(keys.clone());
        }

        if let Some(index) = self.index {
            args.push("--index".to_string());
            args.push(index.to_string());
        }

        if self.no_stdin {
            args.push("--no-stdin".to_string());
        }

        if !self.sig_proxy {
            args.push("--sig-proxy=false".to_string());
        }

        // Add service
        args.push(self.service.clone());

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeAttachCommand {
    type Output = AttachResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(AttachResult {
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
    fn test_attach_command_basic() {
        let cmd = ComposeAttachCommand::new("web");
        let args = cmd.build_args();
        assert_eq!(args[0], "attach");
        assert!(args.contains(&"web".to_string()));
    }

    #[test]
    fn test_attach_command_with_detach_keys() {
        let cmd = ComposeAttachCommand::new("web").detach_keys("ctrl-p,ctrl-q");
        let args = cmd.build_args();
        assert!(args.contains(&"--detach-keys".to_string()));
        assert!(args.contains(&"ctrl-p,ctrl-q".to_string()));
    }

    #[test]
    fn test_attach_command_with_index() {
        let cmd = ComposeAttachCommand::new("web").index(2).no_stdin();
        let args = cmd.build_args();
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"2".to_string()));
        assert!(args.contains(&"--no-stdin".to_string()));
    }

    #[test]
    fn test_attach_command_with_no_sig_proxy() {
        let cmd = ComposeAttachCommand::new("worker").no_sig_proxy();
        let args = cmd.build_args();
        assert!(args.contains(&"--sig-proxy=false".to_string()));
    }
}
