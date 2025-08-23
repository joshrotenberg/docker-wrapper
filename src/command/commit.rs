//! Docker commit command implementation.
//!
//! This module provides the `docker commit` command for creating a new image
//! from a container's changes.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker commit command builder
///
/// Create a new image from a container's changes.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::CommitCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Commit container changes to a new image
/// let image_id = CommitCommand::new("my-container")
///     .repository("myapp")
///     .tag("v2.0")
///     .message("Updated configuration")
///     .author("Developer <dev@example.com>")
///     .run()
///     .await?;
///
/// println!("Created image: {}", image_id.image_id());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CommitCommand {
    /// Container name or ID
    container: String,
    /// Repository name
    repository: Option<String>,
    /// Tag name
    tag: Option<String>,
    /// Commit message
    message: Option<String>,
    /// Author
    author: Option<String>,
    /// Pause container during commit
    pause: bool,
    /// Dockerfile instructions to apply
    changes: Vec<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl CommitCommand {
    /// Create a new commit command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CommitCommand;
    ///
    /// let cmd = CommitCommand::new("my-container");
    /// ```
    #[must_use]
    pub fn new(container: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            repository: None,
            tag: None,
            message: None,
            author: None,
            pause: true, // Docker default is true
            changes: Vec::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Set the repository name for the new image
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CommitCommand;
    ///
    /// let cmd = CommitCommand::new("my-container")
    ///     .repository("myapp");
    /// ```
    #[must_use]
    pub fn repository(mut self, repository: impl Into<String>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Set the tag for the new image
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CommitCommand;
    ///
    /// let cmd = CommitCommand::new("my-container")
    ///     .repository("myapp")
    ///     .tag("v2.0");
    /// ```
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set the commit message
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the author
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CommitCommand;
    ///
    /// let cmd = CommitCommand::new("my-container")
    ///     .author("John Doe <john@example.com>");
    /// ```
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Do not pause the container during commit
    #[must_use]
    pub fn no_pause(mut self) -> Self {
        self.pause = false;
        self
    }

    /// Apply Dockerfile instruction to the created image
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::CommitCommand;
    ///
    /// let cmd = CommitCommand::new("my-container")
    ///     .change("ENV VERSION=2.0")
    ///     .change("EXPOSE 8080")
    ///     .change("CMD [\"app\", \"--production\"]");
    /// ```
    #[must_use]
    pub fn change(mut self, change: impl Into<String>) -> Self {
        self.changes.push(change.into());
        self
    }

    /// Execute the commit command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The container doesn't exist
    /// - The repository/tag format is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::CommitCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = CommitCommand::new("my-container")
    ///     .repository("myapp")
    ///     .tag("snapshot")
    ///     .run()
    ///     .await?;
    ///
    /// println!("New image ID: {}", result.image_id());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<CommitResult> {
        let output = self.execute().await?;

        // Parse image ID from output
        let image_id = output.stdout.trim().to_string();

        Ok(CommitResult { output, image_id })
    }
}

#[async_trait]
impl DockerCommand for CommitCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["commit".to_string()];

        if let Some(ref author) = self.author {
            args.push("--author".to_string());
            args.push(author.clone());
        }

        for change in &self.changes {
            args.push("--change".to_string());
            args.push(change.clone());
        }

        if let Some(ref message) = self.message {
            args.push("--message".to_string());
            args.push(message.clone());
        }

        if !self.pause {
            args.push("--pause=false".to_string());
        }

        // Add container name/ID
        args.push(self.container.clone());

        // Add repository[:tag] if specified
        if let Some(ref repo) = self.repository {
            let mut image_name = repo.clone();
            if let Some(ref tag) = self.tag {
                image_name.push(':');
                image_name.push_str(tag);
            }
            args.push(image_name);
        }

        args.extend(self.executor.raw_args.clone());
        args
    }

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }
}

/// Result from the commit command
#[derive(Debug, Clone)]
pub struct CommitResult {
    /// Raw command output
    pub output: CommandOutput,
    /// ID of the created image
    pub image_id: String,
}

impl CommitResult {
    /// Check if the commit was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success && !self.image_id.is_empty()
    }

    /// Get the created image ID
    #[must_use]
    pub fn image_id(&self) -> &str {
        &self.image_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_basic() {
        let cmd = CommitCommand::new("test-container");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["commit", "test-container"]);
    }

    #[test]
    fn test_commit_with_repository() {
        let cmd = CommitCommand::new("test-container").repository("myapp");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["commit", "test-container", "myapp"]);
    }

    #[test]
    fn test_commit_with_repository_and_tag() {
        let cmd = CommitCommand::new("test-container")
            .repository("myapp")
            .tag("v2.0");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["commit", "test-container", "myapp:v2.0"]);
    }

    #[test]
    fn test_commit_with_message_and_author() {
        let cmd = CommitCommand::new("test-container")
            .message("Updated config")
            .author("Dev <dev@example.com>")
            .repository("myapp");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "commit",
                "--author",
                "Dev <dev@example.com>",
                "--message",
                "Updated config",
                "test-container",
                "myapp"
            ]
        );
    }

    #[test]
    fn test_commit_with_changes() {
        let cmd = CommitCommand::new("test-container")
            .change("ENV VERSION=2.0")
            .change("EXPOSE 8080")
            .repository("myapp");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "commit",
                "--change",
                "ENV VERSION=2.0",
                "--change",
                "EXPOSE 8080",
                "test-container",
                "myapp"
            ]
        );
    }

    #[test]
    fn test_commit_no_pause() {
        let cmd = CommitCommand::new("test-container")
            .no_pause()
            .repository("myapp");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["commit", "--pause=false", "test-container", "myapp"]
        );
    }

    #[test]
    fn test_commit_all_options() {
        let cmd = CommitCommand::new("test-container")
            .repository("myapp")
            .tag("v2.0")
            .message("Commit message")
            .author("Author Name")
            .no_pause()
            .change("ENV FOO=bar");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "commit",
                "--author",
                "Author Name",
                "--change",
                "ENV FOO=bar",
                "--message",
                "Commit message",
                "--pause=false",
                "test-container",
                "myapp:v2.0"
            ]
        );
    }
}
