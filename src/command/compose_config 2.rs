//! Docker Compose config command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose config command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are appropriate for config command
pub struct ComposeConfigCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Output format
    pub format: Option<ConfigFormat>,
    /// Resolve image digests
    pub resolve_image_digests: bool,
    /// Don't interpolate environment variables
    pub no_interpolate: bool,
    /// Don't normalize paths
    pub no_normalize: bool,
    /// Don't check consistency
    pub no_consistency: bool,
    /// Show services only
    pub services: bool,
    /// Show volumes only
    pub volumes: bool,
    /// Show profiles only
    pub profiles: bool,
    /// Show images only
    pub images: bool,
    /// Hash of services to include
    pub hash: Option<String>,
    /// Output file path
    pub output: Option<String>,
    /// Quiet mode
    pub quiet: bool,
}

/// Config output format
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    /// YAML format (default)
    Yaml,
    /// JSON format
    Json,
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yaml => write!(f, "yaml"),
            Self::Json => write!(f, "json"),
        }
    }
}

/// Result from compose config command
#[derive(Debug, Clone)]
pub struct ComposeConfigResult {
    /// Raw stdout output (configuration YAML/JSON)
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Whether configuration is valid
    pub is_valid: bool,
}

impl ComposeConfigCommand {
    /// Create a new compose config command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            format: None,
            resolve_image_digests: false,
            no_interpolate: false,
            no_normalize: false,
            no_consistency: false,
            services: false,
            volumes: false,
            profiles: false,
            images: false,
            hash: None,
            output: None,
            quiet: false,
        }
    }

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: ConfigFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set output format to JSON
    #[must_use]
    pub fn format_json(mut self) -> Self {
        self.format = Some(ConfigFormat::Json);
        self
    }

    /// Set output format to YAML
    #[must_use]
    pub fn format_yaml(mut self) -> Self {
        self.format = Some(ConfigFormat::Yaml);
        self
    }

    /// Resolve image digests
    #[must_use]
    pub fn resolve_image_digests(mut self) -> Self {
        self.resolve_image_digests = true;
        self
    }

    /// Don't interpolate environment variables
    #[must_use]
    pub fn no_interpolate(mut self) -> Self {
        self.no_interpolate = true;
        self
    }

    /// Don't normalize paths
    #[must_use]
    pub fn no_normalize(mut self) -> Self {
        self.no_normalize = true;
        self
    }

    /// Don't check consistency
    #[must_use]
    pub fn no_consistency(mut self) -> Self {
        self.no_consistency = true;
        self
    }

    /// Show services only
    #[must_use]
    pub fn services(mut self) -> Self {
        self.services = true;
        self
    }

    /// Show volumes only
    #[must_use]
    pub fn volumes(mut self) -> Self {
        self.volumes = true;
        self
    }

    /// Show profiles only
    #[must_use]
    pub fn profiles(mut self) -> Self {
        self.profiles = true;
        self
    }

    /// Show images only
    #[must_use]
    pub fn images(mut self) -> Self {
        self.images = true;
        self
    }

    /// Set hash of services to include
    #[must_use]
    pub fn hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }

    /// Set output file path
    #[must_use]
    pub fn output(mut self, output: impl Into<String>) -> Self {
        self.output = Some(output.into());
        self
    }

    /// Enable quiet mode
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }
}

impl Default for ComposeConfigCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommandV2 for ComposeConfigCommand {
    type Output = ComposeConfigResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        // Use the ComposeCommand implementation explicitly
        <Self as ComposeCommand>::build_command_args(self)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = <Self as ComposeCommand>::build_command_args(self);
        let output = self.execute_command(args).await?;

        Ok(ComposeConfigResult {
            stdout: output.stdout.clone(),
            stderr: output.stderr,
            success: output.success,
            is_valid: output.success && !output.stdout.is_empty(),
        })
    }
}

impl ComposeCommand for ComposeConfigCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "config"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(format) = self.format {
            args.push("--format".to_string());
            args.push(format.to_string());
        }

        if self.resolve_image_digests {
            args.push("--resolve-image-digests".to_string());
        }

        if self.no_interpolate {
            args.push("--no-interpolate".to_string());
        }

        if self.no_normalize {
            args.push("--no-normalize".to_string());
        }

        if self.no_consistency {
            args.push("--no-consistency".to_string());
        }

        if self.services {
            args.push("--services".to_string());
        }

        if self.volumes {
            args.push("--volumes".to_string());
        }

        if self.profiles {
            args.push("--profiles".to_string());
        }

        if self.images {
            args.push("--images".to_string());
        }

        if let Some(ref hash) = self.hash {
            args.push("--hash".to_string());
            args.push(hash.clone());
        }

        if let Some(ref output) = self.output {
            args.push("--output".to_string());
            args.push(output.clone());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args
    }
}

impl ComposeConfigResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Check if the configuration is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Get the configuration output
    #[must_use]
    pub fn config_output(&self) -> &str {
        &self.stdout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_config_basic() {
        let cmd = ComposeConfigCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"config".to_string()));
    }

    #[test]
    fn test_compose_config_with_format() {
        let cmd = ComposeConfigCommand::new().format_json();
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_compose_config_with_flags() {
        let cmd = ComposeConfigCommand::new()
            .resolve_image_digests()
            .no_interpolate()
            .services()
            .quiet();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--resolve-image-digests".to_string()));
        assert!(args.contains(&"--no-interpolate".to_string()));
        assert!(args.contains(&"--services".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
    }

    #[test]
    fn test_compose_config_show_options() {
        let cmd = ComposeConfigCommand::new().volumes().profiles().images();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--volumes".to_string()));
        assert!(args.contains(&"--profiles".to_string()));
        assert!(args.contains(&"--images".to_string()));
    }

    #[test]
    fn test_compose_config_with_hash_and_output() {
        let cmd = ComposeConfigCommand::new()
            .hash("web=sha256:123")
            .output("output.yml");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--hash".to_string()));
        assert!(args.contains(&"web=sha256:123".to_string()));
        assert!(args.contains(&"--output".to_string()));
        assert!(args.contains(&"output.yml".to_string()));
    }

    #[test]
    fn test_config_format_display() {
        assert_eq!(ConfigFormat::Yaml.to_string(), "yaml");
        assert_eq!(ConfigFormat::Json.to_string(), "json");
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeConfigCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .format_json()
            .services();

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
        assert!(args.contains(&"--services".to_string()));
    }
}
