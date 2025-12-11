//! Docker cp command implementation.
//!
//! This module provides the `docker cp` command for copying files/folders between
//! a container and the local filesystem.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

/// Docker cp command builder
///
/// Copy files/folders between a container and the local filesystem.
/// Use `-` as the source to read from stdin or as the destination to write to stdout.
///
/// # Exampless
///
/// ```no_run
/// use docker_wrapper::CpCommand;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Copy from container to host
/// CpCommand::from_container("my-container", "/app/config.yml")
///     .to_host(Path::new("./config.yml"))
///     .run()
///     .await?;
///
/// // Copy from host to container
/// CpCommand::from_host(Path::new("./data.txt"))
///     .to_container("my-container", "/data/data.txt")
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CpCommand {
    /// Source path (container:path or local path)
    source: String,
    /// Destination path (container:path or local path)
    destination: String,
    /// Archive mode (preserve permissions)
    archive: bool,
    /// Follow symbolic links
    follow_link: bool,
    /// Suppress progress output
    quiet: bool,
    /// Command executor
    pub executor: CommandExecutor,
}

impl CpCommand {
    /// Create a cp command copying from container to host
    ///
    /// # Exampless
    ///
    /// ```
    /// use docker_wrapper::CpCommand;
    /// use std::path::Path;
    ///
    /// let cmd = CpCommand::from_container("my-container", "/etc/config")
    ///     .to_host(Path::new("./config"));
    /// ```
    #[must_use]
    pub fn from_container(container: impl Into<String>, path: impl Into<String>) -> Self {
        let source = format!("{}:{}", container.into(), path.into());
        Self {
            source,
            destination: String::new(),
            archive: false,
            follow_link: false,
            quiet: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Create a cp command copying from host to container
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::CpCommand;
    /// use std::path::Path;
    ///
    /// let cmd = CpCommand::from_host(Path::new("./file.txt"))
    ///     .to_container("my-container", "/app/file.txt");
    /// ```
    #[must_use]
    pub fn from_host(path: &Path) -> Self {
        Self {
            source: path.to_string_lossy().into_owned(),
            destination: String::new(),
            archive: false,
            follow_link: false,
            quiet: false,
            executor: CommandExecutor::new(),
        }
    }

    /// Set destination on host filesystem
    #[must_use]
    pub fn to_host(mut self, path: &Path) -> Self {
        self.destination = path.to_string_lossy().into_owned();
        self
    }

    /// Set destination in container
    #[must_use]
    pub fn to_container(mut self, container: impl Into<String>, path: impl Into<String>) -> Self {
        self.destination = format!("{}:{}", container.into(), path.into());
        self
    }

    /// Archive mode - preserve UIDs/GIDs and permissions
    ///
    /// # Examples
    ///
    /// ```
    /// use docker_wrapper::CpCommand;
    /// use std::path::Path;
    ///
    /// let cmd = CpCommand::from_container("my-container", "/app")
    ///     .to_host(Path::new("./app-backup"))
    ///     .archive();
    /// ```
    #[must_use]
    pub fn archive(mut self) -> Self {
        self.archive = true;
        self
    }

    /// Follow symbolic links in source
    #[must_use]
    pub fn follow_link(mut self) -> Self {
        self.follow_link = true;
        self
    }

    /// Suppress progress output during copy
    #[must_use]
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Execute the cp command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    /// - The source path doesn't exist
    /// - Permission denied for destination
    pub async fn run(&self) -> Result<CpResult> {
        let output = self.execute().await?;
        Ok(CpResult {
            output,
            source: self.source.clone(),
            destination: self.destination.clone(),
        })
    }
}

#[async_trait]
impl DockerCommand for CpCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["cp".to_string()];

        if self.archive {
            args.push("--archive".to_string());
        }

        if self.follow_link {
            args.push("--follow-link".to_string());
        }

        if self.quiet {
            args.push("--quiet".to_string());
        }

        // Add source and destination
        args.push(self.source.clone());
        args.push(self.destination.clone());

        args.extend(self.executor.raw_args.clone());
        args
    }

    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        if self.destination.is_empty() {
            return Err(crate::error::Error::invalid_config(
                "Destination not specified",
            ));
        }

        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }
}

/// Result from the cp command
#[derive(Debug, Clone)]
pub struct CpResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Source path that was copied
    pub source: String,
    /// Destination path
    pub destination: String,
}

impl CpResult {
    /// Check if the copy was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the source path
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the destination path
    #[must_use]
    pub fn destination(&self) -> &str {
        &self.destination
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_from_container_to_host() {
        let cmd = CpCommand::from_container("test-container", "/app/file.txt")
            .to_host(Path::new("./file.txt"));
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["cp", "test-container:/app/file.txt", "./file.txt"]
        );
    }

    #[test]
    fn test_cp_from_host_to_container() {
        let cmd = CpCommand::from_host(Path::new("./data.txt"))
            .to_container("test-container", "/data/data.txt");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["cp", "./data.txt", "test-container:/data/data.txt"]
        );
    }

    #[test]
    fn test_cp_with_archive() {
        let cmd = CpCommand::from_container("test-container", "/app")
            .to_host(Path::new("./backup"))
            .archive();
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["cp", "--archive", "test-container:/app", "./backup"]
        );
    }

    #[test]
    fn test_cp_with_follow_link() {
        let cmd = CpCommand::from_container("test-container", "/link")
            .to_host(Path::new("./file"))
            .follow_link();
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["cp", "--follow-link", "test-container:/link", "./file"]
        );
    }

    #[test]
    fn test_cp_with_all_options() {
        let cmd = CpCommand::from_host(Path::new("./src"))
            .to_container("test-container", "/dest")
            .archive()
            .follow_link()
            .quiet();
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "cp",
                "--archive",
                "--follow-link",
                "--quiet",
                "./src",
                "test-container:/dest"
            ]
        );
    }
}
