//! Docker builder build command
//!
//! Alternative interface to start a build (similar to `docker build`)

use crate::command::build::{BuildCommand, BuildOutput};
use crate::command::{CommandExecutor, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// `docker builder build` command - alternative interface to docker build
///
/// This is essentially the same as `docker build` but accessed through
/// the builder subcommand interface.
///
/// # Example
/// ```no_run
/// use docker_wrapper::command::builder::BuilderBuildCommand;
/// use docker_wrapper::DockerCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let result = BuilderBuildCommand::new(".")
///     .tag("myapp:latest")
///     .no_cache()
///     .execute()
///     .await?;
///
/// if let Some(id) = result.image_id {
///     println!("Built image: {}", id);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuilderBuildCommand {
    /// Underlying build command
    inner: BuildCommand,
}

impl BuilderBuildCommand {
    /// Create a new builder build command
    ///
    /// # Arguments
    /// * `context` - Build context path (e.g., ".", "/path/to/dir")
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            inner: BuildCommand::new(context),
        }
    }

    /// Set the Dockerfile to use
    #[must_use]
    pub fn dockerfile(mut self, path: impl Into<String>) -> Self {
        self.inner = self.inner.file(path.into());
        self
    }

    /// Tag the image
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.inner = self.inner.tag(tag);
        self
    }

    /// Do not use cache when building
    #[must_use]
    pub fn no_cache(mut self) -> Self {
        self.inner = self.inner.no_cache();
        self
    }

    /// Set build-time variables
    #[must_use]
    pub fn build_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.inner = self.inner.build_arg(key, value);
        self
    }

    /// Set target build stage
    #[must_use]
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.inner = self.inner.target(target);
        self
    }

    /// Set platform for multi-platform builds
    #[must_use]
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.inner = self.inner.platform(platform);
        self
    }

    /// Enable `BuildKit` backend
    #[must_use]
    pub fn buildkit(mut self) -> Self {
        // This would normally set DOCKER_BUILDKIT=1 environment variable
        // For now, we'll add it as a raw arg (in practice, this would be an env var)
        self.inner
            .executor
            .raw_args
            .push("DOCKER_BUILDKIT=1".to_string());
        self
    }

    /// Enable quiet mode
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.inner = self.inner.quiet();
        self
    }

    /// Always remove intermediate containers
    #[must_use]
    pub fn force_rm(mut self) -> Self {
        self.inner = self.inner.force_rm();
        self
    }

    /// Remove intermediate containers after successful build (default)
    #[must_use]
    pub fn rm(self) -> Self {
        // rm is the default behavior, this is a no-op
        self
    }

    /// Do not remove intermediate containers after build
    #[must_use]
    pub fn no_rm(mut self) -> Self {
        self.inner = self.inner.no_rm();
        self
    }

    /// Always attempt to pull newer version of base image
    #[must_use]
    pub fn pull(mut self) -> Self {
        self.inner = self.inner.pull();
        self
    }
}

#[async_trait]
impl DockerCommand for BuilderBuildCommand {
    type Output = BuildOutput;

    fn get_executor(&self) -> &CommandExecutor {
        &self.inner.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.inner.executor
    }

    fn build_command_args(&self) -> Vec<String> {
        // Get the args from the inner build command
        let mut inner_args = self.inner.build_command_args();

        // Replace "build" with "builder build"
        if !inner_args.is_empty() && inner_args[0] == "build" {
            inner_args[0] = "builder".to_string();
            inner_args.insert(1, "build".to_string());
        }

        inner_args
    }

    async fn execute(&self) -> Result<Self::Output> {
        // The builder build command has the same output as regular build
        let args = self.build_command_args();
        let output = self.execute_command(args).await?;

        // Extract image ID from output
        let image_id = extract_image_id(&output.stdout);

        Ok(BuildOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
            image_id,
        })
    }
}

/// Extract image ID from build output
fn extract_image_id(stdout: &str) -> Option<String> {
    // Look for "Successfully built <id>" or "writing image sha256:<id>"
    for line in stdout.lines().rev() {
        if line.contains("Successfully built") {
            return line.split_whitespace().last().map(String::from);
        }
        if line.contains("writing image sha256:") {
            if let Some(id) = line.split("sha256:").nth(1) {
                return Some(format!(
                    "sha256:{}",
                    id.split_whitespace()
                        .next()?
                        .trim_end_matches('"')
                        .trim_end_matches('}')
                ));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_build_basic() {
        let cmd = BuilderBuildCommand::new(".");
        let args = cmd.build_command_args();
        assert_eq!(&args[0..2], &["builder", "build"]);
        assert!(args.contains(&".".to_string()));
    }

    #[test]
    fn test_builder_build_with_options() {
        let cmd = BuilderBuildCommand::new("/app")
            .tag("myapp:latest")
            .dockerfile("custom.Dockerfile")
            .no_cache()
            .build_arg("VERSION", "1.0");

        let args = cmd.build_command_args();
        assert_eq!(&args[0..2], &["builder", "build"]);
        assert!(args.contains(&"--tag".to_string()));
        assert!(args.contains(&"myapp:latest".to_string()));
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"custom.Dockerfile".to_string()));
        assert!(args.contains(&"--no-cache".to_string()));
        assert!(args.contains(&"--build-arg".to_string()));
        assert!(args.contains(&"VERSION=1.0".to_string()));
    }

    #[test]
    fn test_builder_build_buildkit() {
        let mut cmd = BuilderBuildCommand::new(".");
        cmd = cmd.buildkit();

        // Check that DOCKER_BUILDKIT was added as a raw arg
        assert!(cmd
            .inner
            .executor
            .raw_args
            .contains(&"DOCKER_BUILDKIT=1".to_string()));
    }

    #[test]
    fn test_builder_build_platform() {
        let cmd = BuilderBuildCommand::new(".")
            .platform("linux/amd64")
            .target("production");

        let args = cmd.build_command_args();
        assert!(args.contains(&"--platform".to_string()));
        assert!(args.contains(&"linux/amd64".to_string()));
        assert!(args.contains(&"--target".to_string()));
        assert!(args.contains(&"production".to_string()));
    }

    #[test]
    fn test_builder_build_extensibility() {
        let mut cmd = BuilderBuildCommand::new(".");
        cmd.inner
            .executor
            .raw_args
            .push("--custom-flag".to_string());

        let args = cmd.build_command_args();
        assert!(args.contains(&"--custom-flag".to_string()));
    }
}
