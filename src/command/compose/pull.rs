//! Docker Compose pull command implementation using unified trait pattern.

use crate::command::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Pull policy for compose pull command
#[derive(Debug, Clone, Copy)]
pub enum ComposePullPolicy {
    /// Always pull images
    Always,
    /// Pull missing images only
    Missing,
}

impl std::fmt::Display for ComposePullPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Missing => write!(f, "missing"),
        }
    }
}

/// Docker Compose pull command builder
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are appropriate for pull command
#[derive(Debug, Clone)]
pub struct ComposePullCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to pull images for (empty for all)
    pub services: Vec<String>,
    /// Ignore images that can be built
    pub ignore_buildable: bool,
    /// Pull what it can and ignore images with pull failures
    pub ignore_pull_failures: bool,
    /// Also pull services declared as dependencies
    pub include_deps: bool,
    /// Pull policy
    pub policy: Option<ComposePullPolicy>,
    /// Pull without printing progress information
    pub quiet: bool,
}

/// Result from compose pull command
#[derive(Debug, Clone)]
pub struct ComposePullResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were pulled
    pub services: Vec<String>,
}

impl ComposePullCommand {
    /// Create a new compose pull command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            ignore_buildable: false,
            ignore_pull_failures: false,
            include_deps: false,
            policy: None,
            quiet: false,
        }
    }

    /// Add a service to pull
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services to pull
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Ignore images that can be built
    #[must_use]
    pub fn ignore_buildable(mut self) -> Self {
        self.ignore_buildable = true;
        self
    }

    /// Pull what it can and ignore images with pull failures
    #[must_use]
    pub fn ignore_pull_failures(mut self) -> Self {
        self.ignore_pull_failures = true;
        self
    }

    /// Also pull services declared as dependencies
    #[must_use]
    pub fn include_deps(mut self) -> Self {
        self.include_deps = true;
        self
    }

    /// Set pull policy
    #[must_use]
    pub fn policy(mut self, policy: ComposePullPolicy) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Pull without printing progress information
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }
}

impl Default for ComposePullCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposePullCommand {
    type Output = ComposePullResult;

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

        Ok(ComposePullResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
        })
    }
}

impl ComposeCommand for ComposePullCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "pull"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.ignore_buildable {
            args.push("--ignore-buildable".to_string());
        }

        if self.ignore_pull_failures {
            args.push("--ignore-pull-failures".to_string());
        }

        if self.include_deps {
            args.push("--include-deps".to_string());
        }

        if let Some(ref policy) = self.policy {
            args.push("--policy".to_string());
            args.push(policy.to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        args.extend(self.services.clone());
        args
    }
}

impl ComposePullResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were pulled
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_pull_basic() {
        let cmd = ComposePullCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"pull".to_string()));
    }

    #[test]
    fn test_compose_pull_with_options() {
        let cmd = ComposePullCommand::new()
            .ignore_buildable()
            .ignore_pull_failures()
            .include_deps()
            .quiet()
            .service("web");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--ignore-buildable".to_string()));
        assert!(args.contains(&"--ignore-pull-failures".to_string()));
        assert!(args.contains(&"--include-deps".to_string()));
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"web".to_string()));
    }

    #[test]
    fn test_compose_pull_with_policy() {
        let cmd = ComposePullCommand::new()
            .policy(ComposePullPolicy::Always)
            .service("db");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--policy".to_string()));
        assert!(args.contains(&"always".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_compose_pull_with_missing_policy() {
        let cmd = ComposePullCommand::new().policy(ComposePullPolicy::Missing);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--policy".to_string()));
        assert!(args.contains(&"missing".to_string()));
    }

    #[test]
    fn test_compose_pull_multiple_services() {
        let cmd = ComposePullCommand::new()
            .service("web")
            .service("db")
            .service("redis");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
        assert!(args.contains(&"redis".to_string()));
    }

    #[test]
    fn test_compose_pull_services_batch() {
        let cmd = ComposePullCommand::new().services(vec!["web", "db"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"db".to_string()));
    }

    #[test]
    fn test_compose_pull_config_integration() {
        let cmd = ComposePullCommand::new()
            .file("docker-compose.yml")
            .project_name("myapp")
            .service("api");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"myapp".to_string()));
        assert!(args.contains(&"pull".to_string()));
        assert!(args.contains(&"api".to_string()));
    }
}
