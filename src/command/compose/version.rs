//! Docker Compose version command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Docker Compose version command builder.
#[derive(Debug, Clone)]
pub struct ComposeVersionCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Format output (pretty, json).
    pub format: Option<VersionFormat>,
    /// Short output.
    pub short: bool,
}

/// Version output format.
#[derive(Debug, Default, Clone, Copy)]
pub enum VersionFormat {
    /// Pretty format (default).
    #[default]
    Pretty,
    /// JSON format.
    Json,
}

impl std::fmt::Display for VersionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pretty => write!(f, "pretty"),
            Self::Json => write!(f, "json"),
        }
    }
}

/// Version information from JSON output.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VersionInfo {
    /// Compose version.
    pub version: String,
}

/// Result from compose version command.
#[derive(Debug, Clone)]
pub struct ComposeVersionResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Parsed version information (if JSON format).
    pub version_info: Option<VersionInfo>,
}

impl ComposeVersionCommand {
    /// Creates a new compose version command.
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            format: None,
            short: false,
        }
    }

    /// Sets output format.
    #[must_use]
    pub fn format(mut self, format: VersionFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets output format to JSON.
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some(VersionFormat::Json);
        self
    }

    /// Sets output format to pretty.
    #[must_use]
    pub fn format_pretty(mut self) -> Self {
        self.format = Some(VersionFormat::Pretty);
        self
    }

    /// Enables short output.
    #[must_use]
    pub fn short(mut self) -> Self {
        self.short = true;
        self
    }
}

impl Default for ComposeVersionCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeVersionCommand {
    fn command_name() -> &'static str {
        <Self as ComposeCommand>::command_name()
    }
    type Output = ComposeVersionResult;

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

        // parses JSON output if format is JSON
        let version_info = if matches!(self.format, Some(VersionFormat::Json)) {
            serde_json::from_str(&output.stdout).ok()
        } else {
            None
        };

        Ok(ComposeVersionResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            version_info,
        })
    }
}

impl ComposeCommand for ComposeVersionCommand {
    fn subcommand_name() -> &'static str {
        "version"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.short {
            args.push("--short".to_string());
        }

        if let Some(format) = self.format {
            args.push("--format".to_string());
            args.push(format.to_string());
        }

        args
    }
}

impl ComposeVersionResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets parsed version information (if JSON format was used).
    #[must_use]
    pub fn version_info(&self) -> Option<&VersionInfo> {
        self.version_info.as_ref()
    }

    /// Gets the version string (from parsed info or raw output).
    #[must_use]
    pub fn version_string(&self) -> Option<String> {
        if let Some(info) = &self.version_info {
            Some(info.version.clone())
        } else {
            // tries to extract version from raw output
            self.stdout
                .lines()
                .find(|line| line.contains("version"))
                .map(|line| line.trim().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_version_basic() {
        let cmd = ComposeVersionCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"version".to_string()));
    }

    #[test]
    fn test_compose_version_with_format() {
        let cmd = ComposeVersionCommand::new().format_json();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_compose_version_with_short() {
        let cmd = ComposeVersionCommand::new().short();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--short".to_string()));
    }

    #[test]
    fn test_compose_version_pretty_format() {
        let cmd = ComposeVersionCommand::new().format_pretty();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"pretty".to_string()));
    }

    #[test]
    fn test_compose_version_all_options() {
        let cmd = ComposeVersionCommand::new().format_json().short();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--short".to_string()));
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_version_format_display() {
        assert_eq!(VersionFormat::Pretty.to_string(), "pretty");
        assert_eq!(VersionFormat::Json.to_string(), "json");
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeVersionCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .format_json();

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }
}
