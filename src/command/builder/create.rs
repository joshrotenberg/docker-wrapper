//! Docker buildx create command implementation.

use crate::command::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Result of buildx create command
#[derive(Debug, Clone)]
pub struct BuildxCreateResult {
    /// The name of the created builder
    pub name: String,
    /// Raw output from the command
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl BuildxCreateResult {
    /// Parse the buildx create output
    fn parse(output: &CommandOutput) -> Self {
        // The output is typically just the builder name
        let name = output.stdout.trim().to_string();
        Self {
            name,
            output: output.stdout.clone(),
            success: output.success,
        }
    }
}

/// Docker buildx create command builder
///
/// Creates a new builder instance for multi-platform builds.
///
/// # Example
///
/// ```rust,no_run
/// use docker_wrapper::{DockerCommand, BuildxCreateCommand};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = BuildxCreateCommand::new()
///     .name("mybuilder")
///     .driver("docker-container")
///     .platform("linux/amd64")
///     .platform("linux/arm64")
///     .use_builder()
///     .bootstrap()
///     .execute()
///     .await?;
///
/// println!("Created builder: {}", result.name);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct BuildxCreateCommand {
    /// Context or endpoint
    context: Option<String>,
    /// Append a node to builder instead of changing it
    append: bool,
    /// Boot builder after creation
    bootstrap: bool,
    /// `BuildKit` daemon config file
    buildkitd_config: Option<String>,
    /// `BuildKit` daemon flags
    buildkitd_flags: Option<String>,
    /// Driver to use
    driver: Option<String>,
    /// Options for the driver
    driver_opts: Vec<String>,
    /// Remove a node from builder instead of changing it
    leave: bool,
    /// Builder instance name
    name: Option<String>,
    /// Create/modify node with given name
    node: Option<String>,
    /// Fixed platforms for current node
    platforms: Vec<String>,
    /// Set the current builder instance
    use_builder: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl BuildxCreateCommand {
    /// Create a new buildx create command
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the context or endpoint
    #[must_use]
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Append a node to builder instead of changing it
    #[must_use]
    pub fn append(mut self) -> Self {
        self.append = true;
        self
    }

    /// Boot builder after creation
    #[must_use]
    pub fn bootstrap(mut self) -> Self {
        self.bootstrap = true;
        self
    }

    /// Set the `BuildKit` daemon config file
    #[must_use]
    pub fn buildkitd_config(mut self, config: impl Into<String>) -> Self {
        self.buildkitd_config = Some(config.into());
        self
    }

    /// Set the `BuildKit` daemon flags
    #[must_use]
    pub fn buildkitd_flags(mut self, flags: impl Into<String>) -> Self {
        self.buildkitd_flags = Some(flags.into());
        self
    }

    /// Set the driver to use (docker-container, kubernetes, remote)
    #[must_use]
    pub fn driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = Some(driver.into());
        self
    }

    /// Add a driver option
    #[must_use]
    pub fn driver_opt(mut self, opt: impl Into<String>) -> Self {
        self.driver_opts.push(opt.into());
        self
    }

    /// Remove a node from builder instead of changing it
    #[must_use]
    pub fn leave(mut self) -> Self {
        self.leave = true;
        self
    }

    /// Set the builder instance name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Create/modify node with given name
    #[must_use]
    pub fn node(mut self, node: impl Into<String>) -> Self {
        self.node = Some(node.into());
        self
    }

    /// Add a fixed platform for current node
    #[must_use]
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platforms.push(platform.into());
        self
    }

    /// Set the current builder instance after creation
    #[must_use]
    pub fn use_builder(mut self) -> Self {
        self.use_builder = true;
        self
    }

    /// Build the command arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = vec!["buildx".to_string(), "create".to_string()];

        if self.append {
            args.push("--append".to_string());
        }

        if self.bootstrap {
            args.push("--bootstrap".to_string());
        }

        if let Some(ref config) = self.buildkitd_config {
            args.push("--buildkitd-config".to_string());
            args.push(config.clone());
        }

        if let Some(ref flags) = self.buildkitd_flags {
            args.push("--buildkitd-flags".to_string());
            args.push(flags.clone());
        }

        if let Some(ref driver) = self.driver {
            args.push("--driver".to_string());
            args.push(driver.clone());
        }

        for opt in &self.driver_opts {
            args.push("--driver-opt".to_string());
            args.push(opt.clone());
        }

        if self.leave {
            args.push("--leave".to_string());
        }

        if let Some(ref name) = self.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        if let Some(ref node) = self.node {
            args.push("--node".to_string());
            args.push(node.clone());
        }

        for platform in &self.platforms {
            args.push("--platform".to_string());
            args.push(platform.clone());
        }

        if self.use_builder {
            args.push("--use".to_string());
        }

        if let Some(ref context) = self.context {
            args.push(context.clone());
        }

        args.extend(self.executor.raw_args.clone());

        args
    }
}

#[async_trait]
impl DockerCommand for BuildxCreateCommand {
    type Output = BuildxCreateResult;

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
        Ok(BuildxCreateResult::parse(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buildx_create_basic() {
        let cmd = BuildxCreateCommand::new();
        let args = cmd.build_args();
        assert_eq!(args, vec!["buildx", "create"]);
    }

    #[test]
    fn test_buildx_create_with_name() {
        let cmd = BuildxCreateCommand::new().name("mybuilder");
        let args = cmd.build_args();
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"mybuilder".to_string()));
    }

    #[test]
    fn test_buildx_create_with_driver() {
        let cmd = BuildxCreateCommand::new().driver("docker-container");
        let args = cmd.build_args();
        assert!(args.contains(&"--driver".to_string()));
        assert!(args.contains(&"docker-container".to_string()));
    }

    #[test]
    fn test_buildx_create_with_platforms() {
        let cmd = BuildxCreateCommand::new()
            .platform("linux/amd64")
            .platform("linux/arm64");
        let args = cmd.build_args();
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/amd64".to_string()));
        assert!(args.contains(&"linux/arm64".to_string()));
    }

    #[test]
    fn test_buildx_create_all_options() {
        let cmd = BuildxCreateCommand::new()
            .name("mybuilder")
            .driver("docker-container")
            .driver_opt("network=host")
            .platform("linux/amd64")
            .bootstrap()
            .use_builder()
            .append();
        let args = cmd.build_args();
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"--driver".to_string()));
        assert!(args.contains(&"--driver-opt".to_string()));
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"--bootstrap".to_string()));
        assert!(args.contains(&"--use".to_string()));
        assert!(args.contains(&"--append".to_string()));
    }
}
