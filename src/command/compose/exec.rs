//! Docker Compose exec command implementation using unified trait pattern.

use crate::{
    compose::{ComposeCommand, ComposeConfig},
    error::Result,
    CommandExecutor, DockerCommand,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Docker Compose exec command builder.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // multiple boolean flags are appropriate for exec command
pub struct ComposeExecCommand {
    /// Base command executor.
    pub executor: CommandExecutor,
    /// Base compose configuration.
    pub config: ComposeConfig,
    /// Service to execute command in.
    pub service: String,
    /// Command and arguments to execute.
    pub command: Vec<String>,
    /// Runs in detached mode.
    pub detach: bool,
    /// Disables pseudo-TTY allocation.
    pub no_tty: bool,
    /// Keeps STDIN open even if not attached.
    pub interactive: bool,
    /// Runs as specified user.
    pub user: Option<String>,
    /// Working directory inside the container.
    pub workdir: Option<String>,
    /// Sets environment variables.
    pub env: HashMap<String, String>,
    /// Container index (if service has multiple instances).
    pub index: Option<u32>,
    /// Uses privileged mode.
    pub privileged: bool,
}

/// Result from [`ComposeExecCommand`].
#[derive(Debug, Clone)]
pub struct ComposeExecResult {
    /// Raw stdout output.
    pub stdout: String,
    /// Raw stderr output.
    pub stderr: String,
    /// Success status.
    pub success: bool,
    /// Exit code from the command.
    pub exit_code: i32,
    /// Service that the command was executed in.
    pub service: String,
    /// Whether the command was run in detached mode.
    pub detached: bool,
}

impl ComposeExecCommand {
    /// Creates a new [`ComposeExecCommand`].
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            executor: CommandExecutor::new(),
            config: ComposeConfig::new(),
            service: service.into(),
            command: Vec::new(),
            detach: false,
            no_tty: false,
            interactive: false,
            user: None,
            workdir: None,
            env: HashMap::new(),
            index: None,
            privileged: false,
        }
    }

    /// Sets the command to execute.
    #[must_use]
    pub fn cmd<I, S>(mut self, command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command = command.into_iter().map(Into::into).collect();
        self
    }

    /// Adds a command argument.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.command.push(arg.into());
        self
    }

    /// Adds multiple arguments.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command.extend(args.into_iter().map(Into::into));
        self
    }

    /// Runs in detached mode.
    #[must_use]
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }

    /// Disables pseudo-TTY allocation.
    #[must_use]
    pub fn no_tty(mut self) -> Self {
        self.no_tty = true;
        self
    }

    /// Keeps STDIN open even if not attached.
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// Runs as specified user.
    #[must_use]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets working directory inside the container.
    #[must_use]
    pub fn workdir(mut self, workdir: impl Into<String>) -> Self {
        self.workdir = Some(workdir.into());
        self
    }

    /// Sets an environment variable.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Sets multiple environment variables.
    #[must_use]
    pub fn envs(mut self, env_vars: HashMap<String, String>) -> Self {
        self.env.extend(env_vars);
        self
    }

    /// Sets container index (for services with multiple instances).
    #[must_use]
    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    /// Uses privileged mode.
    #[must_use]
    pub fn privileged(mut self) -> Self {
        self.privileged = true;
        self
    }
}

#[async_trait]
impl DockerCommand for ComposeExecCommand {
    type Output = ComposeExecResult;

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

        Ok(ComposeExecResult {
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
            exit_code: output.exit_code,
            service: self.service.clone(),
            detached: self.detach,
        })
    }
}

impl ComposeCommand for ComposeExecCommand {
    fn subcommand_name() -> &'static str {
        "exec"
    }

    fn config(&self) -> &ComposeConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut ComposeConfig {
        &mut self.config
    }

    fn build_subcommand_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.detach {
            args.push("--detach".to_string());
        }

        if self.no_tty {
            args.push("--no-TTY".to_string());
        }

        if self.interactive {
            args.push("--interactive".to_string());
        }

        // add user
        if let Some(ref user) = self.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        // add working directory
        if let Some(ref workdir) = self.workdir {
            args.push("--workdir".to_string());
            args.push(workdir.clone());
        }

        // add environment variables
        for (key, value) in &self.env {
            args.push("--env".to_string());
            args.push(format!("{key}={value}"));
        }

        // add container index
        if let Some(index) = self.index {
            args.push("--index".to_string());
            args.push(index.to_string());
        }

        if self.privileged {
            args.push("--privileged".to_string());
        }

        // add service name
        args.push(self.service.clone());

        // add command and arguments
        args.extend(self.command.clone());

        args
    }
}

