//! Docker Compose attach command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;

/// Docker Compose attach command.
///
/// Attach to a running container's output.
#[derive(Debug, Clone, Default)]
pub struct ComposeAttachCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Service to attach to.
    pub service: String,
    /// Detach keys sequence.
    pub detach_keys: Option<String>,
    /// Container index if service has multiple instances.
    pub index: Option<u32>,
    /// Doesn't stream STDIN.
    pub no_stdin: bool,
    /// Uses a pseudo-TTY.
    pub sig_proxy: bool,
}

/// Result from attach command.
#[derive(Debug, Clone)]
pub struct AttachResult {
    /// Output from the command.
    pub output: String,
    /// Whether the operation succeeded.
    pub success: bool,
}

impl ComposeAttachCommand {
    /// Creates a new attach command.
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            service: service.into(),
            sig_proxy: true, // Default to true
            ..Default::default()
        }
    }

    /// Sets detach keys.
    #[must_use]
    pub fn detach_keys(mut self, keys: impl Into<String>) -> Self {
        self.detach_keys = Some(keys.into());
        self
    }

    /// Sets container index.
    #[must_use]
    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    /// Doesn't attach to STDIN.
    #[must_use]
    pub fn no_stdin(mut self) -> Self {
        self.no_stdin = true;
        self
    }

    /// Disables signal proxy.
    #[must_use]
    pub fn no_sig_proxy(mut self) -> Self {
        self.sig_proxy = false;
        self
    }
}

#[async_trait]
impl DockerCommand for ComposeAttachCommand {
    type Output = AttachResult;

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

        Ok(AttachResult {
            output: output.stdout,
            success: output.success,
        })
    }
}

impl ComposeCommand for ComposeAttachCommand {
    fn subcommand_name() -> &'static str {
        "attach"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // add flags
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_command_basic() {
        let cmd = ComposeAttachCommand::new("web");
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"attach".to_string()));
        assert!(full_args.contains(&"web".to_string()));
    }

    #[test]
    fn test_attach_command_with_detach_keys() {
        let cmd = ComposeAttachCommand::new("web").detach_keys("ctrl-p,ctrl-q");
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--detach-keys".to_string()));
        assert!(args.contains(&"ctrl-p,ctrl-q".to_string()));
    }

    #[test]
    fn test_attach_command_with_index() {
        let cmd = ComposeAttachCommand::new("web").index(2).no_stdin();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"2".to_string()));
        assert!(args.contains(&"--no-stdin".to_string()));
    }

    #[test]
    fn test_attach_command_with_no_sig_proxy() {
        let cmd = ComposeAttachCommand::new("worker").no_sig_proxy();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--sig-proxy=false".to_string()));
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeAttachCommand::new("web")
            .file("docker-compose.yml")
            .project_name("my-project");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
    }
}
