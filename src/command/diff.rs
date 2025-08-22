//! Docker diff command implementation.
//!
//! This module provides the `docker diff` command for inspecting filesystem changes in a container.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;
use std::ffi::OsStr;

/// Docker diff command builder
///
/// Inspect changes to files or folders on a container's filesystem.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::DiffCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Show filesystem changes
/// let changes = DiffCommand::new("my-container")
///     .run()
///     .await?;
///
/// for change in changes.filesystem_changes() {
///     println!("{}: {}", change.change_type, change.path);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct DiffCommand {
    /// Container name or ID
    container: String,
    /// Command executor
    executor: CommandExecutor,
}

impl DiffCommand {
    /// Create a new diff command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::DiffCommand;
    ///
    /// let cmd = DiffCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            executor: CommandExecutor::new(),
        }
    }

    /// Execute the diff command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::DiffCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = DiffCommand::new("my-container")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Filesystem changes:");
    ///     for change in result.filesystem_changes() {
    ///         println!("{}: {}", change.change_type, change.path);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<DiffResult> {
        let output = self.execute().await?;

        // Parse filesystem changes from output
        let filesystem_changes = Self::parse_filesystem_changes(&output.stdout);

        Ok(DiffResult {
            output,
            container: self.container.clone(),
            filesystem_changes,
        })
    }

    /// Parse filesystem changes from diff command output
    fn parse_filesystem_changes(stdout: &str) -> Vec<FilesystemChange> {
        let mut changes = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Format: "C /path/to/file" where C is change type (A, D, C)
            if line.len() > 2 {
                let change_char = line.chars().next().unwrap_or(' ');
                let path = &line[2..]; // Skip change character and space

                let change_type = match change_char {
                    'A' => FilesystemChangeType::Added,
                    'D' => FilesystemChangeType::Deleted,
                    'C' => FilesystemChangeType::Changed,
                    _ => FilesystemChangeType::Unknown(change_char.to_string()),
                };

                changes.push(FilesystemChange {
                    change_type,
                    path: path.to_string(),
                    raw_line: line.to_string(),
                });
            }
        }

        changes
    }
}

#[async_trait]
impl DockerCommand for DiffCommand {
    type Output = CommandOutput;

    fn command_name(&self) -> &'static str {
        "diff"
    }

    fn build_args(&self) -> Vec<String> {
        vec![self.container.clone()]
    }

    async fn execute(&self) -> Result<Self::Output> {
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

/// Result from the diff command
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Container that was inspected
    pub container: String,
    /// Parsed filesystem changes
    pub filesystem_changes: Vec<FilesystemChange>,
}

impl DiffResult {
    /// Check if the diff command was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the container name
    #[must_use]
    pub fn container(&self) -> &str {
        &self.container
    }

    /// Get the filesystem changes
    #[must_use]
    pub fn filesystem_changes(&self) -> &[FilesystemChange] {
        &self.filesystem_changes
    }

    /// Get the raw command output
    #[must_use]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }

    /// Get change count
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.filesystem_changes.len()
    }

    /// Check if there are any changes
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.filesystem_changes.is_empty()
    }

    /// Get changes by type
    #[must_use]
    pub fn changes_by_type(&self, change_type: &FilesystemChangeType) -> Vec<&FilesystemChange> {
        self.filesystem_changes
            .iter()
            .filter(|change| &change.change_type == change_type)
            .collect()
    }
}

/// Information about a filesystem change in a container
#[derive(Debug, Clone)]
pub struct FilesystemChange {
    /// Type of change (Added, Deleted, Changed)
    pub change_type: FilesystemChangeType,
    /// Path that was changed
    pub path: String,
    /// Raw output line
    pub raw_line: String,
}

/// Type of filesystem change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilesystemChangeType {
    /// File or directory was added
    Added,
    /// File or directory was deleted
    Deleted,
    /// File or directory was changed
    Changed,
    /// Unknown change type with the raw character
    Unknown(String),
}

impl std::fmt::Display for FilesystemChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "Added"),
            Self::Deleted => write!(f, "Deleted"),
            Self::Changed => write!(f, "Changed"),
            Self::Unknown(char) => write!(f, "Unknown({char})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_basic() {
        let cmd = DiffCommand::new("test-container");
        let args = cmd.build_args();
        assert_eq!(args, vec!["test-container"]);
    }

    #[test]
    fn test_parse_filesystem_changes() {
        let output = "A /new/file.txt\nD /deleted/file.txt\nC /changed/file.txt";
        let changes = DiffCommand::parse_filesystem_changes(output);

        assert_eq!(changes.len(), 3);

        assert_eq!(changes[0].change_type, FilesystemChangeType::Added);
        assert_eq!(changes[0].path, "/new/file.txt");

        assert_eq!(changes[1].change_type, FilesystemChangeType::Deleted);
        assert_eq!(changes[1].path, "/deleted/file.txt");

        assert_eq!(changes[2].change_type, FilesystemChangeType::Changed);
        assert_eq!(changes[2].path, "/changed/file.txt");
    }

    #[test]
    fn test_parse_filesystem_changes_empty() {
        let changes = DiffCommand::parse_filesystem_changes("");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_parse_filesystem_changes_unknown_type() {
        let output = "X /unknown/file.txt";
        let changes = DiffCommand::parse_filesystem_changes(output);

        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes[0].change_type,
            FilesystemChangeType::Unknown("X".to_string())
        );
        assert_eq!(changes[0].path, "/unknown/file.txt");
    }

    #[test]
    fn test_filesystem_change_type_display() {
        assert_eq!(FilesystemChangeType::Added.to_string(), "Added");
        assert_eq!(FilesystemChangeType::Deleted.to_string(), "Deleted");
        assert_eq!(FilesystemChangeType::Changed.to_string(), "Changed");
        assert_eq!(
            FilesystemChangeType::Unknown("X".to_string()).to_string(),
            "Unknown(X)"
        );
    }

    #[test]
    fn test_diff_result_helpers() {
        let result = DiffResult {
            output: CommandOutput {
                stdout: "A /new\nD /old".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            container: "test".to_string(),
            filesystem_changes: vec![
                FilesystemChange {
                    change_type: FilesystemChangeType::Added,
                    path: "/new".to_string(),
                    raw_line: "A /new".to_string(),
                },
                FilesystemChange {
                    change_type: FilesystemChangeType::Deleted,
                    path: "/old".to_string(),
                    raw_line: "D /old".to_string(),
                },
            ],
        };

        assert!(result.has_changes());
        assert_eq!(result.change_count(), 2);

        let added = result.changes_by_type(&FilesystemChangeType::Added);
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].path, "/new");

        let deleted = result.changes_by_type(&FilesystemChangeType::Deleted);
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].path, "/old");
    }
}
