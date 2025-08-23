//! Docker Compose version command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

/// Docker Compose version command
///
/// Show Docker Compose version information.
#[derive(Debug, Clone, Default)]
pub struct ComposeVersionCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Format output (pretty, json)
    pub format: Option<VersionFormat>,
    /// Short output
    pub short: bool,
}

/// Version output format
#[derive(Debug, Clone, Copy)]
pub enum VersionFormat {
    /// Pretty format (default)
    Pretty,
    /// JSON format
    Json,
}

impl VersionFormat {
    /// Convert to command line argument
    #[must_use]
    pub fn as_arg(&self) -> &str {
        match self {
            Self::Pretty => "pretty",
            Self::Json => "json",
        }
    }
}

/// Version information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VersionInfo {
    /// Compose version
    pub version: String,
}

/// Result from version command
#[derive(Debug, Clone)]
pub struct VersionResult {
    /// Version information
    pub info: Option<VersionInfo>,
    /// Raw output
    pub raw_output: String,
}

impl ComposeVersionCommand {
    /// Create a new version command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: VersionFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set output format to JSON
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some(VersionFormat::Json);
        self
    }

    /// Enable short output
    #[must_use]
    pub fn short(mut self) -> Self {
        self.short = true;
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["version".to_string()];

        // Add flags
        if self.short {
            args.push("--short".to_string());
        }

        // Add format
        if let Some(format) = &self.format {
            args.push("--format".to_string());
            args.push(format.as_arg().to_string());
        }

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeVersionCommand {
    type Output = VersionResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        // Parse JSON output if format is JSON
        let info = if matches!(self.format, Some(VersionFormat::Json)) {
            serde_json::from_str(&output.stdout).ok()
        } else {
            None
        };

        Ok(VersionResult {
            info,
            raw_output: output.stdout,
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
    fn test_version_command_basic() {
        let cmd = ComposeVersionCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "version");
    }

    #[test]
    fn test_version_command_with_format() {
        let cmd = ComposeVersionCommand::new().format_json();
        let args = cmd.build_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_version_command_with_short() {
        let cmd = ComposeVersionCommand::new().short();
        let args = cmd.build_args();
        assert!(args.contains(&"--short".to_string()));
    }
}
