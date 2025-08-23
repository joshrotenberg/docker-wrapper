//! Docker Compose up command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Docker Compose up command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are needed for compose up options
pub struct ComposeUpCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Services to start (empty for all)
    pub services: Vec<String>,
    /// Run in detached mode
    pub detach: bool,
    /// Don't start linked services
    pub no_deps: bool,
    /// Force recreate containers
    pub force_recreate: bool,
    /// Recreate containers even if configuration unchanged
    pub always_recreate_deps: bool,
    /// Don't recreate containers
    pub no_recreate: bool,
    /// Don't build images
    pub no_build: bool,
    /// Don't start services
    pub no_start: bool,
    /// Build images before starting
    pub build: bool,
    /// Remove orphan containers
    pub remove_orphans: bool,
    /// Scale SERVICE to NUM instances
    pub scale: Vec<(String, u32)>,
    /// Timeout for container shutdown
    pub timeout: Option<Duration>,
    /// Exit code from first container that stops
    pub exit_code_from: Option<String>,
    /// Abort if containers are stopped
    pub abort_on_container_exit: bool,
    /// Attach to dependent containers
    pub attach_dependencies: bool,
    /// Recreate anonymous volumes
    pub renew_anon_volumes: bool,
    /// Wait for services to be healthy
    pub wait: bool,
    /// Maximum wait timeout
    pub wait_timeout: Option<Duration>,
    /// Pull image policy
    pub pull: Option<PullPolicy>,
}

/// Image pull policy
#[derive(Debug, Clone, Copy)]
pub enum PullPolicy {
    /// Always pull images
    Always,
    /// Never pull images
    Never,
    /// Pull missing images (default)
    Missing,
}

impl std::fmt::Display for PullPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
            Self::Missing => write!(f, "missing"),
        }
    }
}

/// Result from compose up command
#[derive(Debug, Clone)]
pub struct ComposeUpResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output  
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Services that were started
    pub services: Vec<String>,
    /// Whether running in detached mode
    pub detached: bool,
}

impl ComposeUpCommand {
    /// Create a new compose up command
    #[must_use]
    pub fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            services: Vec::new(),
            detach: false,
            no_deps: false,
            force_recreate: false,
            always_recreate_deps: false,
            no_recreate: false,
            no_build: false,
            no_start: false,
            build: false,
            remove_orphans: false,
            scale: Vec::new(),
            timeout: None,
            exit_code_from: None,
            abort_on_container_exit: false,
            attach_dependencies: false,
            renew_anon_volumes: false,
            wait: false,
            wait_timeout: None,
            pull: None,
        }
    }

    /// Add a service to start
    #[must_use]
    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Add multiple services
    #[must_use]
    pub fn services<I, S>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.services.extend(services.into_iter().map(Into::into));
        self
    }

    /// Run in detached mode
    #[must_use]
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }

    /// Don't start linked services
    #[must_use]
    pub fn no_deps(mut self) -> Self {
        self.no_deps = true;
        self
    }

    /// Force recreate containers
    #[must_use]
    pub fn force_recreate(mut self) -> Self {
        self.force_recreate = true;
        self
    }

    /// Always recreate dependent containers
    #[must_use]
    pub fn always_recreate_deps(mut self) -> Self {
        self.always_recreate_deps = true;
        self
    }

    /// Don't recreate containers
    #[must_use]
    pub fn no_recreate(mut self) -> Self {
        self.no_recreate = true;
        self
    }

    /// Don't build images
    #[must_use]
    pub fn no_build(mut self) -> Self {
        self.no_build = true;
        self
    }

    /// Don't start services after creating
    #[must_use]
    pub fn no_start(mut self) -> Self {
        self.no_start = true;
        self
    }

    /// Build images before starting
    #[must_use]
    pub fn build(mut self) -> Self {
        self.build = true;
        self
    }

    /// Remove orphan containers
    #[must_use]
    pub fn remove_orphans(mut self) -> Self {
        self.remove_orphans = true;
        self
    }

    /// Scale a service to a specific number of instances
    #[must_use]
    pub fn scale(mut self, service: impl Into<String>, instances: u32) -> Self {
        self.scale.push((service.into(), instances));
        self
    }

    /// Set timeout for container shutdown
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Use exit code from specific container
    #[must_use]
    pub fn exit_code_from(mut self, service: impl Into<String>) -> Self {
        self.exit_code_from = Some(service.into());
        self
    }

    /// Abort when containers stop
    #[must_use]
    pub fn abort_on_container_exit(mut self) -> Self {
        self.abort_on_container_exit = true;
        self
    }

    /// Attach to dependent containers
    #[must_use]
    pub fn attach_dependencies(mut self) -> Self {
        self.attach_dependencies = true;
        self
    }

    /// Recreate anonymous volumes
    #[must_use]
    pub fn renew_anon_volumes(mut self) -> Self {
        self.renew_anon_volumes = true;
        self
    }

    /// Wait for services to be running/healthy
    #[must_use]
    pub fn wait(mut self) -> Self {
        self.wait = true;
        self
    }

    /// Set maximum wait timeout
    #[must_use]
    pub fn wait_timeout(mut self, timeout: Duration) -> Self {
        self.wait_timeout = Some(timeout);
        self
    }

    /// Set pull policy
    #[must_use]
    pub fn pull(mut self, policy: PullPolicy) -> Self {
        self.pull = Some(policy);
        self
    }
}

