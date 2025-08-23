//! Docker Compose config command implementation.

use crate::compose::{ComposeCommandV2 as ComposeCommand, ComposeConfig};
use crate::error::Result;
use async_trait::async_trait;

/// Docker Compose config command
///
/// Validates and displays the Compose configuration.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ComposeConfigCommand {
    /// Base configuration
    pub config: ComposeConfig,
    /// Format output
    pub format: Option<ConfigFormat>,
    /// Resolve image digests
    pub resolve_image_digests: bool,
    /// Don't interpolate environment
    pub no_interpolate: bool,
    /// Don't normalize paths
    pub no_normalize: bool,
    /// Don't check consistency
    pub no_consistency: bool,
    /// Show services
    pub services: bool,
    /// Show volumes
    pub volumes: bool,
    /// Show profiles
    pub profiles: bool,
    /// Show images
    pub images: bool,
    /// Hash of services to include
    pub hash: Option<String>,
    /// Output file
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

impl ConfigFormat {
    /// Convert to command line argument
    #[must_use]
    pub fn as_arg(&self) -> &str {
        match self {
            Self::Yaml => "yaml",
            Self::Json => "json",
        }
    }
}

/// Result from config command
#[derive(Debug, Clone)]
pub struct ConfigResult {
    /// The configuration output (YAML or JSON)
    pub config: String,
    /// Whether the config is valid
    pub is_valid: bool,
}

impl ComposeConfigCommand {
    /// Create a new config command
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

    /// Set output format
    #[must_use]
    pub fn format(mut self, format: ConfigFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Resolve image digests
    #[must_use]
    pub fn resolve_image_digests(mut self) -> Self {
        self.resolve_image_digests = true;
        self
    }

    /// Don't interpolate environment
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

    /// Set services hash
    #[must_use]
    pub fn hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }

    /// Set output file
    #[must_use]
    pub fn output(mut self, path: impl Into<String>) -> Self {
        self.output = Some(path.into());
        self
    }

    /// Enable quiet mode
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["config".to_string()];

        // Add format
        if let Some(format) = &self.format {
            args.push("--format".to_string());
            args.push(format.as_arg().to_string());
        }

        // Add flags
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
        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add hash
        if let Some(hash) = &self.hash {
            args.push("--hash".to_string());
            args.push(hash.clone());
        }

        // Add output
        if let Some(output) = &self.output {
            args.push("--output".to_string());
            args.push(output.clone());
        }

        args
    }
}

#[async_trait]
impl ComposeCommand for ComposeConfigCommand {
    type Output = ConfigResult;

    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    async fn execute_compose(&self, args: Vec<String>) -> Result<Self::Output> {
        let output = self.execute_compose_command(args).await?;

        Ok(ConfigResult {
            config: output.stdout,
            is_valid: output.success,
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
    fn test_config_command_basic() {
        let cmd = ComposeConfigCommand::new();
        let args = cmd.build_args();
        assert_eq!(args[0], "config");
    }

    #[test]
    fn test_config_command_with_format() {
        let cmd = ComposeConfigCommand::new().format(ConfigFormat::Json);
        let args = cmd.build_args();
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
    }

    #[test]
    fn test_config_command_with_flags() {
        let cmd = ComposeConfigCommand::new()
            .resolve_image_digests()
            .no_interpolate()
            .services()
            .quiet();
        let args = cmd.build_args();
        assert!(args.contains(&"--resolve-image-digests".to_string()));
        assert!(args.contains(&"--no-interpolate".to_string()));
        assert!(args.contains(&"--services".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
    }
}
