//! Docker buildx inspect command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of buildx inspect command
#[derive(Debug, Clone)]
pub struct BuildxInspectResult {
    /// The name of the builder
    pub name: Option<String>,
    /// The driver used by the builder
    pub driver: Option<String>,
    /// The status of the builder
    pub status: Option<String>,
    /// Platforms supported by the builder
    pub platforms: Vec<String>,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl BuildxInspectResult {
    /// Parse the buildx inspect output
    fn parse(output: &CommandOutput) -> Self {
        let stdout = &output.stdout;
        let mut name = None;
        let mut driver = None;
        let mut status = None;
        let mut platforms = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("Name:") {
                name = Some(line.trim_start_matches("Name:").trim().to_string());
            } else if line.starts_with("Driver:") {
                driver = Some(line.trim_start_matches("Driver:").trim().to_string());
            } else if line.starts_with("Status:") {
                status = Some(line.trim_start_matches("Status:").trim().to_string());
            } else if line.starts_with("Platforms:") {
                let platform_str = line.trim_start_matches("Platforms:").trim();
                platforms = platform_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }

        Self {
            name,
            driver,
            status,
            platforms,
            output: stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker buildx inspect command builder
///
/// Inspects a builder instance.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, BuildxInspectCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = BuildxInspectCommand::new()
///     .bootstrap()
///     .execute()
///     .await?;
///
/// if let Some(name) = &result.name {
///     println!("Builder: {}", name);
/// }
/// println!("Platforms: {:?}", result.platforms);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct BuildxInspectCommand {
    /// The builder name to inspect (optional, defaults to current)
    name: Option<String>,
    /// Ensure builder has booted before inspecting
    bootstrap: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl BuildxInspectCommand {
    /// Create a new buildx inspect command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the builder name to inspect
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Ensure builder has booted before inspecting
    #[must_use]
    pub fn bootstrap(mut self) -> Self {
        self.bootstrap = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["buildx".to_string(), "inspect".to_string()];

        if self.bootstrap {
            args.push("--bootstrap".to_string());
        }

        if let Some(ref name) = self.name {
            args.push(name.clone());
        }

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for BuildxInspectCommand {
    type Output = BuildxInspectResult;

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        self.build_args()
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_args();
        let output = self.execute_command(args).await?;
        Ok(BuildxInspectResult::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buildx_inspect_basic() {
        let cmd = BuildxInspectCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["buildx", "inspect"]);
    }

    #[test]
    fn test_buildx_inspect_with_name() {
        let cmd = BuildxInspectCommand::new().name("mybuilder");
        let args = cmd.build_args();
        assert!(args.contains(&"mybuilder".to_string()));
    }

    #[test]
    fn test_buildx_inspect_with_bootstrap() {
        let cmd = BuildxInspectCommand::new().bootstrap();
        let args = cmd.build_args();
        assert!(args.contains(&"--bootstrap".to_string()));
    }

    #[test]
    fn test_buildx_inspect_all_options() {
        let cmd = BuildxInspectCommand::new().name("mybuilder").bootstrap();
        let args = cmd.build_args();
        assert!(args.contains(&"--bootstrap".to_string()));
        assert!(args.contains(&"mybuilder".to_string()));
    }

    #[test]
    fn test_buildx_inspect_result_parse() {
        let output = CommandOutput {
            stdout: "Name:   mybuilder\nDriver: docker-container\nStatus: running\nPlatforms: linux/amd64, linux/arm64".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        let result = BuildxInspectResult::parse(&output);
        assert_eq!(result.name, Some("mybuilder".to_string()));
        assert_eq!(result.driver, Some("docker-container".to_string()));
        assert_eq!(result.status, Some("running".to_string()));
        assert_eq!(result.platforms, vec!["linux/amd64", "linux/arm64"]);
    }
}
