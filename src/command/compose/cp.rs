//! Docker Compose cp command implementation using unified trait pattern.

use crate::command::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose cp command builder
#[derive(Debug, Clone)]
pub struct ComposeCpCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Source path (can be service:path or local path)
    pub source: String,
    /// Destination path (can be service:path or local path)  
    pub destination: String,
    /// Archive mode (preserve permissions)
    pub archive: bool,
    /// Follow symbolic links
    pub follow_link: bool,
    /// Index of the container (if service has multiple instances)
    pub index: Option<u32>,
}

/// Result from compose cp command
#[derive(Debug, Clone)]
pub struct ComposeCpResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Source path used
    pub source: String,
    /// Destination path used
    pub destination: String,
}

impl ComposeCpCommand {
    /// Create a new compose cp command
    #[must_use]
    pub fn new(source: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            source: source.into(),
            destination: destination.into(),
            archive: false,
            follow_link: false,
            index: None,
        }
    }

    /// Enable archive mode (preserve permissions and ownership)
    #[must_use]
    pub fn archive(mut self) -> Self {
        self.archive = true;
        self
    }

    /// Follow symbolic links in source path
    #[must_use]
    pub fn follow_link(mut self) -> Self {
        self.follow_link = true;
        self
    }

    /// Set container index if service has multiple instances
    #[must_use]
    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }
}

#[async_trait]
impl DockerCommand for ComposeCpCommand {
    type Output = ComposeCpResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposeCpResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            source: self.source.clone(),
            destination: self.destination.clone(),
        })
    }
}

impl ComposeCommand for ComposeCpCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "cp"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.archive {
            args.push("--archive".to_string());
        }

        if self.follow_link {
            args.push("--follow-link".to_string());
        }

        if let Some(index) = self.index {
            args.push("--index".to_string());
            args.push(index.to_string());
        }

        args.push(self.source.clone());
        args.push(self.destination.clone());

        args
    }
}

impl ComposeCpResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the source path used
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the destination path used
    #[must_use]
    pub fn destination(&self) -> &str {
        &self.destination
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_cp_basic() {
        let cmd = ComposeCpCommand::new("web:/app/config.json", "./config.json");
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web:/app/config.json".to_string()));
        assert!(args.contains(&"./config.json".to_string()));

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"cp".to_string()));
    }

    #[test]
    fn test_compose_cp_with_options() {
        let cmd = ComposeCpCommand::new("./data", "db:/var/lib/data")
            .archive()
            .follow_link()
            .index(2);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--archive".to_string()));
        assert!(args.contains(&"--follow-link".to_string()));
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"2".to_string()));
    }
}