impl ComposeExecResult {
    /// Checks if the command was successful.
    #[must_use]
    pub fn success(&self) -> bool {
        self.success
    }

    /// Gets the exit code from the command.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Gets the service that the command was executed in.
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }

    /// Checks if the command was run in detached mode.
    #[must_use]
    pub fn is_detached(&self) -> bool {
        self.detached
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_exec_basic() {
        let cmd = ComposeExecCommand::new("web");
        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"web".to_string()));

        let full_args = ComposeCommand::build_command_args(&cmd);
        assert_eq!(full_args[0], "compose");
        assert!(full_args.contains(&"exec".to_string()));
        assert!(full_args.contains(&"web".to_string()));
    }

    #[test]
    fn test_compose_exec_with_command() {
        let cmd = ComposeExecCommand::new("db").cmd(vec!["psql", "-U", "postgres"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"db".to_string()));
        assert!(args.contains(&"psql".to_string()));
        assert!(args.contains(&"-U".to_string()));
        assert!(args.contains(&"postgres".to_string()));
    }

    #[test]
    fn test_compose_exec_with_flags() {
        let cmd = ComposeExecCommand::new("app")
            .detach()
            .no_tty()
            .interactive()
            .privileged();

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--no-TTY".to_string()));
        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--privileged".to_string()));
    }

    #[test]
    fn test_compose_exec_with_user_and_workdir() {
        let cmd = ComposeExecCommand::new("web")
            .user("root")
            .workdir("/app")
            .cmd(vec!["bash"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"root".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/app".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"bash".to_string()));
    }

    #[test]
    fn test_compose_exec_with_env_vars() {
        let cmd = ComposeExecCommand::new("worker")
            .env("DEBUG", "1")
            .env("NODE_ENV", "development")
            .cmd(vec!["npm", "test"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--env".to_string()));
        assert!(args.contains(&"DEBUG=1".to_string()));
        assert!(args.contains(&"NODE_ENV=development".to_string()));
    }

    #[test]
    fn test_compose_exec_with_index() {
        let cmd = ComposeExecCommand::new("web")
            .index(2)
            .cmd(vec!["ps", "aux"]);

        let args = cmd.build_subcommand_args();
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"2".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"ps".to_string()));
        assert!(args.contains(&"aux".to_string()));
    }

    #[test]
    fn test_compose_exec_all_options() {
        let cmd = ComposeExecCommand::new("api")
            .detach()
            .user("www-data")
            .workdir("/var/www")
            .env("PHP_ENV", "production")
            .index(1)
            .privileged()
            .cmd(vec!["php", "-v"]);

        let args = cmd.build_subcommand_args();

        // Check flags
        assert!(args.contains(&"--detach".to_string()));
        assert!(args.contains(&"--privileged".to_string()));

        // Check parameters
        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"www-data".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/var/www".to_string()));
        assert!(args.contains(&"--env".to_string()));
        assert!(args.contains(&"PHP_ENV=production".to_string()));
        assert!(args.contains(&"--index".to_string()));
        assert!(args.contains(&"1".to_string()));

        // Check service and command
        assert!(args.contains(&"api".to_string()));
        assert!(args.contains(&"php".to_string()));
        assert!(args.contains(&"-v".to_string()));
    }

    #[test]
    fn test_compose_config_integration() {
        let cmd = ComposeExecCommand::new("database")
            .file("docker-compose.yml")
            .project_name("my-project")
            .user("postgres")
            .cmd(vec!["psql", "-c", "SELECT 1"]);

        let args = ComposeCommand::build_command_args(&cmd);
        assert!(args.contains(&"--file".to_string()));
        assert!(args.contains(&"docker-compose.yml".to_string()));
        assert!(args.contains(&"--project-name".to_string()));
        assert!(args.contains(&"my-project".to_string()));
        assert!(args.contains(&"--user".to_string()));
        assert!(args.contains(&"postgres".to_string()));
        assert!(args.contains(&"database".to_string()));
        assert!(args.contains(&"psql".to_string()));
    }
}
