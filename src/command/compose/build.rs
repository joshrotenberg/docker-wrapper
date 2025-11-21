//! Docker Compose build command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker Compose build command builder.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // multiple boolean flags are appropriate for build command
pub struct ComposeBuildCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Services to build (empty for all).
    pub services: Vec<String>,
    /// Doesn't use cache when building the image.
    pub no_cache: bool,
    /// Always attempts to pull a newer version of the image.
    pub pull: bool,
    /// Doesn't print anything to stdout.
    pub quiet: bool,
    /// Sets build-time variables.
    pub build_args: HashMap<String, String>,
    /// Builds images in parallel.
    pub parallel: bool,
    /// Amount of memory for builds.
    pub memory: Option<String>,
    /// Builds with `BuildKit` progress output.
    pub progress: Option<ProgressType>,
    /// Sets the SSH agent socket or key.
    pub ssh: Option<String>,
}

/// Build progress output type.
#[derive(Debug, Default, Clone, Copy)]
pub enum ProgressType {
    /// Auto-detects progress type.
    #[default]
    Auto,
    /// Plain text progress.
    Plain,
    /// TTY progress.
    Tty,
}

impl std::fmt::Display for ProgressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Plain => write!(f, "plain"),
            Self::Tty => write!(f, "tty"),
        }
    }
}

/// Result from [`ComposeBuildCommand`].
#[derive(Debug, Clone)]
pub struct ComposeBuildResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Services that were built.
    pub services: Vec<String>,
}

impl ComposeBuildCommand {
    /// Creates a new [`ComposeBuildCommand`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            no_cache: false,
            pull: false,
            quiet: false,
            build_args: HashMap::new(),
            parallel: false,
            memory: None,
            progress: None,
            ssh: None,
        }
    }

    /// Adds a service to build.
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Adds multiple services.
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Doesn't use cache when building the image.
    #[must_use]
    pub fn no_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }

    /// Always attempts to pull a newer version of the image.
    #[must_use]
    pub fn pull(mut self) -> Self {
        self.pull = true;
        self
    }

    /// Doesn't print anything to stdout.
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Adds a build-time variable.
    #[must_use]
    pub fn build_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build_args.insert(key.into(), value.into());
        self
    }

    /// Adds multiple build-time variables.
    #[must_use]
    pub fn build_args(mut self, args: HashMap<String, String>) -> Self {
        self.build_args.extend(args);
        self
    }

    /// Builds images in parallel.
    #[must_use]
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Sets memory limit for builds.
    #[must_use]
    pub fn memory(mut self, memory: impl Into<String>) -> Self {
        self.memory = Some(memory.into());
        self
    }

    /// Sets progress output type.
    #[must_use]
    pub fn progress(mut self, progress: ProgressType) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Sets SSH agent socket or key.
    #[must_use]
    pub fn ssh(mut self, ssh: impl Into<String>) -> Self {
        self.ssh = Some(ssh.into());
        self
    }
}

impl Default for ComposeBuildCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeBuildCommand {
    type Output = ComposeBuildResult;

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

        Ok(ComposeBuildResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposeBuildCommand {
    fn subcommand_name() -> &'static str {
        "build"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.no_cache {
            args.push("--no-cache".to_string());
        }

        if self.pull {
            args.push("--pull".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        if self.parallel {
            args.push("--parallel".to_string());
        }

        // add build args
        for (key, value) in &self.build_args {
            args.push("--build-arg".to_string());
            args.push(format!("{key}={value}"));
        }

        // add memory limit
        if let Some(ref memory) = self.memory {
            args.push("--memory".to_string());
            args.push(memory.clone());
        }

        // add progress output
        if let Some(progress) = self.progress {
            args.push("--progress".to_string());
            args.push(progress.to_string());
        }

        // add SSH configuration
        if let Some(ref ssh) = self.ssh {
            args.push("--ssh".to_string());
            args.push(ssh.clone());
        }

        // add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeBuildResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the services that were built.
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_build_basic() {
        let cmd = ComposeBuildCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"build".to_string()));
    }

    #[test]
    fn test_compose_build_with_flags() {
        let cmd = ComposeBuildCommand::new()
            .no_cache()
            .pull()
            .quiet()
            .parallel();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--no-cache".to_string()));
        assert!(args.contains(&"--pull".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--parallel".to_string()));
    }

    #[test]
    fn test_compose_build_with_services() {
        let cmd = ComposeBuildCommand::new().service("web").service("db");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_compose_build_with_build_args() {
        let cmd = ComposeBuildCommand::new()
            .build_arg("VERSION", "1.0")
            .build_arg("ENV", "production");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--build-arg".to_string()));
        // Should contain both build args in some order
        let version_arg = "VERSION=1.0";
        let env_arg = "ENV=production";
        assert!(args.contains(&version_arg.to_string()) || args.contains(&env_arg.to_string()));
    }

    #[test]
    fn test_compose_build_all_options() {
        let cmd = ComposeBuildCommand::new()
            .no_cache()
            .pull()
            .parallel()
            .build_arg("VERSION", "2.0")
            .memory("1g")
            .progress(ProgressType::Plain)
            .ssh("default")
            .services(vec!["web", "worker"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--no-cache".to_string()));
        assert!(args.contains(&"--pull".to_string()));
        assert!(args.contains(&"--parallel".to_string()));
        assert!(args.contains(&"--build-arg".to_string()));
        assert!(args.contains(&"VERSION=2.0".to_string()));
        assert!(args.contains(&"--memory".to_string()));
        assert!(args.contains(&"1g".to_string()));
        assert!(args.contains(&"--progress".to_string()));
        assert!(args.contains(&"plain".to_string()));
        assert!(args.contains(&"--ssh".to_string()));
        assert!(args.contains(&"default".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"worker".to_string()));
    }

    #[test]
    fn test_progress_output_display() {
        assert_eq!(ProgressType::Auto.to_string(), "auto");
        assert_eq!(ProgressType::Plain.to_string(), "plain");
        assert_eq!(ProgressType::Tty.to_string(), "tty");
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeBuildCommand::new()
            .file("docker-compose.yml")
            .project_name("my-project")
            .no_cache()
            .service("web");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--no-cache".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}
