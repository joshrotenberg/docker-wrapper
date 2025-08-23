//! Docker Compose run command implementation using unified trait pattern.

use super::{CommandExecutor, ComposeCommand, ComposeConfig, DockerCommandV2};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker Compose run command builder
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Multiple boolean flags are appropriate for run command
pub struct ComposeRunCommand {
    /// Base command executor
    pub executor: CommandExecutor,
    /// Base compose configuration
    pub config: ComposeConfig,
    /// Service to run
    pub service: String,
    /// Command and arguments to run
    pub command: Vec<String>,
    /// Run container in background
    pub detach: bool,
    /// Automatically remove the container when it exits
    pub rm: bool,
    /// Don't start linked services
    pub no_deps: bool,
    /// Disable pseudo-TTY allocation
    pub no_tty: bool,
    /// Keep STDIN open even if not attached
    pub interactive: bool,
    /// Override the entrypoint
    pub entrypoint: Option<String>,
    /// Set environment variables
    pub env: HashMap<String, String>,
    /// Add or override labels
    pub labels: HashMap<String, String>,
    /// Container name
    pub name: Option<String>,
    /// Publish container ports to host
    pub publish: Vec<String>,
    /// Run as specified user
    pub user: Option<String>,
    /// Working directory inside the container
    pub workdir: Option<String>,
    /// Bind mount volumes
    pub volumes: Vec<String>,
    /// Remove associated volumes when container is removed
    pub volume_rm: bool,
}

/// Result from compose run command
#[derive(Debug, Clone)]
pub struct ComposeRunResult {
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
    /// Success status
    pub success: bool,
    /// Exit code from the container
    pub exit_code: i32,
    /// Service that was run
    pub service: String,
    /// Whether the container was run in detached mode
    pub detached: bool,
}

impl ComposeRunCommand {
    /// Create a new compose run command
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            service: service.into(),
            command: Vec::new(),
            detach: false,
            rm: false,
            no_deps: false,
            no_tty: false,
            interactive: false,
            entrypoint: None,
            env: HashMap::new(),
            labels: HashMap::new(),
            name: None,
            publish: Vec::new(),
            user: None,
            workdir: None,
            volumes: Vec::new(),
            volume_rm: false,
        }
    }

    /// Set the command to run
    #[must_use]
    pub fn cmd<I, S>(mut self, command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command = command.into_iter().map(Into::into).collect();
        self
    }

    /// Add command arguments
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.command.push(arg.into());
        self
    }

    /// Add multiple arguments
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command.extend(args.into_iter().map(Into::into));
        self
    }

    /// Run container in background
    #[must_use]
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }

    /// Automatically remove the container when it exits
    #[must_use]
    pub fn rm(mut self) -> Self {
        self.rm = true;
        self
    }

    /// Don't start linked services
    #[must_use]
    pub fn no_deps(mut self) -> Self {
        self.no_deps = true;
        self
    }

    /// Disable pseudo-TTY allocation
    #[must_use]
    pub fn no_tty(mut self) -> Self {
        self.no_tty = true;
        self
    }

    /// Keep STDIN open even if not attached
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// Override the entrypoint
    #[must_use]
    pub fn entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    /// Set an environment variable
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set multiple environment variables
    #[must_use]
    pub fn envs(mut self, env_vars: HashMap<String, String>) -> Self {
        self.env.extend(env_vars);
        self
    }

    /// Add or override a label
    #[must_use]
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set multiple labels
    #[must_use]
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }

    /// Set container name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Publish a port to the host
    #[must_use]
    pub fn publish(mut self, publish: impl Into<String>) -> Self {
        self.publish.push(publish.into());
        self
    }

    /// Run as specified user
    #[must_use]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set working directory inside the container
    #[must_use]
    pub fn workdir(mut self, workdir: impl Into<String>) -> Self {
        self.workdir = Some(workdir.into());
        self
    }

    /// Bind mount a volume
    #[must_use]
    pub fn volume(mut self, volume: impl Into<String>) -> Self {
        self.volumes.push(volume.into());
        self
    }

    /// Remove associated volumes when container is removed
    #[must_use]
    pub fn volume_rm(mut self) -> Self {
        self.volume_rm = true;
        self
    }
}

#[async_trait]
impl DockerCommandV2 for ComposeRunCommand {
    type Output = ComposeRunResult;

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

        Ok(ComposeRunResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            exit_code: output.exit_code,
            service: self.service.clone(),
            detached: self.detach,
        })
    }
}

