//! Docker import command implementation.
//!
//! This module provides the `docker import` command for importing tarball contents as images.

use super::{CommandExecutor, CommandOutput, DockerCommand};
use crate::error::Result;
use async_trait::async_trait;

/// Docker import command builder
///
/// Import the contents from a tarball to create a filesystem image.
///
/// # Example
///
/// ```no_run
/// use docker_wrapper::ImportCommand;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Import from file
/// let result = ImportCommand::new("backup.tar")
///     .repository("my-app:imported")
///     .run()
///     .await?;
///
/// if result.success() {
///     println!("Image imported: {}", result.image_id().unwrap_or("unknown"));
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ImportCommand {
    /// Source file, URL, or - for stdin
    source: String,
    /// Repository name for the imported image
    repository: Option<String>,
    /// Commit message for the imported image
    message: Option<String>,
    /// Apply Dockerfile instructions while importing
    changes: Vec<String>,
    /// Command executor
    pub executor: CommandExecutor,
}

impl ImportCommand {
    /// Create a new import command
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::ImportCommand;
    ///
    /// // Import from file
    /// let cmd = ImportCommand::new("backup.tar");
    ///
    /// // Import from URL
    /// let cmd = ImportCommand::new("http://example.com/image.tar.gz");
    ///
    /// // Import from stdin
    /// let cmd = ImportCommand::new("-");
    /// ```
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            repository: None,
            message: None,
            changes: Vec::new(),
            executor: CommandExecutor::new(),
        }
    }

    /// Set repository name for the imported image
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::ImportCommand;
    ///
    /// let cmd = ImportCommand::new("backup.tar")
    ///     .repository("my-app:v1.0");
    /// ```
    #[must_use]
    pub fn repository(mut self, repository: impl Into<String>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Set commit message for the imported image
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::ImportCommand;
    ///
    /// let cmd = ImportCommand::new("backup.tar")
    ///     .message("Imported from production backup");
    /// ```
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Apply Dockerfile instruction while importing
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::ImportCommand;
    ///
    /// let cmd = ImportCommand::new("backup.tar")
    ///     .change("ENV PATH=/usr/local/bin:$PATH")
    ///     .change("EXPOSE 8080");
    /// ```
    #[must_use]
    pub fn change(mut self, change: impl Into<String>) -> Self {
        self.changes.push(change.into());
        self
    }

    /// Apply multiple Dockerfile instructions while importing
    ///
    /// # Example
    ///
    /// ```
    /// use docker_wrapper::ImportCommand;
    ///
    /// let cmd = ImportCommand::new("backup.tar")
    ///     .changes(vec![
    ///         "ENV NODE_ENV=production",
    ///         "EXPOSE 3000",
    ///         "CMD [\"npm\", \"start\"]"
    ///     ]);
    /// ```
    #[must_use]
    pub fn changes<I, S>(mut self, changes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.changes.extend(changes.into_iter().map(Into::into));
        self
    }

    /// Execute the import command
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Docker daemon is not running
    /// - The source file doesn't exist or is not readable
    /// - The tarball is corrupted or invalid
    /// - Network issues (for URL sources)
    /// - Insufficient disk space
    ///
    /// # Example
    ///
    /// ```no_run
    /// use docker_wrapper::ImportCommand;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = ImportCommand::new("app-backup.tar")
    ///     .repository("my-app:restored")
    ///     .message("Restored from backup")
    ///     .run()
    ///     .await?;
    ///
    /// if result.success() {
    ///     println!("Import successful!");
    ///     if let Some(image_id) = result.image_id() {
    ///         println!("New image ID: {}", image_id);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<ImportResult> {
        let output = self.execute().await?;

        // Parse image ID from output (usually the only line in stdout)
        let image_id = Self::parse_image_id(&output.stdout);

        Ok(ImportResult {
            output,
            source: self.source.clone(),
            repository: self.repository.clone(),
            image_id,
        })
    }

    /// Parse image ID from import command output
    fn parse_image_id(stdout: &str) -> Option<String> {
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return None;
        }

        // The output is typically just the image ID/digest
        Some(trimmed.to_string())
    }
}

#[async_trait]
impl DockerCommand for ImportCommand {
    type Output = CommandOutput;

    fn build_command_args(&self) -> Vec<String> {
        let mut args = vec!["import".to_string()];

        // Add message if specified
        if let Some(ref message) = self.message {
            args.push("--message".to_string());
            args.push(message.clone());
        }

        // Add changes if specified
        for change in &self.changes {
            args.push("--change".to_string());
            args.push(change.clone());
        }

        // Add source
        args.push(self.source.clone());

        // Add repository if specified
        if let Some(ref repository) = self.repository {
            args.push(repository.clone());
        }

        args.extend(self.executor.raw_args.clone());
        args
    }

    async fn execute(&self) -> Result<Self::Output> {
        let args = self.build_command_args();
        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();
        self.executor
            .execute_command(&command_name, command_args)
            .await
    }

