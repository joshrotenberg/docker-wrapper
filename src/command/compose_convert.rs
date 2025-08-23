//! Docker Compose convert command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose convert command builder
#[derive(Debug, Clone)]
pub struct ComposeConvertCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Output format
    pub format: Option<ConvertFormat>,
    /// Output file path
    pub output: Option<String>,
}

/// Convert output format
#[derive(Debug, Clone, Copy)]
pub enum ConvertFormat {
    /// YAML format (default)
    Yaml,
    /// JSON format
    Json,
}

impl std::fmt::Display for ConvertFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yaml => write!(f, "yaml"),
            Self::Json => write!(f, "json"),
        }
    }
}

/// Result from compose convert command
#[derive(Debug, Clone)]
pub struct ComposeConvertResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Converted configuration
    pub converted_config: String,
}

impl ComposeConvertCommand {
    /// Create a new compose convert command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            format: None,
            output: None,
        }
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: ConvertFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set output format to JSON
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some(ConvertFormat::Json);
        self
    }

    /// Set output format to YAML
    #[must_use]
    pub fn format_yaml(mut self) -> Self {
        self.format = Some(ConvertFormat::Yaml);
        self
    }

    /// Set output file path
    #[must_use]
    pub fn output(mut self, output: impl Into<String>) -> Self {
        self.output = Some(output.into());
        self
    }
}

impl Default for ComposeConvertCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeConvertCommand {
    type Output = ComposeConvertResult;

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

        Ok(ComposeConvertResult {
            stdout: output.stdout.clone(),
            stderr: output.stderr,
            success: output.success,
            converted_config: output.stdout,
        })
    }
}

impl ComposeCommand for ComposeConvertCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "convert"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(format) = self.format {
            args.push("--format".to_string());
            args.push(format.to_string());
        }

        if let Some(ref output) = self.output {
            args.push("--output".to_string());
            args.push(output.clone());
        }

        args
    }
}

impl ComposeConvertResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the converted configuration
    #[must_use]
    pub fn converted_config(&self) -> &str {
        &self.converted_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_convert_basic() {
        let cmd = ComposeConvertCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"convert".to_string()));
    }

    #[test]
    fn test_compose_convert_with_format() {
        let cmd = ComposeConvertCommand::new().format_json();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_compose_convert_with_output() {
        let cmd = ComposeConvertCommand::new().output("output.yml");
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--output".to_string()));
        assert!(args.contains(&"output.yml".to_string()));
    }
}