impl Default for ComposeUpCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DockerCommand for ComposeUpCommand {
    type Output = ComposeUpResult;

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

        Ok(ComposeUpResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            services: self.services.clone(),
            detached: self.detach,
        })
    }
}

impl ComposeCommand for ComposeUpCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "up"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.detach {
            args.push("--detach".to_string());
        }

        if self.no_deps {
            args.push("--no-deps".to_string());
        }

        if self.force_recreate {
            args.push("--force-recreate".to_string());
        }

        if self.always_recreate_deps {
            args.push("--always-recreate-deps".to_string());
        }

        if self.no_recreate {
            args.push("--no-recreate".to_string());
        }

        if self.no_build {
            args.push("--no-build".to_string());
        }

        if self.no_start {
            args.push("--no-start".to_string());
        }

        if self.build {
            args.push("--build".to_string());
        }

        if self.remove_orphans {
            args.push("--remove-orphans".to_string());
        }

        for (service, count) in &self.scale {
            args.push("--scale".to_string());
            args.push(format!("{service}={count}"));
        }

        if let Some(timeout) = self.timeout {
            args.push("--timeout".to_string());
            args.push(timeout.as_secs().to_string());
        }

        if let Some(ref service) = self.exit_code_from {
            args.push("--exit-code-from".to_string());
            args.push(service.clone());
        }

        if self.abort_on_container_exit {
            args.push("--abort-on-container-exit".to_string());
        }

        if self.attach_dependencies {
            args.push("--attach-dependencies".to_string());
        }

        if self.renew_anon_volumes {
            args.push("--renew-anon-volumes".to_string());
        }

        if self.wait {
            args.push("--wait".to_string());
        }

        if let Some(timeout) = self.wait_timeout {
            args.push("--wait-timeout".to_string());
            args.push(timeout.as_secs().to_string());
        }

        if let Some(ref pull) = self.pull {
            args.push("--pull".to_string());
            args.push(pull.to_string());
        }

        // Add service names at the end
        args.extend(self.services.clone());

        args
    }
}

impl ComposeUpResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the services that were started
    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }

    /// Check if running in detached mode
    #[must_use]
    pub fn is_detached(&self) -> bool {
        self.detached
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_up_basic() {
        let cmd = ComposeUpCommand::new();
        let args = cmd.build_subcommand_args();
        assert!(args.is_empty());

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"up".to_string()));
    }

    #[test]
    fn test_compose_up_detached() {
        let cmd = ComposeUpCommand::new().detach();
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["--detach"]);
    }

    #[test]
    fn test_compose_up_with_services() {
        let cmd = ComposeUpCommand::new().service("web").service("db");
        let args = cmd.build_subcommand_args();
        assert_eq!(args, vec!["web", "db"]);
    }

    #[test]
    fn test_compose_up_all_options() {
        let cmd = ComposeUpCommand::new()
            .detach()
            .build()
            .remove_orphans()
            .scale("web", 3)
            .wait()
            .pull(PullPolicy::Always)
            .service("web")
            .service("db");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--build".to_string()));
        assert!(args.contains(&"--remove-orphans".to_string()));
        assert!(args.contains(&"--scale".to_string()));
        assert!(args.contains(&"web=3".to_string()));
        assert!(args.contains(&"--wait".to_string()));
        assert!(args.contains(&"--pull".to_string()));
        assert!(args.contains(&"always".to_string()));
    }

    #[test]
    fn test_pull_policy_display() {
        assert_eq!(PullPolicy::Always.to_string(), "always");
        assert_eq!(PullPolicy::Never.to_string(), "never");
        assert_eq!(PullPolicy::Missing.to_string(), "missing");
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeUpCommand::new()
            .file("docker-compose.yml")
            .project_name("my-project")
            .detach()
            .service("web");

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"web".to_string()));
    }
}
