//! Docker rm command implementation.
//!
//! This module provides the `docker rm` command for removing stopped containers.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker rm command builder
#[derive(Debug, Clone)]
pub struct RmCommand {
    /// Container names or IDs to remove
    containers: Vec<String>,
    /// Force removal of running containers
    force: bool,
    /// Remove anonymous volumes associated with the container
    volumes: bool,
    /// Remove the specified link
    link: bool,
    /// Command executor
    executor: CommandExecutor,
}

impl RmCommand {
    /// Create a new rm command for a single container
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            containers: vec![container.into()],
            force: false,
            volumes: false,
            link: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a new rm command for multiple containers
    #[must_use]
    pub fn new_multiple(containers: Vec<impl Into<String>>) -> Self {
        Self {
            containers: containers.into_iter().map(Into::into).collect(),
            force: false,
            volumes: false,
            link: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Add another container to remove
    #[must_use]
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.containers.push(container.into());
        self
    }

    /// Force removal of running containers (uses SIGKILL)
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Remove anonymous volumes associated with the container
    #[must_use]
    pub fn volumes(mut self) -> Self {
        self.volumes = true;
        self
    }

    /// Remove the specified link
    #[must_use]
    pub fn link(mut self) -> Self {
        self.link = true;
        self
    }

    /// Execute the rm command
    ///
    /// # Errors
    /// Returns an error if:
    /// - No containers are specified
    /// - The Docker daemon is not running
    /// - Any of the specified containers don't exist
    /// - A container is running and force flag is not set
    pub async fn run(&self) -> Result<RmResult> {
        let output = self.execute().await?;

        // Parse removed container IDs from output
        let removed: Vec<String> = output
            .stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();

        Ok(RmResult { removed, output })
    }
}

#[async_trait]
impl DockerCommand for RmCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "rm"
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.force {
            args.push("--force".to_string());
        }

        if self.volumes {
            args.push("--volumes".to_string());
        }

        if self.link {
            args.push("--link".to_string());
        }

        // Add container names/IDs
        args.extend(self.containers.clone());

        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.containers.is_empty() {
            return Err(Error::invalid_config("No containers specified for removal"));
        }

        self.executor
            .execute_command(self.command_name(), self.build_args())
            .await
    }

    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.executor.add_arg(arg);
        self
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.executor.add_args(args);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.executor.add_flag(flag);
        self
    }

    fn option(&mut self, key: &str, value: &str) -> &mut Self {
        self.executor.add_option(key, value);
        self
    }
}

/// Result from the rm command
#[derive(Debug, Clone)]
pub struct RmResult {
    /// List of removed container IDs
    pub removed: Vec<String>,
    /// Raw command output
    pub output: CommandOutput,
}

impl RmResult {
    /// Check if all containers were removed successfully
    #[must_use]
    pub fn all_removed(&self) -> bool {
        self.output.success
    }

    /// Get the number of containers removed
    #[must_use]
    pub fn count(&self) -> usize {
        self.removed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_single_container() {
        let cmd = RmCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_rm_multiple_containers() {
        let cmd = RmCommand::new_multiple(vec!["container1", "container2", "container3"]);
        let args = cmd.build_args();
        assert_eq!(args, vec!["container1", "container2", "container3"]);
    }

    #[test]
    fn test_rm_with_force() {
        let cmd = RmCommand::new("test-container").force();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--force", "test-container"]);
    }

    #[test]
    fn test_rm_with_volumes() {
        let cmd = RmCommand::new("test-container").volumes();
        let args = cmd.build_args();
        assert_eq!(args, vec!["--volumes", "test-container"]);
    }

    #[test]
    fn test_rm_with_all_options() {
        let cmd = RmCommand::new("test-container").force().volumes().link();
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec!["--force", "--volumes", "--link", "test-container"]
        );
    }

    #[test]
    fn test_rm_builder_chain() {
        let cmd = RmCommand::new("container1")
            .container("container2")
            .container("container3")
            .force()
            .volumes();
        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "--force",
                "--volumes",
                "container1",
                "container2",
                "container3"
            ]
        );
    }
}