impl ComposeCommand for ComposeRunCommand {
    fn get_config(&self) -> &ComposeConfig {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn subcommand(&self) -> &'static str {
        "run"
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.detach {
            args.push("--detach".to_string());
        }

        if self.rm {
            args.push("--rm".to_string());
        }

        if self.no_deps {
            args.push("--no-deps".to_string());
        }

        if self.no_tty {
            args.push("--no-TTY".to_string());
        }

        if self.interactive {
            args.push("--interactive".to_string());
        }

        // Add entrypoint
        if let Some(ref entrypoint) = self.entrypoint {
            args.push("--entrypoint".to_string());
            args.push(entrypoint.clone());
        }

        // Add environment variables
        for (key, value) in &self.env {
            args.push("--env".to_string());
            args.push(format!("{key}={value}"));
        }

        // Add labels
        for (key, value) in &self.labels {
            args.push("--label".to_string());
            args.push(format!("{key}={value}"));
        }

        // Add container name
        if let Some(ref name) = self.name {
            args.push("--name".to_string());
            args.push(name.clone());
        }

        // Add published ports
        for publish in &self.publish {
            args.push("--publish".to_string());
            args.push(publish.clone());
        }

        // Add user
        if let Some(ref user) = self.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        // Add working directory
        if let Some(ref workdir) = self.workdir {
            args.push("--workdir".to_string());
            args.push(workdir.clone());
        }

        // Add volumes
        for volume in &self.volumes {
            args.push("--volume".to_string());
            args.push(volume.clone());
        }

        if self.volume_rm {
            args.push("--volume".to_string());
            args.push("rm".to_string());
        }

        // Add service name
        args.push(self.service.clone());

        // Add command and arguments
        args.extend(self.command.clone());

        args
    }
}

impl ComposeRunResult {
    /// Check if the command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Get the exit code from the container
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Get the service that was run
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }

    /// Check if the container was run in detached mode
    #[must_use]
    pub fn is_detached(&self) -> bool {
        self.detached
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_run_basic() {
        let cmd = ComposeRunCommand::new("web");
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"run".to_string()));
        assert!(full_args.contains(&"web".to_string()));
    }

    #[test]
    fn test_compose_run_with_command() {
        let cmd = ComposeRunCommand::new("worker").cmd(vec!["python", "script.py"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"worker".to_string()));
        assert!(args.contains(&"python".to_string()));
        assert!(args.contains(&"script.py".to_string()));
    }

    #[test]
    fn test_compose_run_with_flags() {
        let cmd = ComposeRunCommand::new("app")
            .detach()
            .rm()
            .no_deps()
            .interactive();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--rm".to_string()));
        assert!(args.contains(&"--no-deps".to_string()));
        assert!(args.contains(&"--interactive".to_string()));
    }

    #[test]
    fn test_compose_run_with_env_and_labels() {
        let cmd = ComposeRunCommand::new("test")
            .env("NODE_ENV", "development")
            .env("DEBUG", "true")
            .label("version", "1.0")
            .label("component", "api");

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--env".to_string()));
        assert!(args.contains(&"NODE_ENV=development".to_string()));
        assert!(args.contains(&"DEBUG=true".to_string()));
        assert!(args.contains(&"--label".to_string()));
        assert!(args.contains(&"version=1.0".to_string()));
        assert!(args.contains(&"component=api".to_string()));
    }

    #[test]
    fn test_compose_run_all_options() {
        let cmd = ComposeRunCommand::new("database")
            .detach()
            .rm()
            .name("test-db")
            .user("postgres")
            .workdir("/app")
            .volume("/data:/var/lib/postgresql/data")
            .publish("5432:5432")
            .entrypoint("docker-entrypoint.sh")
            .cmd(vec!["postgres"])
            .env("POSTGRES_DB", "testdb")
            .label("env", "test");

        let args = cmd.build_subcommand_args();

        // Check flags
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--rm".to_string()));

        // Check named parameters
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"test-db".to_string()));
        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"postgres".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/app".to_string()));
        assert!(args.contains(&"--volume".to_string()));
        assert!(args.contains(&"/data:/var/lib/postgresql/data".to_string()));
        assert!(args.contains(&"--publish".to_string()));
        assert!(args.contains(&"5432:5432".to_string()));
        assert!(args.contains(&"--entrypoint".to_string()));
        assert!(args.contains(&"docker-entrypoint.sh".to_string()));

        // Check service and command
        assert!(args.contains(&"database".to_string()));
        assert!(args.contains(&"postgres".to_string()));

        // Check env and labels
        assert!(args.contains(&"POSTGRES_DB=testdb".to_string()));
        assert!(args.contains(&"env=test".to_string()));
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeRunCommand::new("worker")
            .file("docker-compose.yml")
            .project_name("my-project")
            .rm()
            .cmd(vec!["python", "worker.py"]);

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--rm".to_string()));
        assert!(args.contains(&"worker".to_string()));
        assert!(args.contains(&"python".to_string()));
        assert!(args.contains(&"worker.py".to_string()));
    }
}