    fn get_executor(&self) -> &CommandExecutor {
        &self.executor
    }

    fn get_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
}

/// Result from the import command
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Raw command output
    pub output: CommandOutput,
    /// Source that was imported
    pub source: String,
    /// Repository name (if specified)
    pub repository: Option<String>,
    /// Imported image ID
    pub image_id: Option<String>,
}

impl ImportResult {
    /// Check if the import was successful
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.success
    }

    /// Get the source that was imported
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the repository name
    #[must_use]
    pub fn repository(&self) -> Option<&str> {
        self.repository.as_deref()
    }

    /// Get the imported image ID
    #[must_use]
    pub fn image_id(&self) -> Option<&str> {
        self.image_id.as_deref()
    }

    /// Get the raw command output
    #[must_use]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }

    /// Check if a repository name was specified
    #[must_use]
    pub fn has_repository(&self) -> bool {
        self.repository.is_some()
    }

    /// Check if import was from stdin
    #[must_use]
    pub fn imported_from_stdin(&self) -> bool {
        self.source == "-"
    }

    /// Check if import was from URL
    #[must_use]
    pub fn imported_from_url(&self) -> bool {
        self.source.starts_with("http://") || self.source.starts_with("https://")
    }

    /// Check if import was from file
    #[must_use]
    pub fn imported_from_file(&self) -> bool {
        !self.imported_from_stdin() && !self.imported_from_url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_basic() {
        let cmd = ImportCommand::new("backup.tar");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["import", "backup.tar"]);
    }

    #[test]
    fn test_import_with_repository() {
        let cmd = ImportCommand::new("backup.tar").repository("my-app:v1.0");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["import", "backup.tar", "my-app:v1.0"]);
    }

    #[test]
    fn test_import_all_options() {
        let cmd = ImportCommand::new("backup.tar")
            .repository("my-app:v1.0")
            .message("Imported from backup")
            .change("ENV NODE_ENV=production")
            .change("EXPOSE 3000");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "import",
                "--message",
                "Imported from backup",
                "--change",
                "ENV NODE_ENV=production",
                "--change",
                "EXPOSE 3000",
                "backup.tar",
                "my-app:v1.0"
            ]
        );
    }

    #[test]
    fn test_import_with_changes() {
        let cmd = ImportCommand::new("app.tar")
            .changes(vec!["ENV PATH=/usr/local/bin:$PATH", "WORKDIR /app"]);
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec![
                "import",
                "--change",
                "ENV PATH=/usr/local/bin:$PATH",
                "--change",
                "WORKDIR /app",
                "app.tar"
            ]
        );
    }

    #[test]
    fn test_import_from_stdin() {
        let cmd = ImportCommand::new("-").repository("stdin-image");
        let args = cmd.build_command_args();
        assert_eq!(args, vec!["import", "-", "stdin-image"]);
    }

    #[test]
    fn test_import_from_url() {
        let cmd = ImportCommand::new("http://example.com/image.tar.gz").repository("remote-image");
        let args = cmd.build_command_args();
        assert_eq!(
            args,
            vec!["import", "http://example.com/image.tar.gz", "remote-image"]
        );
    }

    #[test]
    fn test_parse_image_id() {
        assert_eq!(
            ImportCommand::parse_image_id("sha256:abcd1234"),
            Some("sha256:abcd1234".to_string())
        );
        assert_eq!(ImportCommand::parse_image_id(""), None);
        assert_eq!(ImportCommand::parse_image_id("  \n  "), None);
    }

    #[test]
    fn test_import_result() {
        let result = ImportResult {
            output: CommandOutput {
                stdout: "sha256:abcd1234".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            source: "backup.tar".to_string(),
            repository: Some("my-app:v1.0".to_string()),
            image_id: Some("sha256:abcd1234".to_string()),
        };

        assert!(result.success());
        assert_eq!(result.source(), "backup.tar");
        assert_eq!(result.repository(), Some("my-app:v1.0"));
        assert_eq!(result.image_id(), Some("sha256:abcd1234"));
        assert!(result.has_repository());
        assert!(!result.imported_from_stdin());
        assert!(!result.imported_from_url());
        assert!(result.imported_from_file());
    }

    #[test]
    fn test_import_result_source_types() {
        let stdin_result = ImportResult {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            source: "-".to_string(),
            repository: None,
            image_id: None,
        };
        assert!(stdin_result.imported_from_stdin());
        assert!(!stdin_result.imported_from_url());
        assert!(!stdin_result.imported_from_file());

        let url_result = ImportResult {
            output: CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
            },
            source: "https://example.com/image.tar".to_string(),
            repository: None,
            image_id: None,
        };
        assert!(!url_result.imported_from_stdin());
        assert!(url_result.imported_from_url());
        assert!(!url_result.imported_from_file());
    }
}
